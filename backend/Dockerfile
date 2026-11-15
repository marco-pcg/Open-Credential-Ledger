FROM rust:1.96-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
RUN cargo build --locked --release -p ocl-api
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/ocl-api /usr/local/bin/ocl-api
USER 65532:65532
ENV BIND_ADDRESS=0.0.0.0:8080
EXPOSE 8080
CMD ["ocl-api"]
