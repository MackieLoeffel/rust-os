; Multiboot-Konstanten
MULTIBOOT_PAGE_ALIGN equ 1<<0
MULTIBOOT_MEMORY_INFO equ 1<<1
MULTIBOOT_HEADER_MAGIC equ 0x1badb002
MULTIBOOT_HEADER_FLAGS equ MULTIBOOT_PAGE_ALIGN | MULTIBOOT_MEMORY_INFO
MULTIBOOT_HEADER_CHKSUM equ -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)
MULTIBOOT_START_ADDRESS equ 0x100000
INIT_STACK_SIZE equ (1024*4)

; Deskriptoren global machen für realmode.asm
;; [GLOBAL idt_desc_global]

; Globaler Einsprungspunkt für das System
[GLOBAL startup]

; externe Funktionen und Variablen, die hier benötigt werden
;; [EXTERN guardian] ; Wird zur Interruptbehandlung aufgerufen
;; [EXTERN ap_stack] ; hier liegt die Adresse des Stacks für den gerade gebooteten AP
;; [EXTERN kernel_init] ; Wird zur Interruptbehandlung aufgerufen
;; [EXTERN gdt_desc_global]        ; Der "Pointer" zur globalen Deskriptortabelle (machine/gdt.cc)
[EXTERN start]

[SECTION .text]

startup:
	jmp skip_multiboot_hdr

; multiboot header, auch der Zugriff über das Symbol ist aligned
align 4
multiboot_header:
	dd MULTIBOOT_HEADER_MAGIC
	dd MULTIBOOT_HEADER_FLAGS
	dd MULTIBOOT_HEADER_CHKSUM

skip_multiboot_hdr:

    call start

    hlt





; Unterbrechungen sperren
	;; cli
; NMI verbieten
	;; mov al, 0x80
	;; out 0x70, al

    ;; mov dword [ap_stack], init_stack + INIT_STACK_SIZE

;; segment_init:
; GDT setzen
	;; lgdt [gdt_desc_global]
; Unterbrechungsbehandlung sicherstellen
    ;; lidt [idt_desc_global]

; Datensegmentregister initialisieren
	;; mov ax, 0x10
	;; mov ds, ax
	;; mov es, ax
	;; mov fs, ax
	;; mov gs, ax
	;; mov ss, ax

; cs Segment Register neu laden
	;; jmp 0x8:load_cs

;; load_cs:

; Stackpointer initialisieren
	;; mov esp, [ap_stack]
; Richtung für String-Operationen festlegen
	;; cld

; Wechsel in C/C++
    ;; call kernel_init

; Startup-Code fuer die APs
;; startup_ap:
; Alle Segmentselektoren neu laden, die zeigen noch auf die ap_gdt
; Nach Intel Manual Kapitel 9.9.1 ist das nötig
	;; mov	ax, 0x10
	;; mov	ds, ax
	;; mov	es, ax
	;; mov	fs, ax
	;; mov	gs, ax
	;; mov	ss, ax

; jetzt ist die CPU im protected mode

    ;; jmp segment_init

;Unterbrechungsbehandlungen

; Die Interruptbehandlung muss in Assembler gestartet werden, um die
;; IRQ <irq-num> <error-code?>
;; %macro IRQ 2
;; align 8
;; irq_entry_%1:
    ;;  Den CPU Kontext Sichern
    ;; push edx
    ;; push ecx
    ;; push eax
    ;; Einen Pointer auf den CPU Kontext als zweites Argument pushen
    ;; push esp
    ;; Die Interrupt-Nummer ist das erste Argument
    ;; push %1
    ;; call guardian
    ;; add esp, 8                 ; 2 Argumente vom Stack holen
    ;; pop eax
    ;; pop ecx
    ;; pop edx
    ;; iret
;; %endmacro

;; IRQ 0, 0
;; IRQ 1, 0
;; IRQ 2, 0
;; IRQ 3, 0
;; IRQ 4, 0
;; IRQ 5, 0
;; IRQ 6, 0
;; IRQ 7, 0
;; IRQ 8, 1
;; IRQ 9, 0
;; IRQ 10, 1
;; IRQ 11, 1
;; IRQ 12, 1
;; IRQ 13, 1
;; IRQ 14, 1
;; IRQ 15, 0
;; IRQ 16, 0
;; IRQ 17, 1

;; %assign i 18
;; %rep 238
;; IRQ i, 0
;; %assign i i+1
;; %endrep

;; [SECTION .data]
;  'interrupt descriptor table' mit 256 Eintraegen.
;; align 4
;; idt:
;; %macro idt_entry 1
	;; dw (irq_entry_%1 - startup + MULTIBOOT_START_ADDRESS) & 0xffff
	;; dw 0x0008
    	;; dw	0x8e00
	;; dw ((irq_entry_%1 - startup + MULTIBOOT_START_ADDRESS) & 0xffff0000) >> 16
;; %endmacro

;; %assign i 0
;; %rep 256
;; idt_entry i
;; %assign i i+1
;; %endrep

;; idt_desc_global:
	;; dw	256*8-1		; idt enthaelt 256 Eintraege
	;; dd	idt

;; [SECTION .bss]

init_stack:
    resb INIT_STACK_SIZE

; setup_ap, Start der restlichen Prozessoren
; Umschaltung in den 'Protected-Mode'
; Dieser Code wird von APICSystem::copySetupAPtoLowMem() reloziert!
;; [SECTION .setup_ap_seg]

;; USE16

;; setup_ap:
; Segmentregister initialisieren
	;; mov	ax,cs ; Daten- und Codesegment sollen
	;; mov	ds,ax ; hierher zeigen. Stack brauchen wir hier nicht.

; Unterbrechungen sperren
	;; cli
; NMI verbieten
	;; mov al, 0x80
	;; out 0x70, al

; vorrübergehende GDT setzen
	;; lgdt [ap_gdtd - setup_ap]

; Umschalten in den Protected Mode
	;; mov eax,cr0 ; Setze PM bit im Kontrollregister 1
	;; or  eax,1
	;; mov cr0,eax
	;; jmp	dword 0x08:startup_ap

;; align 4
;; ap_gdt:
    ;; dw 0,0,0,0   ; NULL Deskriptor
; Codesegment von 0-4GB
    ;; dw 0xFFFF    ; 4Gb - (0x100000*0x1000 = 4Gb)
    ;; dw 0x0000    ; base address=0
    ;; dw 0x9A00    ; code read/exec
    ;; dw 0x00CF    ; granularity=4096, 386 (+5th nibble of limit)
; Datensegment von 0-4GB
    ;; dw 0xFFFF    ; 4Gb - (0x100000*0x1000 = 4Gb)
    ;; dw 0x0000    ; base address=0
    ;; dw 0x9200    ; data read/write
    ;; dw 0x00CF    ; granularity=4096, 386 (+5th nibble of limit)

;; ap_gdtd:
	;; dw $ - ap_gdt - 1 ; Limit
	;; dd 0x40000 + ap_gdt - setup_ap ; Physikalische Adresse der ap_gdt
