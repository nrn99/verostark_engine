# Build Stage
FROM rust:bookworm AS builder

WORKDIR /usr/src/verostark_engine

# Install build dependencies (if any needed for mlua/pingora C deps)
# mlua vendored feature handles lua. pingora might need cmake/clang depending on features (e.g. boringssl)
RUN apt-get update && apt-get install -y pkg-config libssl-dev cmake clang curl && rm -rf /var/lib/apt/lists/*

# Download Model (Project Cortex) - Cached Layer
# Moved before Cargo.toml so dependency changes don't trigger re-download
RUN mkdir -p model
# Config & Tokenizer (Standard w/ ONNX/Quantized compatibility)
RUN curl -L -o model/config.json https://huggingface.co/dslim/bert-base-NER/resolve/main/config.json
# Tokenizer (dslim/bert-base-NER lacks tokenizer.json, using base model)
RUN curl -L -o model/tokenizer.json https://huggingface.co/bert-base-cased/resolve/main/tokenizer.json
# Using standard F32 model (~400MB) for Tier 1 stability. 
# For Tier 2 (Quantized ~40MB), we need to quantize or find a specific pre-quantized file compatible with candle-transformers.
RUN curl -L -o model/model.safetensors https://huggingface.co/dslim/bert-base-NER/resolve/main/model.safetensors

# Cache dependencies
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Build app
COPY . .
RUN touch src/main.rs
RUN cargo build --release

# Quantum Compression (Phase 5)
# Convert F32 -> F16 (50% size reduction)
RUN ./target/release/quantize model/model.safetensors model/model.safetensors.f16
RUN rm model/model.safetensors
RUN mv model/model.safetensors.f16 model/model.safetensors


# Runtime Stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY --from=builder /usr/src/verostark_engine/target/release/verostark_engine /app/verostark_engine

# Copy policies for hot-reloading
COPY policies /app/policies

# Copy Cortex Model
COPY --from=builder /usr/src/verostark_engine/model /app/model

# Expose port
EXPOSE 8000

# Run
CMD ["./verostark_engine"]
