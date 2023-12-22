use toy_vm::VM;

fn main() {
    println!("Starting VM...");

    let mut vm = VM::default();

    let addr_start = toy_vm::PC_START;

    let mut prgrm = [0; u16::MAX as usize + 1];
    prgrm[addr_start as usize] = 718;

    vm.load(prgrm);

    vm.run();


    println!("Work done, exiting the VM !");
}
