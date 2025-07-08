ARG OPENRESTY_VERSION="1.27.1.2-2-noble"
FROM registry.hub.docker.com/openresty/openresty:${OPENRESTY_VERSION}

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

COPY hack/tooling/test-dependencies-0.1.0-0.rockspec /test-dependencies-0.1.0-0.rockspec

ENV RUSTUP_HOME="/rust/rustup"
ENV PATH="/rust/cargo/bin:${PATH}"

RUN curl https://sh.rustup.rs -sSf | CARGO_HOME=/rust/cargo sh -s -- -y \
  && apt-get update \
  && apt-get install --no-install-recommends -y git=1:2.* cpanminus=1.* valgrind=1:3.* \
  && cargo install --root /rust/cargo cargo-valgrind \
  && cpanm --notest Test::Nginx \
  # Install stylua
  && curl -sSf -L "https://github.com/JohnnyMorganz/StyLua/releases/download/v2.1.0/stylua-linux-x86_64.zip" -o /tmp/stylua-linux-x86_64.zip \
  && unzip /tmp/stylua-linux-x86_64.zip -d /usr/local/bin \
  && rm -f /tmp/stylua-linux-x86_64.zip \
  # Download and install EmmyLuaDebugger
  && mkdir -p /usr/local/lib/lua/5.1 \
  && curl -sSf -L "https://github.com/EmmyLua/EmmyLuaDebugger/releases/download/1.8.6/linux-x64.zip" -o /tmp/emmylua-debugger.zip \
  && unzip /tmp/emmylua-debugger.zip -d /usr/local/lib/lua/5.1 \
  && rm -f /tmp/emmylua-debugger.zip \
  # Install LuaRocks
  && curl -sSf -L "https://luarocks.github.io/luarocks/releases/luarocks-3.12.2.tar.gz" -o /tmp/luarocks.tar.gz \
  && tar -xzf /tmp/luarocks.tar.gz -C /tmp \
  && cd /tmp/luarocks-3.12.2 \
  && ./configure --with-lua=/usr/local/openresty/luajit \
  && make \
  && make install \
  && cd - \
  && rm -rf /tmp/luarocks.tar.gz /tmp/luarocks-3.12.2 \
  # Install LuaRocks and dependencies
  && luarocks build /test-dependencies-0.1.0-0.rockspec --only-deps \
  # Clean up apt cache
  && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# COPY hack/tooling/busted /usr/local/bin/busted
