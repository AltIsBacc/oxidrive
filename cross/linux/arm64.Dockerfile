ARG CROSS_BASE_IMAGE
FROM $CROSS_BASE_IMAGE

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_PATH="/usr/lib/aarch64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH"

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    libasound2-dev:arm64 \
    libjack-jackd2-dev:arm64 \
    libjack-jackd2-0:arm64 && \
    rm -rf /var/lib/apt/lists/*

