FROM rust:1.80-bookworm AS build

ARG TARGETARCH
RUN echo "TARGETARCH is set to $TARGETARCH"

RUN mkdir /deps
WORKDIR /deps
RUN git clone https://github.com/google-coral/libedgetpu
RUN git clone --branch v2.16.1 --depth 1 https://github.com/tensorflow/tensorflow

RUN apt-get update && apt-get install -y \
    libusb-1.0-0-dev libavutil-dev libavcodec-dev pkg-config cmake xxd

RUN if [ "$TARGETARCH" = "amd64" ]; then \
    echo "LIBEDGETPU_ARCH=k8" >> /edgetpu-env; \
elif [ "$TARGETARCH" = "arm64" ]; then \
    echo "LIBEDGETPU_ARCH=aarch64" >> /edgetpu-env; \
else exit 1; \
fi

RUN wget https://github.com/bazelbuild/bazelisk/releases/download/v1.20.0/bazelisk-linux-$TARGETARCH
RUN chmod +x bazelisk-linux-$TARGETARCH
RUN mv bazelisk-linux-$TARGETARCH /usr/bin/bazel
WORKDIR /deps/libedgetpu
RUN bash -c 'source /edgetpu-env && CPU=$LIBEDGETPU_ARCH make -j8'

RUN mkdir /deps/tflite_build
WORKDIR /deps/tflite_build
RUN cmake ../tensorflow/tensorflow/lite/c && cmake --build . -j8

RUN bash -c 'source /edgetpu-env && echo "LIBEDGETPU_ARCH=$LIBEDGETPU_ARCH"'
RUN bash -c 'source /edgetpu-env && cp -v /deps/libedgetpu/out/throttled/$LIBEDGETPU_ARCH/*.so* /lib/ && cp /lib/libedgetpu.so.1.0 /lib/libedgetpu.so'
RUN cp -v /deps/tflite_build/*.so /lib/

RUN mkdir -p /app/libs
WORKDIR /app

COPY src /app/src
COPY plugins /app/plugins
COPY Cargo.toml Cargo.lock /app/
RUN cargo build \
    --package common \
    --package csv \
    --package env \
    --package fs \
    --package handler \
    --package hls \
    --package log@0.2.18 \
    --package monitor \
    --package mp4 \
    --package plugin \
    --package recdb \
    --package recording \
    --package rust-embed \
    --package vod \
    --package web \
    --package auth_basic \
    --package auth_none \
    --package motion \
    --package tflite \
    --package thumb_scale \
    --release
COPY web /app/web
RUN cargo build --package sentryshot --release

RUN ldd /app/target/release/sentryshot | grep "=> /" | awk '{print $3}' | xargs -I '{}' cp -v '{}' /app/libs/

FROM gcr.io/distroless/base-debian12
COPY --from=build /app/target/release/sentryshot /app/
COPY --from=build /app/target/release/*.so /app/
COPY web /app/web
COPY --from=build /app/libs/* /lib/
WORKDIR /app
ENV TZ=Etc/UTC
ENTRYPOINT ["/app/sentryshot", "run", "--config", "/app/configs/sentryshot.toml"]
