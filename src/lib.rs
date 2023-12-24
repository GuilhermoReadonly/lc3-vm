use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{self, BufRead, StdinLock, Stdout, Write};

pub const PC_START: usize = 0x3000;

trait Instruction<R, W>: Debug
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>);
}

pub struct VM<R, W>
where
    R: BufRead,
    W: Write,
{
    memory: Memory,
    registers: HashMap<Reg, u16>,
    halt: bool,
    reader: R,
    writer: W,
}

impl<R, W> VM<R, W>
where
    R: BufRead,
    W: Write,
{
    pub fn load(&mut self, program: [u16; u16::MAX as usize + 1]) {
        self.memory.load(program)
    }
    pub fn run(&mut self) {
        while !self.halt {
            let current_addr = self.registers[&Reg::Rpc];

            let instruction = self.memory.read(current_addr);

            println!("State: {:#?}", self.registers);

            print!("Instruction: {instruction:016b}.");

            let op: Box<dyn Instruction<R, W>> = instruction.into();

            println!(" Decoded as {op:?}");

            op.execute(self);
        }
    }

    fn inc_rpc(&mut self) {
        let next_addr = self.registers[&Reg::Rpc] + 1;
        self.registers.insert(Reg::Rpc, next_addr);
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

impl Default for VM<StdinLock<'_>, Stdout> {
    fn default() -> Self {
        let stdio = io::stdin();
        let input = stdio.lock();
        let output = io::stdout();
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
            reader: input,
            writer: output,
        }
    }
}

impl Default for VM<&[u8], Vec<u8>> {
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
            reader: b"",
            writer: Vec::default(),
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

impl<R, W> Instruction<R, W> for AddConst
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let result = vm.registers[&self.sr].wrapping_add(self.value);
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
        vm.inc_rpc();
    }
}

#[derive(Debug)]
struct AddReg {
    dr: Reg,
    sr1: Reg,
    sr2: Reg,
}

impl<R, W> Instruction<R, W> for AddReg
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let result = vm.registers[&self.sr1].wrapping_add(vm.registers[&self.sr2]);
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
        vm.inc_rpc();
    }
}

#[derive(Debug, PartialEq)]
struct AndConst {
    dr: Reg,
    sr: Reg,
    value: u16,
}

impl<R, W> Instruction<R, W> for AndConst
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let result = vm.registers[&self.sr] & self.value;
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
        vm.inc_rpc();
    }
}

#[derive(Debug)]
struct AndReg {
    dr: Reg,
    sr1: Reg,
    sr2: Reg,
}

impl<R, W> Instruction<R, W> for AndReg
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let result = vm.registers[&self.sr1] & vm.registers[&self.sr2];
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
        vm.inc_rpc();
    }
}

#[derive(Debug)]
struct Ld {
    dr: Reg,
    offset: u16,
}

impl<R, W> Instruction<R, W> for Ld
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let address = vm.registers[&Reg::Rpc] + self.offset;
        let result = vm.memory.read(address);
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
        vm.inc_rpc();
    }
}

#[derive(Debug)]
struct Ldi {
    dr: Reg,
    offset: u16,
}

impl<R, W> Instruction<R, W> for Ldi
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let address1 = vm.registers[&Reg::Rpc] + self.offset;
        let address2 = vm.memory.read(address1);
        let result = vm.memory.read(address2);
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
        vm.inc_rpc();
    }
}

#[derive(Debug)]
struct Ldr {
    dr: Reg,
    base: Reg,
    offset: u16,
}

impl<R, W> Instruction<R, W> for Ldr
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let address = vm.registers[&self.base] + self.offset;
        let result = vm.memory.read(address);
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
        vm.inc_rpc();
    }
}

#[derive(Debug)]
struct Lea {
    dr: Reg,
    offset: u16,
}

impl<R, W> Instruction<R, W> for Lea
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let address = vm.registers[&Reg::Rpc] + self.offset;
        vm.registers.insert(self.dr, address);
        vm.uf(&self.dr);
        vm.inc_rpc();
    }
}

#[derive(Debug)]
struct St {
    sr: Reg,
    offset: u16,
}

impl<R, W> Instruction<R, W> for St
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let address = vm.registers[&Reg::Rpc] + self.offset;
        let value = vm.registers[&self.sr];
        vm.memory.write(address, value);
        vm.inc_rpc();
    }
}

#[derive(Debug)]
struct Sti {
    sr: Reg,
    offset: u16,
}

impl<R, W> Instruction<R, W> for Sti
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let address1 = vm.registers[&Reg::Rpc] + self.offset;
        let address2 = vm.memory.read(address1);
        let value = vm.registers[&self.sr];
        vm.memory.write(address2, value);
        vm.inc_rpc();
    }
}

#[derive(Debug)]
struct Str {
    sr: Reg,
    base: Reg,
    offset: u16,
}

impl<R, W> Instruction<R, W> for Str
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let address = vm.registers[&self.base] + self.offset;
        let value = vm.registers[&self.sr];
        vm.memory.write(address, value);
        vm.inc_rpc();
    }
}

#[derive(Debug)]
struct Not {
    dr: Reg,
    sr: Reg,
}

impl<R, W> Instruction<R, W> for Not
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let result = !vm.registers[&self.sr];
        vm.registers.insert(self.dr, result);
        vm.uf(&self.dr);
        vm.inc_rpc();
    }
}

#[derive(Debug)]
struct Jmp {
    base: Reg,
}

impl<R, W> Instruction<R, W> for Jmp
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let new_rpc = vm.registers[&self.base];
        vm.registers.insert(Reg::Rpc, new_rpc);
    }
}

#[derive(Debug)]
struct Jsrr {
    base: Reg,
}

impl<R, W> Instruction<R, W> for Jsrr
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let rpc = vm.registers[&Reg::Rpc];
        vm.registers.insert(Reg::R7, rpc);
        let new_rpc = vm.registers[&self.base];
        vm.registers.insert(Reg::Rpc, new_rpc);
    }
}

#[derive(Debug)]
struct Jsr {
    offset: u16,
}

impl<R, W> Instruction<R, W> for Jsr
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let rpc = vm.registers[&Reg::Rpc];
        vm.registers.insert(Reg::R7, rpc);
        let new_rpc = vm.registers[&Reg::Rpc] + self.offset;
        vm.registers.insert(Reg::Rpc, new_rpc);
    }
}

#[derive(Debug)]
struct Br {
    offset: u16,
    nzp: u16,
}

impl<R, W> Instruction<R, W> for Br
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        if self.nzp & vm.registers[&Reg::Rcnd] > 0 {
            let rpc = vm.registers[&Reg::Rpc];
            vm.registers.insert(Reg::Rpc, rpc.wrapping_add(self.offset));
        } else {
            vm.inc_rpc();
        }
    }
}


#[derive(Debug)]
struct TrapGetC;

impl<R, W> Instruction<R, W> for TrapGetC
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let mut buf = [0; 1];
        let _ = vm.reader.read(&mut buf);
        let c = buf[0] as u16;
        vm.registers.insert(Reg::R0, c);
    }
}


#[derive(Debug)]
struct TrapOutC;

impl<R, W> Instruction<R, W> for TrapOutC
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let c = vm.registers[&Reg::R0];
        let _ = vm.writer.write_all(&[c as u8][..]);
    }
}

#[derive(Debug)]
struct TrapPuts;

impl<R, W> Instruction<R, W> for TrapPuts
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let address = vm.registers[&Reg::R0];
        
        let mut c = vm.memory.read(address);
        let mut i = 0;
        while c != 0{
            let buf = vec![c as u8];
            let _ = vm.writer.write_all(buf.as_slice());
            i += 1;
            c = vm.memory.read(address + i);
        }
    }
}

#[derive(Debug)]
struct TrapIn;

impl<R, W> Instruction<R, W> for TrapIn
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let mut buf = [0; 1];
        let _ = vm.reader.read(&mut buf);
        let c = buf[0] as u16;
        vm.registers.insert(Reg::R0, c);
        let _ = vm.writer.write_all(&buf[..]);
    }
}

#[derive(Debug)]
struct TrapHalt;

impl<R, W> Instruction<R, W> for TrapHalt
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
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

    /// get offset 11
    fn poff11(n: u16) -> u16 {
        n & 0x7FF
    }
    /// get offset 6
    fn poff(n: u16) -> u16 {
        n & 0x3F
    }

    /// Sign extend imm5
    fn sextimm(n: u16) -> u16 {
        Reg::sext(Reg::imm(n), 5)
    }

    /// Get the 5th bit as boolean
    fn fimm(instruction: u16) -> bool {
        Reg::get_nth_bit(instruction, 5)
    }

    /// Extract the bits b11, b10, b9
    fn fncd(instruction: u16) -> u16 {
        (instruction >> 9) & 07
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

impl<R, W> From<u16> for Box<dyn Instruction<R, W>>
where
    R: BufRead,
    W: Write,
{
    fn from(instruction: u16) -> Self {
        let opcode = instruction >> 12;
        match opcode {
            0b0000 => {
                let offset = Reg::poff9(instruction);
                let nzp = Reg::fncd(instruction);
                Box::new(Br { offset, nzp })
            }
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
            }
            0b0100 => {
                if Reg::get_nth_bit(instruction, 11) {
                    Box::new(Jsr {
                        offset: Reg::poff11(instruction),
                    })
                } else {
                    Box::new(Jsrr {
                        base: Reg::sr1(instruction),
                    })
                }
            }
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
            0b0110 => {
                let dr = Reg::dr(instruction);
                let offset = Reg::poff(instruction);
                let base = Reg::sr1(instruction);
                Box::new(Ldr { dr, base, offset })
            }
            0b0111 => {
                let sr = Reg::dr(instruction);
                let offset = Reg::poff(instruction);
                let base = Reg::sr1(instruction);
                Box::new(Str { sr, base, offset })
            }
            // 0b1000 => Op::Rti,
            0b1001 => Box::new(Not {
                dr: Reg::dr(instruction),
                sr: Reg::sr1(instruction),
            }),
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
            0b1100 => {
                let base = Reg::sr1(instruction);
                Box::new(Jmp { base })
            }
            // 0b1101 => Op::Unused,
            0b1110 => {
                let dr = Reg::dr(instruction);
                let offset = Reg::poff9(instruction);
                Box::new(Lea { dr, offset })
            }
            0b1111 => {
                let trap_vect = instruction & 0b0000000011111111; 
                match trap_vect{
                    0x20 =>Box::new(TrapGetC),
                    0x21 =>Box::new(TrapOutC),
                    0x22 =>Box::new(TrapPuts),
                    0x23 =>Box::new(TrapIn),
                    // 0x24 =>Box::new(TrapPutsp),
                    0x25 =>Box::new(TrapHalt),
                    // 0x26 =>Box::new(TrapInu16),
                    // 0x27 =>Box::new(TrapOutu16),
                    _ => panic!("Trap vect {trap_vect:016b} as no matching trap"),
                }
            },
            _ => panic!("Op code {instruction:016b} as no matching opcode"),
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

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0001000001000010.into();
        op.execute(&mut vm);
        assert_eq!(vm.registers[&Reg::R0], 0b0000000000000111); // 7
    }

    #[test]
    fn test_exec_add_const() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R3, 0b1111111111110111); // -9

        let op: Box<dyn Instruction<&[u8], Vec<u8>>>= 0b0001000011100111.into();
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R0], 0b1111111111111110); // -2
    }

    #[test]
    fn test_exec_and_reg() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R4, 0b1010101010101010);
        vm.registers.insert(Reg::R5, 0b0101010101010101);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0101000001000010.into();
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R0], 0);
    }

    #[test]
    fn test_exec_and_const() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R6, 0b1010101010101010);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0101000110110101.into();
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R0], 0b1010101010100000);
    }

    #[test]
    fn test_exec_trap_getc() {
        let mut vm = VM::default();
        vm.reader = &[0x41, 0x0A][..];

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1111000000100000.into();
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R0], 0x41); // 0x41 == A
    }

    #[test]
    fn test_exec_trap_outc() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R0, 0x41);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1111000000100001.into();
        op.execute(&mut vm);

        assert_eq!(vm.writer, vec![0x41]);
    }

    #[test]
    fn test_exec_trap_puts() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R0, 718);
        vm.memory.mem[718] = 0x41; // A
        vm.memory.mem[719] = 0x42; // B
        vm.memory.mem[720] = 0x43; // C
        vm.memory.mem[721] = 0x0;

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1111000000100010.into();
        op.execute(&mut vm);

        assert_eq!(vm.writer, vec![0x41, 0x42, 0x43]);
    }

    #[test]
    fn test_exec_trap_in() {
        let mut vm = VM::default();
        vm.reader = &[0x41, 0x0A][..];

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1111000000100011.into();
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R0], 0x41); // 0x41 == A
        assert_eq!(vm.writer, vec![0x41]);
    }

    #[test]
    fn test_run() {
        let mut vm = VM::<&[u8], Vec<u8>>::default();

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
        program[PC_START + 9] = 0b0110110000000001; // ldr base R0/7 offset 1 DATA8/18 in r6/18
        program[PC_START + 10] = 0b0111010100000010; // str base R0/7 offset 2 r6/1 in DATA7/18
        program[PC_START + 11] = 0b1110011100000000; // lea offset 256 in r3/PC_START + 11 + 256
        program[PC_START + 12] = 0b1001101101111111; // not r5/21845 in r5/-21846 = 43690
        program[PC_START + 13] = 0b1100000101000000; // jmp r5/43690
        program[43690] = 0b0100100000000111; // jsr offset 7
        program[43697] = 0b0100000100000000; // jsrr r4/718
        program[718] = 0b0000011111111111; // br false
        program[719] = 0b0000100000000100; // br true offset 4
        program[723] = 0b1111000000100101; // halt

        // DATA
        program[PC_START + 5 + 256] = 0b0101010101010101; // DATA1/21845
        program[PC_START + 6 + 256] = 0b0000000000000000; // DATA2/0
        program[PC_START + 8 + 256] = 0b0000000000000001; // DATA5/1
        program[0] = 718; // DATA3/718
        program[8] = 18; // DATA3/718
        vm.load(program);

        vm.run();

        assert_eq!(vm.registers[&Reg::R0], 7);
        assert_eq!(vm.registers[&Reg::R1], 1);
        assert_eq!(vm.registers[&Reg::R2], 4);
        assert_eq!(vm.registers[&Reg::R7], 43697);
        assert_eq!(vm.registers[&Reg::R5], 43690);
        assert_eq!(vm.registers[&Reg::R4], 718);
        assert_eq!(vm.memory.mem[PC_START + 7 + 256], 4);
        assert_eq!(vm.memory.mem[1], 718);
        assert_eq!(vm.registers[&Reg::R6], 18);
        assert_eq!(vm.registers[&Reg::R3], PC_START as u16 + 11 + 256);
        assert_eq!(vm.memory.mem[8], 18); //Data7/18
    }
}
