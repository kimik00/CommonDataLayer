# syntax=docker/dockerfile:experimental

# -----------------
# Cargo Build Stage
# -----------------

FROM rust:1.47.0 as cargo-build

# For librdkafka
RUN apt update && apt install -y cmake

WORKDIR /usr/src/cdl/
COPY . ./

ARG ENV
ARG BIN

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/cdl/target \
    if [ "$ENV" = "DEV" ]; \
    then CARGO_PROFILE="debug"; \
    else CARGO_ARGS="--release"; CARGO_PROFILE="release"; \
    fi && \
    cargo build $CARGO_ARGS --workspace $FEATURE_FLAGS && \
    mkdir output && \
    bash -c "find target/$CARGO_PROFILE/$BIN -type f -executable | xargs -I{} cp {} output/"

RUN if [ "$ENV" != "DEV" ]; \
    then for f in output/*; do strip $f; done; fi
