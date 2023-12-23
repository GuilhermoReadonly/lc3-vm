use std::collections::HashMap;
use std::fmt::Debug;

pub const PC_START: usize = 0x3000;

trait Instruction: Debug {
    fn execute(&self, vm: &mut VM);
}

pub struct VM {
    memory: Memory,
    registers: HashMap<Reg, u16>,
    halt: bool,
}

impl VM {
    pub fn load(&mut self, program: [u16; u16::MAX as usize + 1]) {
        self.memory.load(program)
    }
    pub fn run(&mut self) {
        while !self.halt {
            let current_addr = self.registers[&Reg::Rpc];
            let next_addr = self.registers[&Reg::Rpc] + 1;
            let instruction = self.memory.read(current_addr);

            println!("State: {:#?}", self.registers);

            print!("Instruction: {instruction:016b}.");

            let op: Box<dyn Instruction> = instruction.into();

            println!(" Decoded as {op:?}");

            op.execute(self);
            self.registers.insert(Reg::Rpc, next_addr);
        }
    }

    fn uf(&mut self, r: &Reg) {
        if self.registers[r] == 0 {
            self.registers.insert(Reg::Rcnd, 1 << 1);
        } else if self.registers[r] >> 15 == 1 {
            self.registers.insert(Reg::Rcnd, 1 << 2);
        } else {
            self.registers.insert(Reg::Rcnd, 1 << 0);
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self {
            memory: Memory::default(),
            registers: HashMap::from([
                (Reg::R0, 0),
                (Reg::R1, 0),
                (Reg::R2, 0),
                (Reg::R3, 0),
                (Reg::R4, 0),
                (Reg::R5, 0),
                (Reg::R6, 0),
                (Reg::R7, 0),
                (Reg::Rcnd, 0),
                (Reg::Rpc, PC_START as u16),
            ]),
            halt: false,
        }
    }
}

struct Memory {
    mem: [u16; u16::MAX as usize + 1],
}

impl Memory {
    fn read(&self, address: u16) -> u16 {
        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, val: u16) -> () {
        self.mem[address as usize] = val;
    }

    fn load(&mut self, memory: [u16; u16::MAX as usize + 1]) {
        self.mem = memory;
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            mem: [0; u16::MAX as usize + 1],
        }
    }
}

#[derive(Debug, PartialEq)]
struct AddConst {
    dr: Reg,
    sr: Reg,
    value: u16,
}

impl Instruction for AddConst {
    fn execute(&self, vm: &mut VM) {
        let result = vm.registers[&self.sr].wrapping_add(self.value);
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
    }
}

#[derive(Debug)]
struct AddReg {
    dr: Reg,
    sr1: Reg,
    sr2: Reg,
}

impl Instruction for AddReg {
    fn execute(&self, vm: &mut VM) {
        let result = vm.registers[&self.sr1].wrapping_add(vm.registers[&self.sr2]);
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
    }
}

#[derive(Debug, PartialEq)]
struct AndConst {
    dr: Reg,
    sr: Reg,
    value: u16,
}

impl Instruction for AndConst {
    fn execute(&self, vm: &mut VM) {
        let result = vm.registers[&self.sr] & self.value;
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
    }
}

#[derive(Debug)]
struct AndReg {
    dr: Reg,
    sr1: Reg,
    sr2: Reg,
}

impl Instruction for AndReg {
    fn execute(&self, vm: &mut VM) {
        let result = vm.registers[&self.sr1] & vm.registers[&self.sr2];
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
    }
}

#[derive(Debug)]
struct Ld {
    dr: Reg,
    offset: u16,
}

impl Instruction for Ld {
    fn execute(&self, vm: &mut VM) {
        let address = vm.registers[&Reg::Rpc] + self.offset;
        let result = vm.memory.read(address);
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
    }
}

#[derive(Debug)]
struct Ldi {
    dr: Reg,
    offset: u16,
}

impl Instruction for Ldi {
    fn execute(&self, vm: &mut VM) {
        let address1 = vm.registers[&Reg::Rpc] + self.offset;
        let address2 = vm.memory.read(address1);
        let result = vm.memory.read(address2);
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
    }
}

#[derive(Debug)]
struct St {
    sr: Reg,
    offset: u16,
}

impl Instruction for St {
    fn execute(&self, vm: &mut VM) {
        let address = vm.registers[&Reg::Rpc] + self.offset;
        let value = vm.registers[&self.sr];
        vm.memory.write(address, value);
    }
}

#[derive(Debug)]
struct Sti {
    sr: Reg,
    offset: u16,
}

impl Instruction for Sti {
    fn execute(&self, vm: &mut VM) {
        let address1 = vm.registers[&Reg::Rpc] + self.offset;
        let address2 = vm.memory.read(address1);
        let value = vm.registers[&self.sr];
        vm.memory.write(address2, value);
    }
}

#[derive(Debug)]
struct TrapHalt {}

impl Instruction for TrapHalt {
    fn execute(&self, vm: &mut VM) {
        vm.halt = true;
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
enum Reg {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    Rpc,
    Rcnd,
}

impl Reg {
    fn dr(instruction: u16) -> Self {
        let reg_nb = (instruction >> 9) & 0b0000000000000111;
        reg_nb.into()
    }
    fn sr1(instruction: u16) -> Self {
        let reg_nb = (instruction >> 6) & 0b0000000000000111;
        reg_nb.into()
    }
    fn sr2(instruction: u16) -> Self {
        let reg_nb = instruction & 0b0000000000000111;
        reg_nb.into()
    }
    fn imm(instruction: u16) -> u16 {
        instruction & 0b0000000000011111
    }

    /// if the bth bit of n is 1, fill up n with 1s the remaining bits else return n
    fn sext(n: u16, b: usize) -> u16 {
        if (n >> (b - 1)) & 1 == 1 {
            n | (0xFFFF << b)
        } else {
            n
        }
    }

    /// get offset 9
    fn poff9(n: u16) -> u16 {
        n & 0x1FF
    }

    /// Sign extend imm5
    fn sextimm(n: u16) -> u16 {
        Reg::sext(Reg::imm(n), 5)
    }

    /// Get the 5th bit as boolean
    fn fimm(instruction: u16) -> bool {
        Reg::get_nth_bit(instruction, 5)
    }

    fn get_nth_bit(value: u16, n: usize) -> bool {
        ((value >> n) & 1) == 1
    }
}

impl From<u16> for Reg {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::R0,
            1 => Self::R1,
            2 => Self::R2,
            3 => Self::R3,
            4 => Self::R4,
            5 => Self::R5,
            6 => Self::R6,
            7 => Self::R7,
            _ => panic!("The number {value} is not in [0..7]"),
        }
    }
}

impl From<u16> for Box<dyn Instruction> {
    fn from(instruction: u16) -> Self {
        let opcode = instruction >> 12;
        match opcode {
            // 0b0000 => Op::Br,
            0b0001 => {
                if Reg::fimm(instruction) {
                    Box::new(AddConst {
                        dr: Reg::dr(instruction),
                        sr: Reg::sr1(instruction),
                        value: Reg::sextimm(instruction),
                    })
                } else {
                    Box::new(AddReg {
                        dr: Reg::dr(instruction),
                        sr1: Reg::sr1(instruction),
                        sr2: Reg::sr2(instruction),
                    })
                }
            }
            0b0010 => {
                let dr = Reg::dr(instruction);
                let offset = Reg::poff9(instruction);
                Box::new(Ld { dr, offset })
            }
            0b0011 => {
                let sr = Reg::dr(instruction);
                let offset = Reg::poff9(instruction);
                Box::new(St { sr, offset })
            },
            // 0b0100 => Op::Jsr,
            0b0101 => {
                if Reg::fimm(instruction) {
                    Box::new(AndConst {
                        dr: Reg::dr(instruction),
                        sr: Reg::sr1(instruction),
                        value: Reg::sextimm(instruction),
                    })
                } else {
                    Box::new(AndReg {
                        dr: Reg::dr(instruction),
                        sr1: Reg::sr1(instruction),
                        sr2: Reg::sr2(instruction),
                    })
                }
            }
            // 0b0110 => Op::Ldr,
            // 0b0111 => Op::Str,
            // 0b1000 => Op::Rti,
            // 0b1001 => Op::Not,
            0b1010 => {
                let dr = Reg::dr(instruction);
                let offset = Reg::poff9(instruction);
                Box::new(Ldi { dr, offset })
            }
            0b1011 => {
                let sr = Reg::dr(instruction);
                let offset = Reg::poff9(instruction);
                Box::new(Sti { sr, offset })
            }
            // 0b1100 => Op::Jmp,
            // 0b1101 => Op::Unused,
            // 0b1110 => Op::Lea,
            0b1111 => Box::new(TrapHalt {}),
            _ => panic!("Op code {instruction} as no matching opcode"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_add_reg() {
        let mut vm = VM::default();

        vm.registers.insert(Reg::R1, 0b0000000000000100); // 4
        vm.registers.insert(Reg::R2, 0b0000000000000011); // 3

        let op = AddReg {
            dr: Reg::R0,
            sr1: Reg::R1,
            sr2: Reg::R2,
        };
        op.execute(&mut vm);
        assert_eq!(vm.registers[&Reg::R0], 0b0000000000000111); // 7

        vm.registers.insert(Reg::R3, 0b1111111111111100); // -4
        vm.registers.insert(Reg::R4, 0b0000000000000011); // 3
        let op = AddReg {
            dr: Reg::R0,
            sr1: Reg::R3,
            sr2: Reg::R4,
        };
        op.execute(&mut vm);
        assert_eq!(vm.registers[&Reg::R0], 0b1111111111111111); // -1

        vm.registers.insert(Reg::R6, 0b1111111111111111); // -1
        vm.registers.insert(Reg::R7, 0b1111111111111111); // -1
        let op = AddReg {
            dr: Reg::R0,
            sr1: Reg::R7,
            sr2: Reg::R6,
        };
        op.execute(&mut vm);
        assert_eq!(vm.registers[&Reg::R0], 0b1111111111111110); // -2
    }

    #[test]
    fn test_run() {
        let mut vm = VM::default();

        let mut program = [0; u16::MAX as usize + 1];
        program[PC_START + 0] = 0b0001001001100011; // add r1/0 and 3 in r1/3
        program[PC_START + 1] = 0b0001010010100100; // add r2/0 and 4 in r2/4
        program[PC_START + 2] = 0b0001000001000010; // add r1/3 and r2/4 in r0/7
        program[PC_START + 3] = 0b0101001001100001; // and r1/3 and 1 in r1/1
        program[PC_START + 4] = 0b0101111000000010; // and r0/7 and r2/4 in r7/4
        program[PC_START + 5] = 0b0010101100000000; // ld offset 256 DATA1/21845 in r5/21845
        program[PC_START + 6] = 0b1010100100000000; // ldi offset 256 DATA2/0 Data3/718 in r4/718
        program[PC_START + 7] = 0b0011010100000000; // st offset 256 r2/4 in DATA4/4
        program[PC_START + 8] = 0b1011100100000000; // sti offset 256 in r4/718 DATA5/1 Data6/718
        program[PC_START + 9] = 0b1111000000100101; // halt

        // DATA
        program[PC_START + 5 + 256] = 0b0101010101010101; // DATA1/21845
        program[PC_START + 6 + 256] = 0b0000000000000000; // DATA2/0
        program[PC_START + 8 + 256] = 0b0000000000000001; // DATA5/1
        program[0] = 718; // DATA3/718
        vm.load(program);

        vm.run();

        assert_eq!(vm.registers[&Reg::R0], 7);
        assert_eq!(vm.registers[&Reg::R1], 1);
        assert_eq!(vm.registers[&Reg::R2], 4);
        assert_eq!(vm.registers[&Reg::R7], 4);
        assert_eq!(vm.registers[&Reg::R5], 21845);
        assert_eq!(vm.registers[&Reg::R4], 718);
        assert_eq!(vm.memory.mem[PC_START + 7 + 256], 4);
        assert_eq!(vm.memory.mem[1], 718);
    }
}
