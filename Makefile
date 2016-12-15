ASM = nasm
CXX = g++
LD = ld
QEMU = qemu-system-x86_64

ASM_SOURCES = $(shell find . -name "*.asm" -and ! -name "startup.asm")
ASM_OBJECTS = $(patsubst %.asm,_%.o, $(notdir $(ASM_SOURCES)))
OBJPRE = $(addprefix $(OBJDIR)/,$(ASM_OBJECTS))
STARTUP_OBJECT = $(OBJDIR)/_startup.o
OBJDIR = ./build

ASMFLAGS = -f elf64
LDFLAGS =
LDHEAD =
LDTAIL =
LDLIBS =

QEMUKERNEL = $(OBJDIR)/system
QEMUCPUs = 4
QEMUINITRD = /dev/null

TARGET_TRIPLE = x84_64-unknown-linux-gnu
RUST_LIB_DIR = ./target/$(TARGET_TRIPLE)/debug/

all: $(QEMUKERNEL)

$(OBJDIR)/_%.o : boot/%.asm Makefile
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(ASM) $(ASMFLAGS) -o $@ $<

$(QEMUKERNEL): $(STARTUP_OBJECT) $(OBJPRE) Makefile
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(LD) -T boot/sections.ld -o $(QEMUKERNEL) $(LDFLAGS) $(STARTUP_OBJECT) $(LDHEAD) $(OBJPRE) $(LDTAIL) $(LDLIBS)

qemu: all
	$(QEMU) -kernel $(QEMUKERNEL) -initrd $(QEMUINITRD) -k en-us -smp $(QEMUCPUs)

clean:
	rm -rf $(OBJDIR)
