# Tilt commands module (requires Tilt to be running)

# Show available tilt commands
default:
    @echo "Available tilt commands (requires Tilt to be running):"
    @echo "  test         Run tests on all distributions"
    @echo "  test-debian  Run tests on Debian only"
    @echo "  test-alpine  Run tests on Alpine only"

# Run tests on all distributions via Tilt
test:
    @echo "🧪 Running tests on all distributions..."
    @tilt trigger test-all

# Run tests on Debian distribution
test-debian:
    @echo "🧪 Running tests on Debian..."
    @tilt trigger debian-tests

# Run tests on Alpine distribution
test-alpine:
    @echo "🧪 Running tests on Alpine..."
    @tilt trigger alpine-tests
