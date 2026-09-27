FROM rust:1.75-slim AS builder
WORKDIR /app
RUN apt-get update \
  && apt-get install -y --no-install-recommends gcc pkg-config libc6-dev ca-certificates \
  && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust-test-app /app/rust-test-app
EXPOSE 8080
CMD ["./rust-test-app"]
