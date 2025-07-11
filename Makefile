VERSION ?= 0.1.0
REVISION ?= 0

ifeq ($(OS), Darwin)
  SHLIB_EXT = dylib
else
  SHLIB_EXT = so
endif

LIB_FILES = $(shell find lib -type f -name '*.lua')

ROCKSPEC_DEV_FILE := cel-dev-0.rockspec
ROCKSPEC_RELEASE_FILE := cel-$(VERSION)-$(REVISION).rockspec
ROCK_RELEASE_FILE := cel-$(VERSION)-$(REVISION).all.rock

TEST_RESULTS_PATH := test-results

# Overwrite if you want to use `docker` with sudo
DOCKER ?= docker

_docker_is_podman = $(shell $(DOCKER) --version | grep podman 2>/dev/null)

# Set default run flags:
# - remove container on exit
# - set username/UID to executor
DOCKER_USER ?= $$(id -u)
DOCKER_USER_OPT = $(if $(_docker_is_podman),--userns keep-id,--user $(DOCKER_USER))
DOCKER_RUN_ADDITIONAL_FLAGS ?=
DOCKER_RUN_FLAGS ?= --rm --interactive $(DOCKER_RUN_ADDITIONAL_FLAGS) $(DOCKER_USER_OPT)

MOUNT_PATH_IN_CONTAINER := /workspace

DOCKER_NO_CACHE :=

BUILDKIT_PROGRESS :=

# Busted runtime profile
BUSTED_RUN_PROFILE := default
BUSTED_FILTER :=

# Busted exclude tags
BUSTED_EXCLUDE_TAGS :=
BUSTED_NO_KEEP_GOING := false
BUSTED_COVERAGE := false
BUSTED_EMMY_DEBUGGER := false

BUSTED_EMMY_DEBUGGER_ENABLED_ARGS =

BUSTED_ARGS = --config-file $(MOUNT_PATH_IN_CONTAINER)/.busted --run '$(BUSTED_RUN_PROFILE)' --exclude-tags='$(BUSTED_EXCLUDE_TAGS)' --filter '$(BUSTED_FILTER)'
ifneq ($(BUSTED_NO_KEEP_GOING), false)
	BUSTED_ARGS += --no-keep-going
endif

ifneq ($(BUSTED_COVERAGE), false)
	BUSTED_ARGS += --coverage
endif

ifneq ($(BUSTED_EMMY_DEBUGGER), false)
	BUSTED_EMMY_DEBUGGER_ENABLED_ARGS = -e BUSTED_EMMY_DEBUGGER='/usr/local/lib/lua/5.1/emmy_core.so'
endif

CONTAINER_CI_TOOLING_OPENRESTY_VERSION ?=
CONTAINER_CI_TOOLING_IMAGE_PATH := hack/tooling
CONTAINER_CI_TOOLING_IMAGE_TAG ?= 0.1.0
CONTAINER_CI_TOOLING_IMAGE_NAME := localhost/cel-lua-test-tooling:$(CONTAINER_CI_TOOLING_IMAGE_TAG)
CONTAINER_CI_TOOLING_IMAGE_METADATA_FILE := $(CONTAINER_CI_TOOLING_IMAGE_PATH)/docker.build.metadata.json
CONTAINER_CI_TOOLING_BUILD_ADDITIONAL_FLAGS ?=

CONTAINER_CI_TOOLING_BUILD ?= DOCKER_BUILDKIT=1 BUILDKIT_PROGRESS=$(BUILDKIT_PROGRESS) $(DOCKER) build \
	$(if $(CONTAINER_CI_TOOLING_OPENRESTY_VERSION),--build-arg OPENRESTY_VERSION='$(CONTAINER_CI_TOOLING_OPENRESTY_VERSION)',) \
	-f '$(CONTAINER_CI_TOOLING_IMAGE_PATH)/Dockerfile' \
	--metadata-file '$(CONTAINER_CI_TOOLING_IMAGE_METADATA_FILE)' \
	$(DOCKER_NO_CACHE) \
	$(CONTAINER_CI_TOOLING_BUILD_ADDITIONAL_FLAGS) \
	-t '$(CONTAINER_CI_TOOLING_IMAGE_NAME)' \
	.

# TODO:
CONTAINER_CI_TOOLING_RUN ?= MSYS_NO_PATHCONV=1 $(DOCKER) run $(DOCKER_RUN_FLAGS) \
	-e BUSTED_EMMY_DEBUGGER_HOST='0.0.0.0' \
	-e BUSTED_EMMY_DEBUGGER_PORT='9966' \
	-e BUSTED_EMMY_DEBUGGER_SOURCE_PATH='/usr/local/share/lua/5.1/kong/plugins:/usr/local/share/lua/5.1/kong/enterprise_edition' \
	-e BUSTED_EMMY_DEBUGGER_SOURCE_PATH_MAPPING='$(MOUNT_PATH_IN_CONTAINER);$(PWD):/usr/local/share/lua/5.1;$(PWD)/.luarocks:/usr/local/openresty/lualib;$(PWD)/.luarocks' \
	$(BUSTED_EMMY_DEBUGGER_ENABLED_ARGS) \
	-v '$(PWD):$(MOUNT_PATH_IN_CONTAINER)' \
	-v '$(PWD)/_build/debugger/emmy_debugger.lua:/usr/local/share/lua/5.1/kong/tools/emmy_debugger.lua' \
	-v '$(PWD)/_build/debugger/busted:/kong/bin/busted' \
	'$(CONTAINER_CI_TOOLING_IMAGE_NAME)'

RELEASE_FOLDER = target/release
DEBUG_RELEASE_FOLDER = target/debug

RM := rm
RMDIR := $(RM) -rf

TAG ?=

.PHONY: all
all: lint format test

$(ROCKSPEC_DEV_FILE): cel.rockspec
	cp cel.rockspec $(ROCKSPEC_DEV_FILE)
	$(CONTAINER_CI_RUN) luarocks new_version $(ROCKSPEC_DEV_FILE) --tag=dev-0 --dir .

$(ROCKSPEC_RELEASE_FILE): $(ROCKSPEC_DEV_FILE)
	$(CONTAINER_CI_RUN) luarocks new_version $(ROCKSPEC_DEV_FILE) --tag=v$(VERSION)-$(REVISION) --dir .

.PHONY: release-rockspec
release-rockspec: $(ROCKSPEC_RELEASE_FILE)

.PHONY: release-rockspec
release-info:
	@echo "VERSION=v$(VERSION)-$(REVISION)"
	@echo "ROCKSPEC_RELEASE_FILE=$(ROCKSPEC_RELEASE_FILE)"

# Rebuild the rock file every time the rockspec or the lib/**.lua files change
$(ROCK_RELEASE_FILE): container-ci-tooling $(ROCKSPEC_RELEASE_FILE) $(LIB_FILES)
	$(CONTAINER_CI_TOOLING_RUN) luarocks make --pack-binary-rock --deps-mode none $(ROCKSPEC_RELEASE_FILE)

test-results:
	mkdir -p $(TEST_RESULTS_PATH)

.PHONY: pack
pack: $(ROCK_RELEASE_FILE)

# If the image metadata file is not present, build the image
$(CONTAINER_CI_TOOLING_IMAGE_METADATA_FILE): hack/tooling/test-dependencies-0.1.0-0.rockspec hack/tooling/Dockerfile
	$(CONTAINER_CI_TOOLING_BUILD)

.PHONY: container-ci-tooling
container-ci-tooling: $(CONTAINER_CI_TOOLING_IMAGE_METADATA_FILE)

.PHONY: container-ci-tooling-debug
container-ci-tooling-debug: BUILDKIT_PROGRESS = 'plain'
container-ci-tooling-debug: DOCKER_NO_CACHE = '--no-cache'
container-ci-tooling-debug: container-ci-tooling

.PHONY: lint
lint: lint-lua lint-rust

.PHONY: lint-lua
lint-lua: container-ci-tooling
	$(CONTAINER_CI_TOOLING_RUN) luacheck --no-default-config --config .luacheckrc .

.PHONY: lint-rust
lint-rust: container-ci-tooling
	$(CONTAINER_CI_TOOLING_RUN) sh -c "id; pwd; echo $$HOME; cargo clippy --version"
	$(CONTAINER_CI_TOOLING_RUN) cargo clippy --all-targets --all-features -- -D warnings

.PHONY: fmt
fmt: format

.PHONY: format
format: format-lua format-rust

.PHONY: format-lua
format-lua: container-ci-tooling
	$(CONTAINER_CI_TOOLING_RUN) stylua .

.PHONY: format-rust
format-rust: container-ci-tooling
	$(CONTAINER_CI_TOOLING_RUN) cargo fmt

.PHONY: test-unit
test-unit: clean-test-results test-results container-ci-tooling
	$(CONTAINER_CI_TOOLING_RUN) busted $(BUSTED_ARGS)
	@if [ -f $(TEST_RESULTS_PATH)/luacov.stats.out ]; then \
		$(CONTAINER_CI_TOOLING_RUN) luacov-console $(MOUNT_PATH_IN_CONTAINER)/kong; luacov-console -s ;\
		$(CONTAINER_CI_TOOLING_RUN) luacov -r html; mv luacov.report.out luacov.report.html ;\
		echo "Coverage report: file://$(PWD)/$(TEST_RESULTS_PATH)/luacov.report.html" ;\
		$(CONTAINER_CI_TOOLING_RUN) sh -c "(cd $(MOUNT_PATH_IN_CONTAINER)/$(TEST_RESULTS_PATH); luacov -r lcov; sed -e 's|/kong-plugin/||' -e 's/^\(DA:[0-9]\+,[0-9]\+\),[^,]*/\1/' luacov.report.out > lcov.info)" ;\
	fi

# .PHONY: test-lua-busted
# test-lua-busted: container-ci-tooling
# 	$(CONTAINER_CI_TOOLING_RUN) ./hack/tooling/busted-luajit $(BUSTED_ARGS)
# @if [ -f $(TEST_RESULTS_PATH)/luacov.stats.out ]; then \
# 	$(CONTAINER_CI_TOOLING_RUN) luacov-console $(MOUNT_PATH_IN_CONTAINER)/kong; luacov-console -s ;\
# 	$(CONTAINER_CI_TOOLING_RUN) luacov -r html; mv luacov.report.out luacov.report.html ;\
# 	echo "Coverage report: file://$(PWD)/$(TEST_RESULTS_PATH)/luacov.report.html" ;\
# 	$(CONTAINER_CI_TOOLING_RUN) sh -c "(cd $(MOUNT_PATH_IN_CONTAINER)/$(TEST_RESULTS_PATH); luacov -r lcov; sed -e 's|/kong-plugin/||' -e 's/^\(DA:[0-9]\+,[0-9]\+\),[^,]*/\1/' luacov.report.out > lcov.info)" ;\
# fi

.PHONY: test
test: test-lua test-rust

.PHONY: test-lua
test-lua: test-busted-luajit test-busted-resty

.PHONY: test-rust
test-rust: test-rust-memory-valgrind

.PHONY: test-rust-memory-valgrind
test-rust-memory-valgrind: DOCKER_RUN_ADDITIONAL_FLAGS=--tty -e TERM=xterm-256color
test-rust-memory-valgrind: container-ci-tooling
	$(CONTAINER_CI_TOOLING_RUN) cargo valgrind test

.PHONY: test-busted-luajit
test-busted-luajit: DOCKER_RUN_ADDITIONAL_FLAGS=--tty -e TERM=xterm-256color
test-busted-luajit: build container-ci-tooling
	echo "Running busted-luajit tests..."
	echo DOCKER_USE_TTY=$(DOCKER_USE_TTY)
	test -t 1 && echo "--tty" || echo ""
	$(CONTAINER_CI_TOOLING_RUN) ./hack/tooling/busted-luajit $(BUSTED_ARGS)

.PHONY: test-busted-resty
test-busted-resty: DOCKER_RUN_ADDITIONAL_FLAGS=--tty -e TERM=xterm-256color
test-busted-resty: build container-ci-tooling
	$(CONTAINER_CI_TOOLING_RUN) ./hack/tooling/busted-resty $(BUSTED_ARGS)

# Rust build targets
.PHONY: build
build: $(RELEASE_FOLDER)/libcel_lua.$(SHLIB_EXT) $(RELEASE_FOLDER)/libcel_lua.a

$(RELEASE_FOLDER)/libcel_lua.%: src/**/*.rs src/*.rs
	cargo build --release

$(DEBUG_RELEASE_FOLDER)/libcel_lua.%: src/**/*.rs src/*.rs
	cargo build

.PHONY: lua-language-server-add-kong
lua-language-server-add-kong: container-ci-tooling
	-mkdir -p .luarocks
	$(CONTAINER_CI_TOOLING_RUN) cp -rv /usr/local/share/lua/5.1/. $(MOUNT_PATH_IN_CONTAINER)/.luarocks
	$(CONTAINER_CI_TOOLING_RUN) cp -rv /usr/local/openresty/lualib/. $(MOUNT_PATH_IN_CONTAINER)/.luarocks

.PHONY: tooling-shell
tooling-shell: DOCKER_RUN_ADDITIONAL_FLAGS=--tty
tooling-shell: container-ci-tooling
	$(CONTAINER_CI_TOOLING_RUN) bash

.PHONY: tooling-shell-root
tooling-shell-root: DOCKER_USER=0
tooling-shell-root: tooling-shell
	$(CONTAINER_CI_TOOLING_RUN) bash

.PHONY: pre-commit
pre-commit:
	pre-commit run --all-files

.PHONY: clean-test-results
clean-test-results:
	-$(RMDIR) test-results

.PHONY: clean-rockspec
clean-rockspec:
	-$(RMDIR) cel-*.rockspec

.PHONY: clean-rock
clean-rock:
	-$(RMDIR) *.rock

.PHONY: clean-target
clean-target:
	-$(RMDIR) target

.PHONY: clean-container-ci-tooling
clean-container-ci-tooling:
	-$(DOCKER) rmi '$(CONTAINER_CI_TOOLING_IMAGE_NAME)'
	-$(RM) $(CONTAINER_CI_TOOLING_IMAGE_METADATA_FILE)

.PHONY: clean
clean: clean-test-results
clean: clean-rock clean-rockspec
clean: clean-target
clean: clean-container-ci-tooling
	-$(RMDIR) .luarocks
