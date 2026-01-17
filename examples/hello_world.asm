.data
  str: .asciiz "Hello world\n"
.text
.globl main
main:
  li $v0, 4
  la $a0, str
  syscall
  li $v0, 10
  syscall
