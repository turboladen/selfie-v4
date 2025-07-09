FROM debian:bookworm-slim

# Set environment variables
ENV CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_BACKTRACE=1 \
    DEBIAN_FRONTEND=noninteractive

# Install essential packages, Rust, and create user in a single layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    ca-certificates \
    bash \
    procps \
    file \
    sudo \
    && mkdir -p /usr/local/cargo /usr/local/rustup \
    && chmod 755 /usr/local/cargo /usr/local/rustup \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain stable --no-modify-path \
    && chmod -R a+w /usr/local/cargo /usr/local/rustup \
    && /usr/local/cargo/bin/rustup component add clippy rustfmt \
    && useradd -m -s /bin/bash developer \
    && usermod -aG sudo developer \
    && echo "developer ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers \
    && echo 'export PATH="/usr/local/cargo/bin:$PATH"' >> /home/developer/.bashrc \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Set working directory
WORKDIR /workspace

# Switch to developer user
USER developer

# Verify Rust is available
RUN rustc --version && cargo --version

CMD ["sleep", "infinity"]
