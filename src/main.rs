use std::{env, fs::File, io::Stdout, time::Instant};

use toy_vm::{unsafe_zone, LibCReader, VM};

fn main() {
    println!("Starting VM...");

    unsafe_zone::disable_input_buffering();

    let mut vm: VM<LibCReader, Stdout> = VM::default();

    let mut args = env::args();
    args.next();
    let program_path = args.next().expect("The first argument is the program path");

    let f = File::open(program_path).expect("Path exist");

    vm.load(f);

    let start = Instant::now();
    let nb_instructions = vm.run();
    let duration = start.elapsed();

    println!("executed {nb_instructions} instructions in {:?}", duration);

    unsafe_zone::restore_input_buffering();
}
