

# Lint with clippy
lint:
    bazel run //crates/webapp:webapp_clippy

# Run the application locally
run:
    bazel run //crates/webapp:webapp

# Test with curl (don’t forget to start the server first !) 
test:
    curl -v -X POST -H "Content-Type: application/json" -d '{"username":"jlc","password":"pass"}' http://localhost:8080/user

# Build webapp binary
build:
    bazel build //crates/webapp:webapp

# Build docker image and push to local registry
docker-build:
    bazel run //oci:rust_app_server_image_tarball

# Build docker image and push to local registry
docker-debug-build:
    bazel run //oci:rust_app_server_debug_image_tarball

# Update third party dependencies
update-deps:
    cd third-party && rm Cargo.lock && cargo generate-lockfile && cd ..
    bazel run //third-party:vendor

# Push Image to ghcr.io (requires login)
docker-push:
    docker tag rust-devops-azure-sample:latest ghcr.io/jlcanela/rust-devops-azure-sample:0.0.1
    docker push ghcr.io/jlcanela/rust-devops-azure-sample:0.0.1

# Start docker image with shell
docker-run-debug: docker-debug-build
    docker run -it --env-file .env --entrypoint=/busybox/sh -p 8080:8080 rust_app_server_debug:latest

# Run webapp with docker
docker-run: docker-build
    docker run -it --env-file .env --entrypoint=/webapp -p 8080:8080 rust_app_server:latest

# Verify the policies
policies:
    cd cedar-policies && ./run.sh && cd ..

# Scan vulnerabilities with docker scout
security:
    docker scout cves local://rust-devops-azure-sample:latest
