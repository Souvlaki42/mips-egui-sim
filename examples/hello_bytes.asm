.data
  bytes: .byte 72, 101, 108, 108, 111, 32, 98, 121, 116, 101, 115, 10, 0
.text
.globl main
main:
  li $v0, 4
  la $a0, bytes
  syscall
  li $v0, 10
  syscall
