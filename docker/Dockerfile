ARG BUILDER_BASE_IMAGE="rust:bullseye"
ARG RUNNER_BASE_IMAGE="debian:bullseye-slim"

ARG SERVICE_BIN=time-service-server
ARG CLIENT_BIN=time-service-client

ARG ENTRYPOINT_LOCAL=docker/scripts/image-entrypoint.sh

ARG SERVICE_BIN_BUILDER=target/release/${SERVICE_BIN}
ARG CLIENT_BIN_BUILDER=target/release/${CLIENT_BIN}

ARG SERVICE_BIN_RUNNER=/usr/bin/${SERVICE_BIN}
ARG CLIENT_BIN_RUNNER=/usr/bin/${CLIENT_BIN}


## Build
FROM ${BUILDER_BASE_IMAGE} as builder
ARG SERVICE_BIN
ARG CLIENT_BIN

WORKDIR /app
COPY . .

# Use the sparse protocol
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN apt update && \
    apt-get install -y --no-install-recommends apt-utils && \
    apt install -y curl unzip && \
    rm -rf /var/lib/apt/lists/*

ENV PB_VERSION=24.4
ENV PB_REL="https://github.com/protocolbuffers/protobuf/releases"
RUN curl -LO $PB_REL/download/v${PB_VERSION}/protoc-${PB_VERSION}-linux-x86_64.zip && \
    unzip protoc-${PB_VERSION}-linux-x86_64.zip -d /usr/local

# Build our binaries (NOTE: --release).
RUN cargo build --release --bin ${SERVICE_BIN} --bin ${CLIENT_BIN}

## Build Runner
FROM ${RUNNER_BASE_IMAGE} AS runner
ARG ENTRYPOINT_LOCAL
ARG CLIENT_BIN_BUILDER
ARG CLIENT_BIN_RUNNER
ARG SERVICE_BIN_RUNNER
ARG SERVICE_BIN_BUILDER

RUN apt update && \
    apt-get install -y --no-install-recommends apt-utils && \
    apt install -y netcat dnsutils && \
    rm -rf /var/lib/apt/lists/*

COPY ${ENTRYPOINT_LOCAL} /usr/bin/image-entrypoint.sh
COPY --from=builder /app/${SERVICE_BIN_BUILDER} ${SERVICE_BIN_RUNNER}
COPY --from=builder /app/${CLIENT_BIN_BUILDER} ${CLIENT_BIN_RUNNER}


ENV RUST_LOG=info
ENV SERVER_LISTEN_ADDR=0.0.0.0
ENV SERVER_LISTEN_PORT=50051
ENV SERVER_ADDR=${SERVER_LISTEN_ADDR}
ENV SERVER_PORT=${SERVER_LISTEN_PORT}
ENV USE_CLIENT_BINARY=false

ENV SERVER_ENTRYPOINT_BINARY=${SERVICE_BIN_RUNNER}
ENV CLIENT_ENTRYPOINT_BINARY=${CLIENT_BIN_RUNNER}

ENTRYPOINT ["/usr/bin/image-entrypoint.sh"]