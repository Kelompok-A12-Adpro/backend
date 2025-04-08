FROM docker.io/rust:1-slim-bookworm AS build

ARG pkg=gatherlove-be

WORKDIR /build

COPY . .

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Build the application
RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    set -eux; \
    cargo build --release; \
    cp target/release/backend ./main

FROM docker.io/debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /build/main ./

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=80

EXPOSE 80

CMD ["./main"]