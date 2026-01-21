FROM ubuntu:latest

RUN apt update && apt install -y \
    meson ninja-build \
    libglib2.0-0 libpixman-1-0 libsdl2-2.0-0 libslirp0 libgcrypt20-dev libgcrypt20 libslirp-dev \
    libpixman-1-dev libsdl2-dev libgtk-3-dev libvte-2.91-dev libspice-server-dev libusb-1.0-0-dev \
    libcap-ng-dev libattr1-dev libaio-dev libnfs-dev libiscsi-dev libcurl4-openssl-dev libssh-dev \
    libpng-dev libjpeg-dev libncurses5-dev libseccomp-dev libfdt-dev libvirglrenderer-dev libepoxy-dev \
    libdrm-dev libgbm-dev libpmem-dev libdaxctl-dev liburing-dev
RUN apt install -y python3.11
RUN curl -sS https://bootstrap.pypa.io/get-pip.py | python3.11
RUN python3.11 -m pip install --upgrade pip setuptools
RUN git clone -b esp-develop-9.2.2-20250228 https://github.com/espressif/qemu.git --depth 1 --single-branch
WORKDIR /qemu
RUN python3.11 -m venv .venv
RUN . .venv/bin/activate && \
    pip install sphinx==5.3.0 sphinx_rtd_theme==1.1.1 && \
    ./configure --target-list=xtensa-softmmu,riscv32-softmmu && \
    make -j$(nproc) && \
    make install \
