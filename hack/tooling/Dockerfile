FROM registry.hub.docker.com/openresty/openresty:1.27.1.2-2-noble

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

COPY hack/tooling/test-dependencies-0.1.0-0.rockspec /test-dependencies-0.1.0-0.rockspec

ENV RUSTUP_HOME="/rust/rustup"

RUN curl https://sh.rustup.rs -sSf | CARGO_HOME=/rust/cargo sh -s -- -y \
  && apt-get update \
  && apt-get install --no-install-recommends -y git=1:2.* cpanminus=1.* \
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
  # Install LuaRocks and dependencies
  && luarocks build /test-dependencies-0.1.0-0.rockspec --only-deps \
  # && luarocks build /test-dependencies-0.1.0-0.rockspec --only-deps OPENSSL_DIR=/usr/local/kong CRYPTO_DIR=/usr/local/kong \
  # Clean up apt cache
  && rm -rf /var/lib/apt/lists/*

# COPY hack/tooling/busted /usr/local/bin/busted

ENV PATH="/rust/cargo/bin:${PATH}"
