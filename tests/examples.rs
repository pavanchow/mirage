use mirage::{assemble, run_collect, VmError};
use std::fs;

fn load(name: &str) -> String {
    fs::read_to_string(format!("examples/{}", name)).expect("example file exists")
}

#[test]
fn factorial_of_five_is_120() {
    let program = assemble(&load("factorial.asm")).expect("assembles");
    let out = run_collect(&program).expect("runs");
    assert_eq!(out, "120\n");
}

#[test]
fn fibonacci_twelve_is_144() {
    let program = assemble(&load("fibonacci.asm")).expect("assembles");
    let out = run_collect(&program).expect("runs");
    assert_eq!(out, "144\n");
}

#[test]
fn sum_one_to_ten_is_55() {
    let program = assemble(&load("sum_loop.asm")).expect("assembles");
    let out = run_collect(&program).expect("runs");
    assert_eq!(out, "55\n");
}

#[test]
fn stack_underflow_never_panics() {
    let program = assemble("ADD\nHALT\n").expect("assembles");
    let err = run_collect(&program).unwrap_err();
    assert_eq!(err, VmError::StackUnderflow);
}

#[test]
fn division_by_zero_never_panics() {
    let program = assemble("PUSH 10\nPUSH 0\nDIV\nHALT\n").expect("assembles");
    let err = run_collect(&program).unwrap_err();
    assert_eq!(err, VmError::DivisionByZero);
}

#[test]
fn jump_to_undefined_label_is_an_error_not_a_panic() {
    let err = assemble("JMP nowhere\nHALT\n").unwrap_err();
    assert!(err.to_string().contains("undefined label"));
}
