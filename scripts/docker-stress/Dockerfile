FROM rust:1.86-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build -p wm-server --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y curl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/wm-server /usr/local/bin/
COPY --from=builder /app/.wm /root/.wm
EXPOSE 4090
ENTRYPOINT ["wm-server"]
