FROM ubuntu:latest

# Update default packages
RUN apt-get -qq update
# Get Ubuntu packages
RUN apt-get install -y -q build-essential curl
# Install dependencies
RUN apt-get install -y git wget flex bison gperf python3 python3-pip python3-venv cmake ninja-build ccache libffi-dev libssl-dev dfu-util libusb-1.0-0
RUN mkdir -p ~/esp
RUN cd ~/esp && git clone -b v5.2 --recursive https://github.com/espressif/esp-idf.git
RUN cd ~/esp/esp-idf && ./install.sh all
