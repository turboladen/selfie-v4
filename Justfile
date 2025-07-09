# Justfile for Selfie Multi-Distribution Testing
# =============================================

# Variables
distributions := "debian alpine"

# Docker Compose operations
mod docker 'justfiles/docker.just'
# Tilt operations (requires Tilt running)
mod tilt 'justfiles/tilt.just'

# Default recipe (shows help)
default:
    @just --list

# Show this help message
help:
    @echo "🦀 Selfie Multi-Distribution Testing"
    @echo "===================================="
    @echo ""
    @just --list
    @echo ""
    @echo "📋 Command Categories:"
    @echo "  🔧 standalone    - Run without Tilt (independent)"
    @echo "  💻 local         - Run locally (not in containers)"
    @echo ""
    @echo "💡 For Tilt management, use 'tilt up', 'tilt down' directly"
    @echo "Available distributions: {{distributions}}"





# Build locally (not in containers)
[group('local')]
build:
    @echo "🔨 Building locally..."
    @cargo build --all

# Format code using dprint and cargo
[group('local')]
fmt:
    @echo "📝 Formatting code..."
    @dprint fmt
    @cargo fmt

# Private Tilt helper (used by other recipes)
_tilt-up:
    @tilt up







# Local development (not in containers)
[group('local')]
local-test:
    @echo "🧪 Running tests locally..."
    @cargo test --all --color=always

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
[group('local')]
docs:
    @echo "📚 Generating documentation..."
    @cargo doc --open



[group('local')]
check-deps:
    @echo "📦 Checking for outdated dependencies..."
    @cargo outdated

# CI/CD helpers (standalone - manages own containers)
[group('standalone')]
ci-test:
    @echo "🤖 Running CI test suite..."
    @just docker::start
    @echo "⏳ Waiting for containers to be ready..."
    @sleep 5
    @echo "🧪 Running tests on Debian..."
    @docker-compose exec -T debian bash -c "cd /workspace && cargo test --all --color=always"
    @echo "🧪 Running tests on Alpine..."
    @docker-compose exec -T alpine bash -c "cd /workspace && cargo test --all --color=always"
    @just docker::stop

# Debug helpers
[group('standalone')]
debug-containers:
    @echo "🐳 Container information:"
    @docker-compose ps
    @echo ""
    @echo "📊 Container resource usage:"
    @docker stats --no-stream
    @echo ""
    @echo "💾 Docker system info:"
    @docker system df



[group('standalone')]
debug-images:
    @echo "🐳 Docker images:"
    @docker images | grep selfie

# Test specific functionality locally
[group('local')]
test-streaming:
    @echo "🧪 Running streaming tests specifically..."
    @cargo test test_command_streaming --color=always

[group('local')]
test-integration:
    @echo "🧪 Running integration tests..."
    @cargo test --test '*' --color=always

# Quick shortcuts
alias t := tilt::test
alias c := local-clippy
alias b := build
alias s := docker::status
