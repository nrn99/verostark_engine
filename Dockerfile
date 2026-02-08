# Build Stage
FROM rust:bookworm AS builder

WORKDIR /usr/src/verostark_engine

# Install build dependencies (if any needed for mlua/pingora C deps)
# mlua vendored feature handles lua. pingora might need cmake/clang depending on features (e.g. boringssl)
RUN apt-get update && apt-get install -y pkg-config libssl-dev cmake clang && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Build app
COPY . .
RUN touch src/main.rs
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY --from=builder /usr/src/verostark_engine/target/release/verostark_engine /app/verostark_engine

# Copy policies for hot-reloading
COPY policies /app/policies

# Expose port
EXPOSE 8000

# Run
CMD ["./verostark_engine"]
