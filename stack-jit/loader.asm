mov rax, rdi 
mov r8d, esi 
mov r9, rdx 

imul rsi, 8
sub rsi, 32

mov rbx, rsi

cmp r8d, 0x1
jl end
mov rdi, [rax]

cmp r8d, 0x2
jl end
mov rsi, [rax+8]

cmp r8d, 0x3
jl end
mov rdx, [rax+16]

cmp r8d, 0x4
jl end
mov rcx, [rax+24]

push_loop:
cmp r8d, 0x4
je end
dec r8d

push [rax+r8*8]

jmp push_loop

end:

call r9

cmp rbx, 0x0
jle no_stack_clear
add rsp, rbx

no_stack_clear:
ret