# extending https://github.com/devcontainers/images/tree/main/src/rust
FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

# install missing perl libraries for latexindent formatter to work
RUN apt update && apt install libyaml-tiny-perl libfile-homedir-perl -y
