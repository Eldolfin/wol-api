FROM rust:1 AS chef 
RUN cargo install cargo-chef 
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
COPY ./rust-toolchain.toml .
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR app
RUN mkdir -p /root/.ssh
RUN apt-get update \
  && apt-get install -y curl ssh \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/backend /usr/local/bin
ENTRYPOINT ["/usr/local/bin/backend", "-c", "/app/config.yml"]
