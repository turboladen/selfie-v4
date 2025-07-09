# Justfile for Selfie Multi-Distribution Testing
# =============================================

# Variables
distributions := "debian alpine"

# Default recipe (shows help)
default:
    @just --list

# Show this help message
help:
    @echo "🦀 Selfie Multi-Distribution Testing"
    @echo "===================================="
    @echo ""
    @echo "Available recipes:"
    @just --list
    @echo ""
    @echo "Distribution-specific recipes:"
    @echo "  test-<distro>    Run tests on specific distribution"
    @echo "  shell-<distro>   Open shell in specific container"
    @echo ""
    @echo "Available distributions: {{distributions}}"

# Start Tilt to verify setup
verify:
    @echo "🔍 Starting Tilt to verify setup..."
    @echo "Open http://localhost:10350 to view status"
    @tilt up

# Start all containers
start:
    @echo "🚀 Starting all containers..."
    @docker-compose up -d

# Stop all containers
stop:
    @echo "🛑 Stopping all containers..."
    @docker-compose down

# Show status of all containers
status:
    @echo "📊 Container status:"
    @docker-compose ps

# Run tests on all distributions via Tilt
test:
    @echo "🧪 Running tests on all distributions..."
    @tilt trigger test-all

# Build locally (not in containers)
build:
    @echo "🔨 Building locally..."
    @cargo build --all

# Clean up all containers and volumes
clean:
    @echo "🧹 Cleaning up..."
    @docker-compose down -v
    @docker system prune -f

# Tilt commands
[group('tilt')]
tilt-up:
    @echo "🚀 Starting Tilt..."
    @tilt up

[group('tilt')]
tilt-down:
    @echo "🛑 Stopping Tilt..."
    @tilt down

[group('tilt')]
tilt-ci:
    @echo "🤖 Running Tilt in CI mode..."
    @tilt up --stream --hud=false

# Distribution-specific recipes
[group('debian')]
test-debian:
    @echo "🧪 Running tests on Debian..."
    @tilt trigger debian-tests

[group('debian')]
shell-debian:
    @echo "🐚 Opening shell in Debian container..."
    @docker-compose exec debian bash || docker-compose exec -T debian bash



[group('alpine')]
test-alpine:
    @echo "🧪 Running tests on Alpine..."
    @tilt trigger alpine-tests

[group('alpine')]
shell-alpine:
    @echo "🐚 Opening shell in Alpine container..."
    @docker-compose exec alpine bash || docker-compose exec -T alpine bash



# Quick development workflow
[group('dev')]
dev: start
    @echo "🚀 Starting development environment..."
    @sleep 2
    @tilt up

[group('dev')]
dev-test:
    @echo "🧪 Running quick test cycle..."
    @just test

# Docker Compose shortcuts
[group('docker')]
up:
    @docker-compose up -d

[group('docker')]
down:
    @docker-compose down

[group('docker')]
logs:
    @docker-compose logs -f

[group('docker')]
build-images:
    @echo "🔨 Building all Docker images..."
    @docker-compose build

[group('docker')]
rebuild-images:
    @echo "🔨 Rebuilding all Docker images from scratch..."
    @docker-compose build --no-cache

# Local development (not in containers)
[group('local')]
local-test:
    @echo "🧪 Running tests locally..."
    @cargo test --all

[group('local')]
local-clippy:
    @echo "📎 Running clippy locally..."
    @cargo clippy --all-targets -- -D warnings

[group('local')]
local-build:
    @echo "🔨 Building locally..."
    @cargo build --all

[group('local')]
local-check:
    @echo "🔍 Running cargo check locally..."
    @cargo check --all

# Documentation
[group('docs')]
docs:
    @echo "📚 Generating documentation..."
    @cargo doc --open

# Maintenance recipes
[group('maintenance')]
update-rust:
    @echo "🦀 Updating Rust toolchain in all containers..."
    @docker-compose exec -T debian rustup update || true
    @docker-compose exec -T alpine rustup update || true

[group('maintenance')]
check-deps:
    @echo "📦 Checking for outdated dependencies..."
    @cargo outdated

# CI/CD helpers
[group('ci')]
ci-test: tilt-ci
    @echo "🤖 Running CI test suite..."
    @tilt trigger test-all
    @tilt down

# Installation helpers
[group('install')]
install-deps:
    @echo "📦 Installing dependencies..."
    @echo "Please install the following tools:"
    @echo "  • Docker: https://docs.docker.com/get-docker/"
    @echo "  • Docker Compose: https://docs.docker.com/compose/install/"
    @echo "  • Tilt: https://docs.tilt.dev/install.html"
    @echo "  • Just: https://github.com/casey/just#installation"

[group('install')]
install-deps-brew:
    @echo "🍺 Installing dependencies via Homebrew..."
    @command -v brew >/dev/null 2>&1 || { echo "Homebrew not found. Please install it first."; exit 1; }
    @brew install docker docker-compose tilt-dev/tap/tilt just
    @echo "✅ Dependencies installed. Make sure Docker Desktop is running."

# Debug helpers
[group('debug')]
debug-containers:
    @echo "🐳 Container information:"
    @docker-compose ps
    @echo ""
    @echo "📊 Container resource usage:"
    @docker stats --no-stream
    @echo ""
    @echo "💾 Docker system info:"
    @docker system df

[group('debug')]
debug-rust:
    @echo "🦀 Checking Rust installations..."
    @echo "=== debian ==="
    @docker-compose exec -T debian bash -c "rustc --version && cargo --version" || echo "Rust not available in debian"
    @echo "=== alpine ==="
    @docker-compose exec -T alpine bash -c "rustc --version && cargo --version" || echo "Rust not available in alpine"

[group('debug')]
debug-images:
    @echo "🐳 Docker images:"
    @docker images | grep selfie

# Test specific functionality
[group('test')]
test-streaming:
    @echo "🧪 Running streaming tests specifically..."
    @cargo test test_command_streaming

[group('test')]
test-integration:
    @echo "🧪 Running integration tests..."
    @cargo test --test '*'

# Quick shortcuts
alias t := test
alias c := local-clippy
alias b := build
alias s := status
alias v := verify
