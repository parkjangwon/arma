# syntax=docker/dockerfile:1
FROM rust:latest AS builder

ARG TARGETARCH
WORKDIR /build

RUN apt-get update \
    && apt-get install -y --no-install-recommends musl-tools \
    && rm -rf /var/lib/apt/lists/*

RUN case "${TARGETARCH}" in \
      amd64) rustup target add x86_64-unknown-linux-musl ;; \
      arm64) rustup target add aarch64-unknown-linux-musl ;; \
      *) echo "unsupported TARGETARCH=${TARGETARCH}" && exit 1 ;; \
    esac

COPY . .

ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc

RUN case "${TARGETARCH}" in \
      amd64) cargo build --release --target x86_64-unknown-linux-musl --bin arma && cp target/x86_64-unknown-linux-musl/release/arma /tmp/arma ;; \
      arm64) cargo build --release --target aarch64-unknown-linux-musl --bin arma && cp target/aarch64-unknown-linux-musl/release/arma /tmp/arma ;; \
      *) exit 1 ;; \
    esac

FROM scratch AS runtime

WORKDIR /app
COPY --from=builder /tmp/arma /app/arma
COPY config.yaml /app/config.yaml
COPY filter_packs /app/filter_packs

EXPOSE 8080
ENTRYPOINT ["/app/arma", "start"]
