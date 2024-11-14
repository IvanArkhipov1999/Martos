# C example for risc-v esp32-c6 architecture

Presented here is a straightforward C example utilizing Martos.

It has empty setup function.
Additionally, within the loop function, the counter value is incremented fifty times.

## How to install dependencies

For comprehensive guidance on installing the necessary dependencies for developing applications targeting the RISC-V ESP32-C6 architecture,
please refer to [the official website](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/index.html#manual-installation).
Below is an illustrative example demonstrating the installation of building toolchains on a Linux (Ubuntu/Debian):
```
sudo apt-get install -y git wget flex bison gperf python3 python3-pip python3-venv cmake ninja-build ccache libffi-dev libssl-dev dfu-util libusb-1.0-0 make
mkdir -p ~/esp
cd ~/esp
git clone -b v5.2 --recursive https://github.com/espressif/esp-idf.git
cd ~/esp/esp-idf
./install.sh esp32c6
```

## Before building the example

Before proceeding with building the example, it is essential to obtain the Martos C static library
and [link it](https://github.com/IvanArkhipov1999/Martos/blob/main/examples/c-examples/risc-v-esp32-c6/Makefile#L19) with the example code.

There are multiple avenues through which you can acquire the Martos C static library:
1. [From release artifacts.](https://github.com/IvanArkhipov1999/Martos/releases)
2. [From ci artifacts.](https://github.com/IvanArkhipov1999/Martos/actions)
3. [To build the Martos C static library independently.](https://github.com/IvanArkhipov1999/Martos/tree/main/c-library/risc-v-esp32-c6)


## How to build the example

For a thorough guide on developing projects for the RISC-V ESP32-C6 architecture across various operating systems,
we recommend consulting [the official website](https://docs.espressif.com/projects/esp-idf/en/latest/esp32c6/get-started/index.html#build-your-first-project).
Below, you will find an illustrative example showcasing the building process on a Linux system (Ubuntu/Debian):
```
. $HOME/esp/esp-idf/export.sh
make
```

## How to run the example

To upload the program, you need to format it for the ESP32-C6 and then store it in the SPI Flash chip connected to the actual ESP32-C6 within the module.
You can do that with Espressif’s esptool utility.
To format the ELF file into a binary image:
```
esptool.py --chip esp32c6 elf2image --flash_mode="dio" --flash_freq "40m" --flash_size "4MB" -o main.bin main.elf
```

To flash a binary image to Flash address 0x1000 (where the ESP32-C6 expects a ‘bootloader’ to be located):
```
esptool.py --chip esp32c6 --port /dev/ttyUSB0 --baud 115200 --before default_reset --after hard_reset write_flash -z --flash_mode dio --flash_freq 40m --flash_size detect 0x1000 main.bin
```

To run the program:
```
esptool.py --chip esp32c6 --port /dev/ttyUSB0 --baud 115200 --before default_reset --after hard_reset run
```

Note that you might need to specify a different port, depending on which system resource your ESP32-C6 is connected to.
