# vim:set ft= ts=4 sw=4 et:

use Test::Nginx::Socket::Lua;
use Cwd qw(cwd);

repeat_each(2);

plan tests => repeat_each() * blocks() * 5;

my $pwd = cwd();

our $HttpConfig = qq{
    lua_package_path "$pwd/lib/?.lua;;";
    lua_package_cpath "$pwd/target/debug/?.so;;";
};

no_long_string();
no_diff();

run_tests();

__DATA__

=== TEST 1: create program and context
--- http_config eval: $::HttpConfig
--- config
    location = /t {
        content_by_lua_block {
            local cel = require("cel")
            local program = cel.program
            local context = cel.context

            local p = program.new()
            local c = context.new()

            ngx.say("program created")
            ngx.say("context created")
        }
    }
--- request
GET /t
--- response_body
program created
context created
--- no_error_log
[error]
[warn]
[crit]



=== TEST 2: simple boolean expression
--- http_config eval: $::HttpConfig
--- config
    location = /t {
        content_by_lua_block {
            local cel = require("cel")
            local program = cel.program
            local context = cel.context

            local p = program.new()
            local compiled, err = p:compile("name == 'world'")
            
            if not compiled then
                ngx.say("compilation failed: " .. err)
                return
            end

            local c = context.new()
            c:add_variable("name", "world")

            local result, err = p:execute(c)
            if err then
                ngx.say("execution failed: " .. err)
                return
            end

            ngx.say(result)
        }
    }
--- request
GET /t
--- response_body
true
--- no_error_log
[error]
[warn]
[crit]



=== TEST 3: numeric comparison
--- http_config eval: $::HttpConfig
--- config
    location = /t {
        content_by_lua_block {
            local cel = require("cel")
            local program = cel.program
            local context = cel.context

            local p = program.new()
            local compiled, err = p:compile("age > 18")
            
            if not compiled then
                ngx.say("compilation failed: " .. err)
                return
            end

            local c = context.new()
            c:add_variable("age", 25)

            local result, err = p:execute(c)
            if err then
                ngx.say("execution failed: " .. err)
                return
            end

            ngx.say(result)
        }
    }
--- request
GET /t
--- response_body
true
--- no_error_log
[error]
[warn]
[crit]



=== TEST 4: validation
--- http_config eval: $::HttpConfig
--- config
    location = /t {
        content_by_lua_block {
            local program = require("resty.cel.program")

            local result, err = program.validate("name == 'test' && age > 21")
            
            if not result then
                ngx.say("validation failed: " .. err)
                return
            end

            ngx.say("variable count: " .. result.variable_count)
        }
    }
--- request
GET /t
--- response_body
variable count: 2
--- no_error_log
[error]
[warn]
[crit]
