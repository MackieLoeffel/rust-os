.PHONY: all qemu clean iso

ASM = nasm
CXX = g++
LD = ld
QEMU = qemu-system-x86_64

ASM_SOURCES = $(shell find . -name "*.asm")
ASM_OBJECTS = $(patsubst %.asm,_%.o, $(notdir $(ASM_SOURCES)))
OBJPRE = $(addprefix $(OBJDIR)/,$(ASM_OBJECTS))
OBJDIR = ./build

ASMFLAGS = -f elf64
LDFLAGS = -n
LDLIBS =

KERNEL = $(OBJDIR)/system
ISO = $(OBJDIR)/os.iso
GRUB_CFG = ./boot/grub.cfg
SECTIONS = boot/sections.ld
QEMUCPUs = 4
QEMUINITRD = /dev/null

TARGET_TRIPLE = x84_64-unknown-linux-gnu
RUST_LIB_DIR = ./target/$(TARGET_TRIPLE)/debug/

all: $(QEMUKERNEL)

$(OBJDIR)/_%.o : boot/%.asm Makefile
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(ASM) $(ASMFLAGS) -o $@ $<

$(KERNEL): $(OBJPRE) Makefile
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(LD) -T $(SECTIONS) -o $(KERNEL) $(LDFLAGS) $(OBJPRE) $(LDLIBS)

iso: $(ISO)

$(ISO): $(KERNEL) $(GRUB_CFG)
	mkdir -p build/isofiles/boot/grub
	cp $(KERNEL) build/isofiles/boot/system
	cp $(GRUB_CFG) build/isofiles/boot/grub
	grub-mkrescue -o $(ISO) build/isofiles 2> /dev/null

qemu: iso
	$(QEMU) -cdrom $(ISO)

clean:
	rm -rf $(OBJDIR)
