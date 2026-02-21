FROM --platform=linux/amd64 rust:latest AS builder

WORKDIR /build

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y --no-install-recommends musl-tools && rm -rf /var/lib/apt/lists/*

ENV CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl --bin arma

FROM --platform=linux/amd64 scratch AS runtime

WORKDIR /app

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/arma /app/arma

EXPOSE 8080

ENTRYPOINT ["/app/arma", "start"]
