FROM alpine:latest

# Set environment variables
ENV CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_BACKTRACE=1

# Install essential packages, Rust, and create user in a single layer
RUN apk update && \
    apk add --no-cache \
    curl \
    build-base \
    pkgconfig \
    openssl-dev \
    git \
    ca-certificates \
    bash \
    procps \
    file \
    musl-dev \
    linux-headers \
    sudo \
    shadow \
    && mkdir -p /usr/local/cargo /usr/local/rustup \
    && chmod 755 /usr/local/cargo /usr/local/rustup \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain stable --no-modify-path \
    && chmod -R a+w /usr/local/cargo /usr/local/rustup \
    && /usr/local/cargo/bin/rustup component add clippy rustfmt \
    && adduser -D -s /bin/bash developer \
    && addgroup developer wheel \
    && echo "developer ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers \
    && echo 'export PATH="/usr/local/cargo/bin:$PATH"' >> /home/developer/.bashrc \
    && rm -rf /var/cache/apk/*

# Set working directory
WORKDIR /workspace

# Switch to developer user
USER developer

# Verify Rust is available
RUN rustc --version && cargo --version

CMD ["sleep", "infinity"]
