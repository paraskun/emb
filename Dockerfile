FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

RUN apt update && apt install -y  \
  build-essential                 \
  curl                            \
  ca-certificates                 \
  git

ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  | sh -s -- -y --no-modify-path

ARG TARGETARCH

RUN useradd -m -s /bin/zsh paraskun \
  && echo "paraskun ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

RUN chown -R paraskun:paraskun ${RUSTUP_HOME}
RUN chown -R paraskun:paraskun ${CARGO_HOME}

USER paraskun
ENV USER=paraskun

RUN curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install \
  | sh -s -- --no-daemon

WORKDIR /home/paraskun
RUN git clone https://github.com/paraskun/nixos.git

WORKDIR /home/paraskun/nixos
ENV PATH=/home/paraskun/.nix-profile/bin:$PATH
ENV PATH=/run/current-system/sw/bin:$PATH
ENV PATH=/nix/var/nix/profiles/default/bin:$PATH
ENV NIX_CONFIG="experimental-features = nix-command flakes"
RUN nix run home-manager -- switch --flake .#container@${TARGETARCH}

WORKDIR /home/paraskun
