.PHONY: all qemu clean iso cargo

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

KERNEL = $(OBJDIR)/system
ISO = $(OBJDIR)/os.iso
ISO_CIP = $(OBJDIR)/os-cip.iso
GRUB_CFG = ./boot/grub.cfg
SECTIONS = boot/sections.ld
QEMUCPUs = 4
QEMUINITRD = /dev/null

RUST_SOURCES = $(shell find . -name "*.rs")
TARGET_TRIPLE = x86_64-unknown-linux-gnu
RUST_LIB = ./target/$(TARGET_TRIPLE)/debug/librust_os.a

all: $(KERNEL)

$(OBJDIR)/_%.o : boot/%.asm Makefile
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(ASM) $(ASMFLAGS) -o $@ $<

cargo: $(RUST_LIB)

$(RUST_LIB): $(RUST_SOURCES)
	cargo build $(CARGO_FLAGS) --target=$(TARGET_TRIPLE)

$(KERNEL): $(RUST_LIB) $(OBJPRE) Makefile $(SECTIONS)
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(LD) --gc-sections -T $(SECTIONS) -o $(KERNEL) $(LDFLAGS) $(OBJPRE) $(RUST_LIB)

iso: $(ISO)

$(ISO): $(KERNEL) $(GRUB_CFG)
	mkdir -p build/isofiles/boot/grub
	cp $(KERNEL) build/isofiles/boot/system
	cp $(GRUB_CFG) build/isofiles/boot/grub
	grub-mkrescue -o $(ISO) build/isofiles 2> /dev/null

qemu: $(ISO)
	$(QEMU) -cdrom $(ISO)

iso-cip: $(ISO_CIP)

$(ISO_CIP): $(KERNEL) $(GRUB_CFG)
	mkdir -p build/isofiles/boot/grub
	cp $(KERNEL) build/isofiles/boot/system
	cp $(GRUB_CFG) build/isofiles/boot/grub
	rsync -rz build/isofiles "cip:/tmp/rust-os/"
	ssh cip "grub-mkrescue -o /tmp/rust-os/os.iso /tmp/rust-os/isofiles 2> /dev/null"
	rsync -z "cip:/tmp/rust-os/os.iso" $(ISO_CIP)

qemu-cip: $(ISO_CIP)
	$(QEMU) -cdrom $(ISO_CIP)

clean:
	rm -rf $(OBJDIR)
