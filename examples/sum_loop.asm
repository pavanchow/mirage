; sum 1..=10, prints 55
; locals: 0 = accumulator, 1 = counter
    PUSH 0
    STORE 0
    PUSH 1
    STORE 1
loop:
    LOAD 1
    PUSH 10
    GT
    JNZ done    ; if counter > 10, stop
    LOAD 0
    LOAD 1
    ADD
    STORE 0     ; acc += counter
    LOAD 1
    PUSH 1
    ADD
    STORE 1     ; counter += 1
    JMP loop
done:
    LOAD 0
    PRINT
    HALT
