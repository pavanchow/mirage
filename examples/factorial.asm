; factorial(5) via recursion, prints 120
main:
    PUSH 5
    CALL fact
    PRINT
    HALT

; fact expects n on top of stack, leaves result on top, RET
fact:
    DUP
    PUSH 2
    LT          ; flag = (n < 2)
    JZ recurse
    POP         ; n < 2, drop n
    PUSH 1
    RET
recurse:
    DUP
    PUSH 1
    SUB         ; n - 1
    CALL fact   ; fact(n - 1)
    MUL         ; n * fact(n - 1)
    RET
