TOOLCHAIN = riscv32-esp-elf-

CFLAGS_PLATFORM = -march=rv32imac -mabi=ilp32
LDFLAGS_PLATFORM = $(CFLAGS_PLATFORM)

CC = $(TOOLCHAIN)gcc

# Linker scripts are provided by the esp-hal build of the static library
# (memory map, interrupt vectors, ROM functions), see the -L paths below.

CFLAGS += -Wall -Werror -std=gnu11 -nostdlib $(CFLAGS_PLATFORM)
CFLAGS += -fno-strict-aliasing
CFLAGS += -fdata-sections -ffunction-sections
CFLAGS += -Os -g

MARTOS_RELEASE = ../../../c-library/risc-v-esp32-c6/target/riscv32imac-unknown-none-elf/release
ifeq ($(wildcard $(MARTOS_RELEASE)/build/esp-hal-*/out),)
$(error esp-hal linker scripts not found under $(MARTOS_RELEASE)/build - they must be extracted from the static-library artifact)
endif
LDFLAGS += -L$(MARTOS_RELEASE) -lrisc_v_esp32c6_static_lib
LDFLAGS += $(addprefix -L,$(wildcard $(MARTOS_RELEASE)/build/esp-hal-*/out) $(wildcard $(MARTOS_RELEASE)/build/esp32c6-*/out) $(wildcard $(MARTOS_RELEASE)/build/esp-riscv-rt-*/out) $(wildcard $(MARTOS_RELEASE)/build/esp-rom-sys-*/out))
LDFLAGS += -nostdlib -Tlinkall.x -Wl,-Map=$@.map -Wl,--cref -Wl,--gc-sections
LDFLAGS += $(LDFLAGS_PLATFORM)
LDFLAGS += -lgcc -lesp_rom_sys

C_SRC += ./src/main.c

OBJS += $(C_SRC:.c=.o)

.PHONY: all
all: main.elf

# Rules to build files.

%.o: %.c
	$(CC) -c $(CFLAGS) $< -o $@

main.elf: $(OBJS)
	$(CC) $^ $(LDFLAGS) -o $@

# Target to clean build artifacts.
.PHONY: clean
clean:
	rm -f $(OBJS)
	rm -f ./main.bin
	rm -f ./main.elf ./main.elf.map
