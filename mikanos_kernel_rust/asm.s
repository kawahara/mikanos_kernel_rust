bits 64

extern kernel_main2

section .bss align=16
kernel_main_stack:
  resb 1024 * 1024

section .text
global kernel_main
kernel_main:
  mov rsp, kernel_main_stack + 1024 * 1024
  call kernel_main2
.fin:
  hlt
  jmp .fin

global load_gdt ; load_gdt(limit: u16, offset: const* u64)
load_gdt:
  push rbp
  mov rbp, rsp
  sub rsp, 10
  mov [rsp], di ; limit
  mov [rsp + 2], rsi ; offset
  lgdt [rsp]
  mov rsp, rbp
  pop rbp
  ret

global set_csss ; set_csss(cs: u16, ss: u16)
set_csss:
  push rbp
  mov rbp, rsp
  mov ss, si
  mov rax, .next
  push rdi ; CS
  push rax ; RIP
  o64 retf
.next
  mov rsp, rbp
  pop rbp
  ret

global set_ds_all ; set_ds_all(value: u16)
set_ds_all:
  mov ds, di
  mov es, di
  mov fs, di
  mov gs, di
  ret

global set_cr3 ; set_cr3(address: u64)
set_cr3:
  mov cr3, rdi
  ret
