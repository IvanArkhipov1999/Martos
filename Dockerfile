FROM espressif/idf-rust:latest

RUN apt-get update && \
    apt-get install -y \
      libudev-dev \
      pkg-config && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-espflash --locked
