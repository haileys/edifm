FROM debian:stretch

RUN apt-get update \
    && apt-get install -y \
        build-essential \
        git \
        curl \
        libpq-dev \
        libmp3lame-dev

RUN curl https://sh.rustup.rs -sSf | sh /dev/stdin -y

# RUN apk update \
#    && apk add \
#        libpq \
#        lame \
#        gcc \
#        curl \
#        git \
#        rust \
#        cargo

# RUN /root/.cargo/bin/rustup target add x86_64-unknown-linux-musl

WORKDIR /workspace
ENV CARGO_HOME=/workspace/.cargo
