ARG BASE_IMAGE=linuxdeepin/deepin:crimson
FROM ${BASE_IMAGE} AS builder

ENV DEBIAN_FRONTEND=noninteractive

WORKDIR /build

# Las dependencias de compilacion se leen de debian/control, asi el CI
# siempre instala exactamente lo que declara Build-Depends.
COPY debian/ /build/debian/
RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential dpkg-dev fakeroot \
        ca-certificates curl && \
    apt-get build-dep -y --no-install-recommends ./ && \
    rm -rf /var/lib/apt/lists/*

# crimson trae rustc 1.81 y cxx/cxx-qt piden >= 1.88: toolchain de rustup.
# debian/rules antepone ~/.cargo/bin al PATH.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
        sh -s -- -y --profile minimal --default-toolchain stable && \
    /root/.cargo/bin/cargo --version

# Descarga de crates en una capa propia (se reutiliza si no cambia Cargo.lock).
COPY Cargo.toml Cargo.lock /build/
COPY gui/Cargo.toml /build/gui/
RUN mkdir -p src gui/src && touch src/lib.rs src/main.rs gui/src/main.rs && \
    /root/.cargo/bin/cargo fetch --locked && \
    rm -rf src gui/src

COPY . /build

RUN dpkg-buildpackage -us -uc -b && \
    mkdir -p /output && \
    cp ../*.deb /output/

FROM scratch AS export
COPY --from=builder /output/ /
