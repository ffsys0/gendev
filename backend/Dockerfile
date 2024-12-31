# ---- Build Stage ----
FROM rust:1.78.0-buster AS builder
WORKDIR /app

# Copy the Cargo files first so we can cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "// dummy file" > src/lib.rs \
    && cargo build
RUN cargo build --release
RUN rm -r src
COPY . .

RUN cargo build --release

# ---- Runtime Stage ----
FROM debian:bullseye-slim AS runtime

RUN mkdir -p /app

COPY --from=builder /app/target/release/backend /app/backend

COPY --from=builder /app/data /app/data

EXPOSE 8080

WORKDIR /app
ENTRYPOINT ["/app/backend"]
