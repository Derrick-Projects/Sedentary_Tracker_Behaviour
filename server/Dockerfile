# syntax=docker/dockerfile:1
FROM rust:1.70 as builder
WORKDIR /workspace
# Copy the entire project (all crates)
COPY ../../ .
RUN apt-get update && apt-get install -y libudev-dev pkg-config
WORKDIR /workspace/server
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libudev-dev
COPY --from=builder /workspace/server/target/release/server /app/server
COPY --from=builder /workspace/.env /app/.env
EXPOSE 8000
CMD ["/app/server"]
