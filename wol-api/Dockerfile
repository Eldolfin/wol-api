FROM rust:1 AS chef 
RUN cargo install cargo-chef 
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR app
COPY --from=builder /app/target/release/wol-relay-server /usr/local/bin
ENTRYPOINT ["/usr/local/bin/wol-relay-server", "-c", "/app/config.yml"]
