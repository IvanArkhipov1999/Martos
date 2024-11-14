TOOLCHAIN = riscv32-esp-elf-

# CFLAGS_PLATFORM = -Wjump-misses-init -fstrict-volatile-bitfields
ASFLAGS_PLATFORM = $(CFLAGS_PLATFORM)
LDFLAGS_PLATFORM = $(CFLAGS_PLATFORM)

CC = $(TOOLCHAIN)gcc
LD = $(TOOLCHAIN)ld
OC = $(TOOLCHAIN)objcopy
OS = $(TOOLCHAIN)size

LDSCRIPT       = ./ld/esp32.ld

CFLAGS += $(INC) -Wall -Werror -std=gnu11 -nostdlib $(CFLAGS_PLATFORM) $(COPT)
CFLAGS += -fno-strict-aliasing
CFLAGS += -fdata-sections -ffunction-sections
CFLAGS += -Os -g

LDFLAGS += -L../../../c-library/risc-v-esp32-c6/target/riscv32imac-unknown-none-elf/release/ -lrisc_v_esp32c6_static_lib
LDFLAGS += -nostdlib -T$(LDSCRIPT) -Wl,-Map=$@.map -Wl,--cref -Wl,--gc-sections
LDFLAGS += $(LDFLAGS_PLATFORM)
LDFLAGS += -lm -lc -lgcc -Wl,--allow-multiple-definition
ASFLAGS += -c -O0 -Wall -fmessage-length=0
ASFLAGS += $(ASFLAGS_PLATFORM)

C_SRC += \
	./src/main.c \

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
	rm -f ./main.elf.map