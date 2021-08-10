# Based from https://github.com/paritytech/substrate/blob/master/.maintain/Dockerfile

FROM phusion/baseimage:bionic-1.0.0 as builder
LABEL maintainer="hi@zeitgeit.pm"
LABEL description="This is the build stage for the Zeitgeist node. Here is created the binary."

ENV DEBIAN_FRONTEND=noninteractive

ARG PROFILE=release
ARG FEATURES=default
WORKDIR /zeitgeist

COPY . /zeitgeist

RUN apt-get update && \
    apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
    apt-get install -y cmake pkg-config libssl-dev git clang libclang-dev

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    export PATH="$PATH:$HOME/.cargo/bin" && \
    rustup toolchain install nightly-2021-08-09 && \
    rustup target add wasm32-unknown-unknown --toolchain nightly-2021-08-09 && \
    rustup default stable && \
    cargo build "--$PROFILE" --features "$FEATURES"

# ==== SECOND STAGE ====

FROM phusion/baseimage:bionic-1.0.0
LABEL maintainer="hi@zeitgeist.pm"
LABEL description="This is the 2nd stage: a very small image where we copy the Zeigeist node binary."
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
    rm -rf /usr/share/* && \
    mv /tmp/ca-certificates /usr/share/ && \
    useradd -m -u 1000 -U -s /bin/sh -d /zeitgeist zeitgeist

COPY --from=builder /zeitgeist/target/$PROFILE/zeitgeist /usr/local/bin

# checks
RUN ldd /usr/local/bin/zeitgeist && \
    /usr/local/bin/zeitgeist --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
    rm -rf /usr/bin /usr/sbin /usr/share/man

USER zeitgeist
EXPOSE 30333 9933 9944

RUN mkdir /zeitgeist/data

VOLUME ["/zeitgeist/data"]

ENTRYPOINT ["/usr/local/bin/zeitgeist"]
