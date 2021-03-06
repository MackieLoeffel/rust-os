global startup
section .multiboot_header
header_start:
    dd 0xe85250d6                ; magic number (multiboot 2)
    dd 0                         ; architecture 0 (protected mode i386)
    dd header_end - header_start ; header length
    ; checksum
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

    ; insert optional multiboot tags here

    ; required end tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end:

extern long_mode_start
section .text
bits 32
startup:
    mov esp, stack_top
    mov edi, ebx                ; save Multiboot info pointer

    call check_multiboot
    call check_cpuid
    call check_long_mode
    call set_up_SSE

    call setup_page_tables
    call enable_paging

    lgdt [gdt64.pointer]
    ;;  update selectors
    mov ax, gdt64.data
    mov ss, ax
    mov ds, ax
    mov es, ax

    jmp gdt64.code:long_mode_start

setup_page_tables:
    ;; create pages
    mov eax, p3_table
    or eax, 0b1000000011        ; used & present & writeable
    mov [p4_table], eax

    mov eax, p2_table
    or eax, 0b1000000011        ; used & present & writeable
    mov [p3_table], eax

    ;;  map each p2 entry to a huge page 2 MiB page
    mov ecx, 0
.map_p2_table:
    mov eax, 0x200000
    mul ecx
    or eax, 0b1010000011        ; used & present 6 writable & huge
    mov [p2_table + ecx * 8], eax

    inc ecx
    cmp ecx, 512
    jne .map_p2_table

    ;; map p4 recursively, as described here: http://os.phil-opp.com/modifying-page-tables.html#recursive-mapping
    mov eax, p4_table
    or eax, 0b1000000011        ; used & present & writable
    mov [p4_table + 511 * 8], eax

    ret

enable_paging:
    ;;  load P4 to cr3 register (cpu uses this to access the P4 table)
    mov eax, p4_table
    mov cr3, eax

    ;;  enable PAE-flag in cr4 (Physical Address Extension)
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ;;  set the long mode bit in the EFER MSR (model specific register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ;;  enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret

    ;;  Prints `ERR: ` and the given error code to screen and hangs.
    ;;  parameter: error code (in ascii) in al
error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

check_multiboot:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
    .no_multiboot:
    mov al, "0"
    jmp error

check_cpuid:
    ;;  Check if CPUID is supported by attempting to flip the ID bit (bit 21)
    ;;  in the FLAGS register. If we can flip it, CPUID is available.

    ;;  Copy FLAGS in to EAX via stack
    pushfd
    pop eax

    ;;  Copy to ECX as well for comparing later on
    mov ecx, eax

    ;;  Flip the ID bit
    xor eax, 1 << 21

    ;;  Copy EAX to FLAGS via the stack
    push eax
    popfd

    ;;  Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax

    ;;  Restore FLAGS from the old version stored in ECX (i.e. flipping the
    ;; ID bit back if it was ever flipped).
    push ecx
    popfd

    ;;  Compare EAX and ECX. If they are equal then that means the bit
    ;;  wasn't flipped, and CPUID isn't supported.
    cmp eax, ecx
    je .no_cpuid
    ret
    .no_cpuid:
    mov al, "1"
    jmp error

check_long_mode:
    ;;  test if extended processor info in available
    mov eax, 0x80000000     ; implicit argument for cpuid
    cpuid               ; get highest supported argument
    cmp eax, 0x80000001     ; it needs to be at least 0x80000001
    jb .no_long_mode    ; if it's less, the CPU is too old for long mode

    ;;  use extended info to test if long mode is available
    mov eax, 0x80000001     ; argument for extended processor info
    cpuid               ; returns various feature bits in ecx and edx
    test edx, 1 << 29       ; test if the LM-bit is set in the D-register
    jz .no_long_mode    ; If it's not set, there is no long mode
    ret
.no_long_mode:
    mov al, "2"
    jmp error
    ;;  Check for SSE and enable it. If it's not supported throw error "a".
set_up_SSE:
    ;;  check for SSE
    mov eax, 0x1
    cpuid
    test edx, 1<<25
    jz .no_SSE

    ;;  enable SSE
    mov eax, cr0
    and ax, 0xFFFB          ; clear coprocessor emulation CR0.EM
    or ax, 0x2          ; set coprocessor monitoring  CR0.MP
    mov cr0, eax
    mov eax, cr4
    or ax, 3 << 9           ; set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
    mov cr4, eax

    ret
    .no_SSE:
    mov al, "a"
    jmp error

section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
stack_bottom:
    resb 4096 * 8
stack_top:

section .rodata
gdt64:
    dq 0                    ; zero entry
.code: equ $ - gdt64
    dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53) ; code segment
.data: equ $ - gdt64
    dq (1<<44) | (1<<47) | (1<<41)                 ; data segment
.pointer:
    dw $ - gdt64 - 1
    dq gdt64
