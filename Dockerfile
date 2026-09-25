FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

RUN apt update && apt install -y  \
  build-essential                 \
  curl                            \
  ca-certificates                 \
  git

ENV RUSTUP_HOME=/tmp/rustup
ENV CARGO_HOME=/tmp/cargo
ENV PATH=/tmp/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  | sh -s -- -y --no-modify-path

ARG TARGETARCH
CMD [ "bash" ]
