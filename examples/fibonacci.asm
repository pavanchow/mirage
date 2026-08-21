; iterative fibonacci, prints fib(12) = 144
; locals: 0 = a, 1 = b, 2 = remaining iterations
    PUSH 0
    STORE 0
    PUSH 1
    STORE 1
    PUSH 12
    STORE 2
loop:
    LOAD 2
    JZ done
    LOAD 0
    LOAD 1
    ADD         ; temp = a + b
    LOAD 1
    STORE 0     ; a = b
    STORE 1     ; b = temp
    LOAD 2
    PUSH 1
    SUB
    STORE 2     ; remaining -= 1
    JMP loop
done:
    LOAD 0
    PRINT
    HALT
