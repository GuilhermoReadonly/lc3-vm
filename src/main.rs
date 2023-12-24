use std::io::{Stdout, StdinLock};

use toy_vm::VM;

fn main() {
    println!("Starting VM...");

    let mut vm: VM<StdinLock, Stdout> = VM::default();

    let addr_start = toy_vm::PC_START;

    let mut prgrm = [0; u16::MAX as usize + 1];
    prgrm[addr_start] = 0b0001000001000010; // add r1 and r2
    prgrm[addr_start + 1] = 0b1111000000100101; // halt

    vm.load(prgrm);

    vm.run();


    println!("Work done, exiting the VM !");
}
