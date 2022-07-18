FROM rust:1.62-bullseye AS builder

WORKDIR /src

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN mkdir -p ./src && \
    echo "fn main() {}" > ./src/main.rs && \
    cargo build --release

COPY . .
RUN touch ./src/main.rs
RUN cargo build --release

# ---

FROM gcr.io/distroless/cc
COPY --from=builder /src/target/release/automan /bin/automan
ENTRYPOINT ["/bin/automan"]
