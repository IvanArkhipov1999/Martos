FROM ubuntu:latest

RUN apt-get -qq update

RUN apt-get install -y -q build-essential curl

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo --help
RUN #/bin/bash -c cargo install espup
RUN #espup install