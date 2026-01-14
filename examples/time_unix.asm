.text
.globl main
main:
  li $v0, 30
  syscall

  li $v0, 1
  syscall

  move $a0, $a1
  li $v0, 1
  syscall

  li $v0, 10
  syscall
