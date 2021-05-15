# Adapted from https://github.com/LukeMathWalker/zero-to-production/blob/main/Dockerfile
FROM lukemathwalker/cargo-chef as planner
WORKDIR /app
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM lukemathwalker/cargo-chef as cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.52.1 AS builder
WORKDIR /app
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
# Build our application, leveraging the cached deps!
RUN cargo build --release --bin pokespeare

FROM debian:buster-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pokespeare pokespeare
COPY config.yml config.yml
ENV HOST=0.0.0.0
ENV APP_PORT=5000

ENTRYPOINT ["./pokespeare"]
