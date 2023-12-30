use std::io::{Stdout, StdinLock};

use toy_vm::VM;

fn main() {
    println!("Starting VM...");

    let mut vm: VM<StdinLock, Stdout> = VM::default();

    let addr_start = toy_vm::PC_START;

    let mut prgrm = [0; u16::MAX as usize + 1];
    // recupere la valeur de stdin
    prgrm[addr_start] = 0b1111000000100110;  // F026 ; tinu16 dans r0
    // initialiser R1 avec la valeur 0
    prgrm[addr_start + 1] = 0b0001001001100000;  // 1260 ; and r1 = r1 & 0 = 0
    // stocker la valeur de R0 en memoire a l'@ stockee en r1
    prgrm[addr_start + 2] = 0b0111000001000000;  // 7040 ; str
    // recuperer une seconde valeur
    prgrm[addr_start + 3] = 0b1111000000100110;  // F026 ; tinu16 dans r0
    // reprendre la 1e valeur de la memoire
    prgrm[addr_start + 4] = 0b0110010001000000;  // 6440 ; ldr
    // ajouter les 2 valeurs ensemble
    prgrm[addr_start + 5] = 0b0001000000000010;  // 1002 ; add r0 = r0 + r2
    // afficher l addition
    prgrm[addr_start + 6] = 0b1111000000100111;  // F027 ; toutu16 r0
    // retour a la ligne en memoire 7
    prgrm[7] = b'\n' as u16;
    // initialiser r0 a la valeur 0
    prgrm[addr_start + 7] = 0b0101000000100000; // 5020 ; 
    // ajouter 7 a r0
    prgrm[addr_start + 8] = 0b0001000000100111; // 1027 ; 
    // afficher le retour a la ligne
    prgrm[addr_start + 9] = 0b1111000000100010; // F022 ; 

    // halt
    prgrm[addr_start + 10] = 0b1111000000100101; // F025 ;  halt

    vm.load(prgrm);

    vm.run();


    println!("Work done, exiting the VM !");
}
