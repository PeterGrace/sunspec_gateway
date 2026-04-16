# Stage 1: Build the UI
FROM node:20 as ui-builder
WORKDIR /app
COPY ui-src/package.json ui-src/package-lock.json ./
RUN npm ci
COPY ui-src/ .
RUN npm run build

# Stage 2: Build the Rust backend
FROM rust:1.83 as rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Copy migrations early as they might be needed for sqlx macros or build scripts
COPY migrations ./migrations
# Create a dummy main.rs to build dependencies first (caching optimization)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Copy the actual source code
COPY src ./src
# Touch main.rs to force rebuild
RUN touch src/main.rs
RUN cargo build --release

# Stage 3: Runtime image
FROM ubuntu:22.04
ARG TARGETARCH

RUN mkdir -p /opt/sunspec_gateway/ui \
    && mkdir -p /opt/sunspec_gateway/models \
    && apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/sunspec_gateway

# Copy compiled artifacts
COPY --from=rust-builder /app/target/release/sunspec_gateway .
COPY --from=ui-builder /app/dist/ ./ui/

# Copy static assets and config
COPY models/ ./models/
# Copy all tools (including stats.so variants) and the selection script
COPY tools/ ./tools/

# Select the correct stats.so based on architecture
RUN if [ "$TARGETARCH" = "amd64" ]; then \
    cp tools/stats.so.amd64 stats.so; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
    cp tools/stats.so.arm64 stats.so; \
    else \
    echo "Error: Unsupported architecture $TARGETARCH"; \
    exit 1; \
    fi

EXPOSE 8300
CMD ["/opt/sunspec_gateway/sunspec_gateway"]
