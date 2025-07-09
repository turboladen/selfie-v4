# Docker commands module

# Show available docker commands
default:
    @echo "Available docker commands:"
    @echo "  start          Start all containers"
    @echo "  stop           Stop all containers"
    @echo "  status         Show container status"
    @echo "  clean          Clean up containers and volumes"
    @echo "  up             Docker compose up -d"
    @echo "  down           Docker compose down"
    @echo "  logs           Show container logs"
    @echo "  build-images   Build all images"
    @echo "  rebuild-images Rebuild all images from scratch"
    @echo "  shell-debian   Open shell in Debian container"
    @echo "  shell-alpine   Open shell in Alpine container"
    @echo "  update-rust    Update Rust in containers"
    @echo "  debug-rust     Check Rust installations"

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

# Clean up all containers and volumes
clean:
    @echo "🧹 Cleaning up..."
    @docker-compose down -v
    @docker system prune -f

# Docker compose up -d
up:
    @docker-compose up -d

# Docker compose down
down:
    @docker-compose down

# Show container logs
logs:
    @docker-compose logs -f

# Build all Docker images
build-images:
    @echo "🔨 Building all Docker images..."
    @docker-compose build

# Rebuild all Docker images from scratch
rebuild-images:
    @echo "🔨 Rebuilding all Docker images from scratch..."
    @docker-compose build --no-cache

# Open shell in Debian container
shell-debian:
    @echo "🐚 Opening shell in Debian container..."
    @docker-compose exec debian bash || docker-compose exec -T debian bash

# Open shell in Alpine container
shell-alpine:
    @echo "🐚 Opening shell in Alpine container..."
    @docker-compose exec alpine bash || docker-compose exec -T alpine bash

# Update Rust toolchain in all containers
update-rust:
    @echo "🦀 Updating Rust toolchain in all containers..."
    @docker-compose exec -T debian rustup update || true
    @docker-compose exec -T alpine rustup update || true

# Check Rust installations in containers
debug-rust:
    @echo "🦀 Checking Rust installations..."
    @echo "=== debian ==="
    @docker-compose exec -T debian bash -c "rustc --version && cargo --version" || echo "Rust not available in debian"
    @echo "=== alpine ==="
    @docker-compose exec -T alpine bash -c "rustc --version && cargo --version" || echo "Rust not available in alpine"
