use std::{
    env,
    fs::File,
    io::{Stdin, Stdout},
};

use toy_vm::{VM, unsafe_zone};




fn main() {
    println!("Starting VM...");

    
    unsafe_zone::disable_input_buffering();

    let mut vm: VM<Stdin, Stdout> = VM::default();

    let mut args = env::args();
    args.next();
    let program_path = args.next().expect("The first argument is the program path");

    let f = File::open(program_path).expect("Path exist");

    vm.load(f);

    vm.run();

    unsafe_zone::restore_input_buffering();

    println!("Work done, exiting the VM !");
}

