use std::collections::HashMap;

pub const PC_START: usize = 0x3000;

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
            self.registers.insert(Reg::Rpc, next_addr) ;
            let instruction = self.memory.read(current_addr);

            print!("Instruction: {instruction:016b}.");

            let op: Op = instruction.into();

            println!(" Decoded as op: {op:?}");

            self.exec(op);
        }
    }

    fn exec(&mut self, op: Op) {
        match op {
            Op::Add {
                dr,
                sr1,
                variant: Add::AddReg(sr2),
            } => self.add_reg(dr, sr1, sr2),
            Op::Add {
                dr,
                sr1,
                variant: Add::AddConst(value),
            } => self.add_const(dr, sr1, value),
            Op::Trap(Trap::Halt) => self.trap_halt(),
            _ => todo!(),
        }
    }

    fn add_reg(&mut self, dr: Reg, sr1: Reg, sr2: Reg) {
        let result = self.registers[&sr1] + self.registers[&sr2];
        self.registers.insert(dr, result);
    }

    fn add_const(&mut self, dr: Reg, sr1: Reg, value: u16) {
        let result = self.registers[&sr1] + value;
        self.registers.insert(dr, result);
    }

    fn trap_halt(&mut self) {
        self.halt = true;
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

    fn _write(&mut self, address: u16, val: u16) -> () {
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
enum Op {
    Br,
    Add { dr: Reg, sr1: Reg, variant: Add },
    Ld,
    St,
    Jsr,
    And,
    Ldr,
    Str,
    Rti,
    Not,
    Ldi,
    Sti,
    Jmp,
    Unused,
    Lea,
    Trap(Trap),
}

#[derive(Debug, PartialEq)]
enum Add {
    AddConst(u16),
    AddReg(Reg),
}
#[derive(Debug, PartialEq)]
enum Trap {
    Halt,
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
        let imm = instruction & 0b0000000000011111;
        if (imm >> (5 - 1)) & 1 == 1 {
            imm | (0xFFFF << 5)
        } else {
            imm
        }
    }
}

fn get_nth_bit(value: u16, n: usize) -> bool {
    ((value >> n) & 1) == 1
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

impl From<u16> for Op {
    fn from(instruction: u16) -> Self {
        let opcode = instruction >> 12;
        match opcode {
            0b0000 => Op::Br,
            0b0001 => {
                if get_nth_bit(instruction, 5) {
                    Op::Add {
                        dr: Reg::dr(instruction),
                        sr1: Reg::sr1(instruction),
                        variant: Add::AddConst(Reg::imm(instruction)),
                    }
                } else {
                    Op::Add {
                        dr: Reg::dr(instruction),
                        sr1: Reg::sr1(instruction),
                        variant: Add::AddReg(Reg::sr2(instruction)),
                    }
                }
            }
            0b0010 => Op::Ld,
            0b0011 => Op::St,
            0b0100 => Op::Jsr,
            0b0101 => Op::And,
            0b0110 => Op::Ldr,
            0b0111 => Op::Str,
            0b1000 => Op::Rti,
            0b1001 => Op::Not,
            0b1010 => Op::Ldi,
            0b1011 => Op::Sti,
            0b1100 => Op::Jmp,
            0b1101 => Op::Unused,
            0b1110 => Op::Lea,
            0b1111 => Op::Trap(Trap::Halt),
            _ => panic!("Op code {instruction} as no matching opcode"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_from() {
        assert_eq!(Into::<Op>::into(0b1111111111111111), Op::Trap(Trap::Halt));
        assert_eq!(Into::<Op>::into(0b1001100110010010), Op::Not);
        assert_eq!(Into::<Op>::into(0b0100001010011100), Op::Jsr);
        assert_eq!(Into::<Op>::into(0b0000010110001110), Op::Br);
    }

    #[test]
    fn test_imm() {
        assert_eq!(Reg::imm(0b1010101010110001), 0b1111111111110001);
    }

    #[test]
    fn test_exec_add() {
        let mut vm = VM::default();

        vm.registers.insert(Reg::R1, 0b0000000000000100); // 4
        vm.registers.insert(Reg::R2, 0b0000000000000011); // 3

        vm.exec(Op::Add {
            dr: Reg::R0,
            sr1: Reg::R1,
            variant: Add::AddReg(Reg::R2),
        });

        assert_eq!(vm.registers[&Reg::R0], 0b0000000000000111); // 7

        vm.registers.insert(Reg::R1, 0b1111111111111100); // -4
        vm.registers.insert(Reg::R2, 0b0000000000000011); // 3

        vm.exec(Op::Add {
            dr: Reg::R0,
            sr1: Reg::R1,
            variant: Add::AddReg(Reg::R2),
        });

        assert_eq!(vm.registers[&Reg::R0], 0b1111111111111111); // -1
    }

    #[test]
    fn test_run() {
        let mut vm = VM::default();

        let mut program = [0; u16::MAX as usize + 1];
        program[PC_START + 0] = 0b0001001001100011; // add r1 and 3 in r1
        program[PC_START + 1] = 0b0001010010100100; // add r2 and 4 in r2
        program[PC_START + 2] = 0b0001000001000010; // add r1 and r2 in r0
        program[PC_START + 3] = 0b1111000000100101; // halt
        vm.load(program);

        vm.run();

        assert_eq!(vm.registers[&Reg::R0], 7);

    }
}
