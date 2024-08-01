# Build stage
#FROM rust:1.77-alpine3.18 as builder
FROM rust:alpine as builder
LABEL org.opencontainers.image.source https://github.com/jlcanela/rust-devops-azure-sample
LABEL org.opencontainers.image.description="A sample Rust Web Application"
LABEL org.opencontainers.image.licenses=MIT

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    gcc \
    make \
    libc-dev \
    openssl-dev \
    clang-dev \
    llvm-dev

ENV CC=clang
ENV RUSTFLAGS="-C target-feature=-crt-static"

# Set the working directory
WORKDIR /usr/src/app

# Copy the entire project
COPY . .

# Build the application
RUN cargo build --release

#RUN rustup target add x86_64-unknown-linux-musl
#RUN cargo build --release --target x86_64-unknown-linux-musl

# Final stage
FROM alpine:3.18

# Install any runtime dependencies (if needed)
RUN apk add --no-cache libgcc

# Copy the binary from the build stage
COPY --from=builder /usr/src/app/target/release/rust-devops-azure-sample /usr/local/bin/

# Set the startup command
CMD ["rust-devops-azure-sample"]
