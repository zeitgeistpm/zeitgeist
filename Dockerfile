# Based from https://github.com/paritytech/substrate/blob/master/.maintain/Dockerfile

FROM phusion/baseimage:jammy-1.0.1 as builder
LABEL maintainer="hi@zeitgeit.pm"
LABEL description="This is the build stage for the Zeitgeist node. Here is created the binary."

ENV DEBIAN_FRONTEND=noninteractive

ARG PROFILE=production
ARG FEATURES=default
WORKDIR /zeitgeist

COPY . /zeitgeist

RUN apt-get update && \
    apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold"

RUN ./scripts/init.sh

RUN . "$HOME/.cargo/env" && cargo build --profile "$PROFILE" --features "$FEATURES"

# ==== SECOND STAGE ====

FROM phusion/baseimage:jammy-1.0.1
LABEL maintainer="hi@zeitgeist.pm"
LABEL description="This is the 2nd stage: a very small image where we copy the Zeigeist node binary."
ARG PROFILE=production

RUN mv /usr/share/ca* /tmp && \
    rm -rf /usr/share/* && \
    mv /tmp/ca-certificates /usr/share/ && \
    useradd -m -u 1000 -U -s /bin/sh -d /zeitgeist zeitgeist

COPY --from=builder /zeitgeist/target/$PROFILE/zeitgeist /usr/local/bin

# checks
RUN ldd /usr/local/bin/zeitgeist && \
    /usr/local/bin/zeitgeist --version

# Shrinking
RUN rm -rf /usr/lib/python* && rm -rf /usr/share/man

USER zeitgeist
EXPOSE 30333 9933 9944

RUN mkdir -p /zeitgeist/data

VOLUME ["/zeitgeist/data"]

ENTRYPOINT ["/usr/local/bin/zeitgeist"]
