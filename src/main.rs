use std::{io::{Stdout, StdinLock}, fs::File, env};

use toy_vm::VM;

fn main() {
    println!("Starting VM...");

    let mut vm: VM<StdinLock, Stdout> = VM::default();

    let mut args = env::args();
    args.next();
    let program_path = args.next().expect("The first argument is the program path");

    let f = File::open(program_path).expect("Path exist");

    vm.load(f);

    vm.run();


    println!("Work done, exiting the VM !");
}
