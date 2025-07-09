# Tiltfile for selfie-v4 multi-distribution testing

# Built-in docker-compose support (no extension needed)

# Define test distributions with their build contexts
distributions = [
    'debian',
    'alpine'
]

# Docker Compose configuration
docker_compose('./docker-compose.yml')

# Build Docker images for each distribution
for distro in distributions:
    docker_build(
        'selfie-v4_' + distro,
        context='./docker/' + distro,
        dockerfile='./docker/' + distro + '/Dockerfile'
    )

# Create Tilt resources for each distribution
for distro in distributions:
    # Configure docker-compose service with manual trigger
    dc_resource(
        distro,
        resource_deps=['selfie-v4_' + distro],
        trigger_mode=TRIGGER_MODE_MANUAL
    )

    # Create a local resource to run tests in each container
    local_resource(
        distro + '-tests',
        cmd='docker-compose exec -T ' + distro + ' bash -c "cd /workspace && cargo test --all"',
        deps=['./crates', './Cargo.toml', './Cargo.lock'],
        resource_deps=[distro],
        auto_init=False,
        trigger_mode=TRIGGER_MODE_MANUAL
    )

# Create a convenience resource to run tests on all distributions
local_resource(
    'test-all',
    cmd='echo "Running tests on all distributions..." && ' + ' && '.join([
        'tilt trigger ' + distro + '-tests' for distro in distributions
    ]),
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL
)

# Print instructions
print("📦 Selfie Multi-Distribution Testing Setup")
print("==========================================")
print("")
print("🐳 Building Docker images for distributions:")
for distro in distributions:
    print("  • " + distro + " (selfie-v4_" + distro + ")")
print("")
print("Available distributions:")
for distro in distributions:
    print("  • " + distro)
print("")
print("Available commands:")
print("  • tilt trigger test-all       - Run tests on all distributions")
print("")
print("Per-distribution commands:")
for distro in distributions:
    print("  • tilt trigger " + distro + "-tests   - Run tests on " + distro)
print("")
print("To get a shell in a container:")
for distro in distributions:
    print("  • docker-compose exec " + distro + " bash")
print("")
print("All resources are set to manual trigger mode to avoid overwhelming your system.")
print("Use 'tilt trigger <resource-name>' to run specific tasks.")
print("")
print("💡 Images are built automatically when Dockerfiles change.")
print("🔄 Use 'tilt up' to start and 'tilt down' to stop the environment.")
