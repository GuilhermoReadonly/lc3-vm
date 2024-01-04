use std::fmt::Debug;
use std::io::{BufRead, Write};

use crate::{Reg, VM};

fn imm5(instruction: u16) -> u16 {
    instruction & 0b0000_0000_0001_1111
}

/// sext(n, b) Sign-extend n. The most significant bit of n is replicated as many times as necessary to
// extend n to 16 bits. For example, if n = 110000, then SEXT(n, 6) = 1111 1111 1111 0000
fn sext(n: u16, b: usize) -> u16 {
    if (n >> (b - 1)) & 1 == 1 {
        n | (0xFFFF << b)
    } else {
        n
    }
}

/// get offset 9
fn off9(n: u16) -> u16 {
    n & 0x1FF
}

/// get offset 6
fn off6(n: u16) -> u16 {
    n & 0x3F
}

/// get offset 11
fn off11(n: u16) -> u16 {
    n & 0x7FF
}

/// Extract the bits b11, b10, b9
fn get_cond(instruction: u16) -> u16 {
    (instruction >> 9) & 0x07
}

fn get_nth_bit(instruction: u16, n: usize) -> bool {
    ((instruction >> n) & 1) == 1
}

pub(crate) trait Instruction<R, W>: Debug
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>);
}

impl<R, W> From<u16> for Box<dyn Instruction<R, W>>
where
    R: BufRead,
    W: Write,
{
    fn from(instruction: u16) -> Self {
        let opcode = instruction >> 12;
        match opcode {
            0b0000 => Box::new(Br::from(instruction)),
            0b0001 => {
                if get_nth_bit(instruction, 5) {
                    Box::new(AddConst::from(instruction))
                } else {
                    Box::new(AddReg::from(instruction))
                }
            }
            0b0010 => Box::new(Ld::from(instruction)),
            0b0011 => Box::new(St::from(instruction)),
            0b0100 => {
                if get_nth_bit(instruction, 11) {
                    Box::new(Jsr::from(instruction))
                } else {
                    Box::new(Jsrr::from(instruction))
                }
            }
            0b0101 => {
                if get_nth_bit(instruction, 5) {
                    Box::new(AndConst::from(instruction))
                } else {
                    Box::new(AndReg::from(instruction))
                }
            }
            0b0110 => Box::new(Ldr::from(instruction)),
            0b0111 => Box::new(Str::from(instruction)),
            // 0b1000 => Op::Rti,
            0b1001 => Box::new(Not::from(instruction)),
            0b1010 => Box::new(Ldi::from(instruction)),
            0b1011 => Box::new(Sti::from(instruction)),
            0b1100 => Box::new(Jmp::from(instruction)),
            // 0b1101 => Op::Unused,
            0b1110 => Box::new(Lea::from(instruction)),
            0b1111 => {
                let trap_vect = instruction & 0b0000000011111111;
                match trap_vect {
                    0x20 => Box::new(TrapGetC),
                    0x21 => Box::new(TrapOutC),
                    0x22 => Box::new(TrapPuts),
                    0x23 => Box::new(TrapIn),
                    0x24 => Box::new(TrapPutsp),
                    0x25 => Box::new(TrapHalt),
                    0x26 => Box::new(TrapInu16),
                    0x27 => Box::new(TrapOutu16),
                    _ => panic!("Trap vect {trap_vect:016b} as no matching trap"),
                }
            }
            _ => panic!("Op code {instruction:016b} as no matching opcode"),
        }
    }
}

#[derive(Debug)]
struct AddConst {
    dr: Reg,
    sr: Reg,
    imm5: u16,
}

impl<R, W> Instruction<R, W> for AddConst
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let result = vm.registers[&self.sr].wrapping_add(sext(self.imm5, 5));
        vm.registers.insert(self.dr, result);
        vm.set_nzp(&self.dr);
    }
}

impl From<u16> for AddConst {
    fn from(instruction: u16) -> Self {
        AddConst {
            dr: Reg::dr(instruction),
            sr: Reg::sr1(instruction),
            imm5: imm5(instruction),
        }
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
        vm.set_nzp(&self.dr);
    }
}

impl From<u16> for AddReg {
    fn from(instruction: u16) -> Self {
        AddReg {
            dr: Reg::dr(instruction),
            sr1: Reg::sr1(instruction),
            sr2: Reg::sr2(instruction),
        }
    }
}

#[derive(Debug)]
struct AndConst {
    dr: Reg,
    sr: Reg,
    imm5: u16,
}

impl<R, W> Instruction<R, W> for AndConst
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let result = vm.registers[&self.sr] & sext(self.imm5, 5);
        vm.registers.insert(self.dr, result);
        vm.set_nzp(&self.dr);
    }
}

impl From<u16> for AndConst {
    fn from(instruction: u16) -> Self {
        AndConst {
            dr: Reg::dr(instruction),
            sr: Reg::sr1(instruction),
            imm5: imm5(instruction),
        }
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
        vm.set_nzp(&self.dr);
    }
}

impl From<u16> for AndReg {
    fn from(instruction: u16) -> Self {
        AndReg {
            dr: Reg::dr(instruction),
            sr1: Reg::sr1(instruction),
            sr2: Reg::sr2(instruction),
        }
    }
}

#[derive(Debug)]
struct Ld {
    dr: Reg,
    offset9: u16,
}

impl<R, W> Instruction<R, W> for Ld
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let rpc = vm.get_rpc();
        let address = rpc.wrapping_add(sext(self.offset9, 9));
        let result = vm.memory.read(address);
        vm.registers.insert(self.dr, result);
        vm.set_nzp(&self.dr);
    }
}

impl From<u16> for Ld {
    fn from(instruction: u16) -> Self {
        Ld {
            dr: Reg::dr(instruction),
            offset9: off9(instruction),
        }
    }
}

#[derive(Debug)]
struct Ldi {
    dr: Reg,
    offset9: u16,
}

impl<R, W> Instruction<R, W> for Ldi
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let rpc = vm.get_rpc();
        let address1 = rpc.wrapping_add(sext(self.offset9, 9));
        let address2 = vm.memory.read(address1);
        let result = vm.memory.read(address2);
        vm.registers.insert(self.dr, result);
        vm.set_nzp(&self.dr);
    }
}

impl From<u16> for Ldi {
    fn from(instruction: u16) -> Self {
        Ldi {
            dr: Reg::dr(instruction),
            offset9: off9(instruction),
        }
    }
}

#[derive(Debug)]
struct Ldr {
    dr: Reg,
    base: Reg,
    offset6: u16,
}

impl<R, W> Instruction<R, W> for Ldr
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let address = vm.registers[&self.base].wrapping_add(sext(self.offset6, 6));
        let result = vm.memory.read(address);
        vm.registers.insert(self.dr, result);
        vm.set_nzp(&self.dr);
    }
}

impl From<u16> for Ldr {
    fn from(instruction: u16) -> Self {
        Ldr {
            dr: Reg::dr(instruction),
            base: Reg::sr1(instruction),
            offset6: off6(instruction),
        }
    }
}

#[derive(Debug)]
struct Lea {
    dr: Reg,
    offset9: u16,
}

impl<R, W> Instruction<R, W> for Lea
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let rpc = vm.get_rpc();
        let address = rpc.wrapping_add(sext(self.offset9, 9));
        vm.registers.insert(self.dr, address);
        vm.set_nzp(&self.dr);
    }
}

impl From<u16> for Lea {
    fn from(instruction: u16) -> Self {
        let dr = Reg::dr(instruction);
        let offset9 = off9(instruction);
        Lea { dr, offset9 }
    }
}

#[derive(Debug)]
struct St {
    sr: Reg,
    offset9: u16,
}

impl<R, W> Instruction<R, W> for St
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let rpc = vm.get_rpc();
        let address = rpc.wrapping_add(sext(self.offset9, 9));
        let value = vm.registers[&self.sr];
        vm.memory.write(address, value);
    }
}

impl From<u16> for St {
    fn from(instruction: u16) -> Self {
        let sr = Reg::dr(instruction);
        let offset9 = off9(instruction);
        St { sr, offset9 }
    }
}

#[derive(Debug)]
struct Sti {
    sr: Reg,
    offset9: u16,
}

impl<R, W> Instruction<R, W> for Sti
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let rpc = vm.get_rpc();
        let address1 = rpc.wrapping_add(sext(self.offset9, 9));
        let address2 = vm.memory.read(address1);
        let value = vm.registers[&self.sr];
        vm.memory.write(address2, value);
    }
}

impl From<u16> for Sti {
    fn from(instruction: u16) -> Self {
        let sr = Reg::dr(instruction);
        let offset9 = off9(instruction);
        Sti { sr, offset9 }
    }
}

#[derive(Debug)]
struct Str {
    sr: Reg,
    base: Reg,
    offset6: u16,
}

impl<R, W> Instruction<R, W> for Str
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let address = vm.registers[&self.base].wrapping_add(sext(self.offset6, 6));
        let value = vm.registers[&self.sr];
        vm.memory.write(address, value);
    }
}

impl From<u16> for Str {
    fn from(instruction: u16) -> Self {
        let sr = Reg::dr(instruction);
        let base = Reg::sr1(instruction);
        let offset6 = off6(instruction);
        Str { sr, base, offset6 }
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
        vm.set_nzp(&self.dr);
    }
}

impl From<u16> for Not {
    fn from(instruction: u16) -> Self {
        let dr = Reg::dr(instruction);
        let sr = Reg::sr1(instruction);
        Not { dr, sr }
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
        vm.registers.insert(Reg::RPC, new_rpc);
    }
}

impl From<u16> for Jmp {
    fn from(instruction: u16) -> Self {
        let base = Reg::sr1(instruction);
        Jmp { base }
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
        let rpc = vm.get_rpc();
        vm.registers.insert(Reg::R7, rpc);
        let new_rpc = vm.registers[&self.base];
        vm.registers.insert(Reg::RPC, new_rpc);
    }
}

impl From<u16> for Jsrr {
    fn from(instruction: u16) -> Self {
        let base = Reg::sr1(instruction);
        Jsrr { base }
    }
}

#[derive(Debug)]
struct Jsr {
    offset11: u16,
}

impl<R, W> Instruction<R, W> for Jsr
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let rpc = vm.get_rpc();
        vm.registers.insert(Reg::R7, rpc);
        let new_rpc = rpc.wrapping_add(sext(self.offset11, 11));
        vm.registers.insert(Reg::RPC, new_rpc);
    }
}

impl From<u16> for Jsr {
    fn from(instruction: u16) -> Self {
        let offset11 = off11(instruction);
        Jsr { offset11 }
    }
}

#[derive(Debug)]
struct Br {
    offset9: u16,
    nzp: u16,
}

impl<R, W> Instruction<R, W> for Br
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let rpc = vm.get_rpc();
        if self.nzp & vm.registers[&Reg::RCond] > 0 {
            vm.registers
                .insert(Reg::RPC, rpc.wrapping_add(sext(self.offset9, 9)));
        }
    }
}

impl From<u16> for Br {
    fn from(instruction: u16) -> Self {
        let offset9 = off9(instruction);
        let nzp = get_cond(instruction);
        Br { offset9, nzp }
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
        let rpc = vm.get_rpc();
        vm.registers.insert(Reg::R7, rpc);

        let mut buf = [0; 1];
        vm.reader.read(&mut buf).expect("read");
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
        let rpc = vm.get_rpc();
        vm.registers.insert(Reg::R7, rpc);

        let c = vm.registers[&Reg::R0];
        vm.writer.write_all(&[c as u8][..]).expect("write_all");
        vm.writer.flush().expect("Writer flushed");
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
        let rpc = vm.get_rpc();
        vm.registers.insert(Reg::R7, rpc);

        let address = vm.registers[&Reg::R0];

        let mut c = vm.memory.read(address);
        let mut i = 0;
        while c != 0 {
            vm.writer.write_all(&[c as u8][..]).expect("write_all");
            i += 1;
            c = vm.memory.read(address + i);
        }
        vm.writer.flush().expect("Writer flushed");
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
        let rpc = vm.get_rpc();
        vm.registers.insert(Reg::R7, rpc);

        let mut buf: [u8; 1] = [0; 1];
        vm.reader.read(&mut buf).expect("read");
        let c = buf[0] as u16;
        vm.registers.insert(Reg::R0, c);
        vm.writer.write_all(&[c as u8][..]).expect("write_all");
        vm.writer.flush().expect("Writer flushed");
    }
}

#[derive(Debug)]
struct TrapPutsp;

impl<R, W> Instruction<R, W> for TrapPutsp
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let rpc = vm.get_rpc();
        vm.registers.insert(Reg::R7, rpc);

        let address = vm.registers[&Reg::R0];

        let mut c = vm.memory.read(address);
        let mut i = 0;
        while c != 0 {
            let num1: u8 = (c >> 8) as u8;
            let num2: u8 = (0b0000000011111111 & c) as u8;
            vm.writer.write_all(&[num1, num2][..]).expect("write_all");

            i += 1;
            c = vm.memory.read(address + i);
        }
        vm.writer.flush().expect("Writer flushed");
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

#[derive(Debug)]
struct TrapInu16;

impl<R, W> Instruction<R, W> for TrapInu16
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let rpc = vm.get_rpc();
        vm.registers.insert(Reg::R7, rpc);

        let mut buf: [u8; 1] = [0; 1];
        let mut all_characters = String::from("");
        let mut character: u8 = 0;
        while character != 0x0A {
            // 0x0A: Enter
            vm.reader.read(&mut buf).expect("read");
            character = buf[0];
            if character.is_ascii_digit() {
                all_characters.push(character as char);
            }
        }

        let number: u16 = u16::from_str_radix(&all_characters, 10).expect("u16 conversion failed");
        vm.registers.insert(Reg::R0, number);
    }
}

#[derive(Debug)]
struct TrapOutu16;

impl<R, W> Instruction<R, W> for TrapOutu16
where
    R: BufRead,
    W: Write,
{
    fn execute(&self, vm: &mut VM<R, W>) {
        let rpc = vm.get_rpc();
        vm.registers.insert(Reg::R7, rpc);

        let c = vm.registers[&Reg::R0];
        let c_string = c.to_string();
        for character in c_string.as_bytes() {
            vm.writer.write_all(&[*character][..]).expect("write_all");
        }
        vm.writer.flush().expect("Writer flushed");
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

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0001_000_001_0_00_010.into();
        op.execute(&mut vm);
        assert_eq!(vm.registers[&Reg::R0], 0b0000000000000111); // 7
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_add_const() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R3, 0b1111_1111_1111_0111); // -9

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0001_000_011_1_00111.into(); // Add R3 + 7
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R0], 0b1111_1111_1111_1110); // -2
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_and_reg() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R4, 0b1010101010101010);
        vm.registers.insert(Reg::R5, 0b0101010101010101);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0101000001000010.into();
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R0], 0);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_and_const() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R6, 0b1010101010101010);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0101_000_110_110101.into(); // AndConst Dr=R0 Sr=R6 const=110101
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R0], 0b1010101010100000);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_ld() {
        let mut vm = VM::default();
        vm.memory.write(0x2FFF, 718);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0010_110_111111111.into(); // Ld Dr=R6 offset=-1
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R6], 718);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_ldi() {
        let mut vm = VM::default();
        vm.memory.write(0x2FFF, 7);
        vm.memory.write(7, 18);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1010_101_111111111.into(); // Ldi Dr=R5 offset=-1
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R5], 18);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_ldr() {
        let mut vm = VM::default();
        vm.memory.write(0xFFFF, 718);
        vm.registers.insert(Reg::R7, 0xFFFE);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0110_010_111_000001.into(); // Ldr Dr=R2 baseR=R7 offset=1
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R2], 718);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_lea() {
        let mut vm = VM::default();

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1110_011_111111111.into(); // Lea Dr=R3 offset=-1
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R3], 0x2FFF);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_not() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R1, 0xF0F0);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1001_000_001_111111.into(); // Not Dr=R0 Sr=R1
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R0], 0x0F0F);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_st() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R2, 718);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0011_010_111111111.into(); // St Sr=R2 offset=-1
        op.execute(&mut vm);

        assert_eq!(vm.memory.read(0x2FFF), 718);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_sti() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R3, 718);
        vm.memory.write(0x2FFF, 0xFFFF);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1011_011_111111111.into(); // Sti Sr=R3 offset=-1
        op.execute(&mut vm);

        assert_eq!(vm.memory.read(0xFFFF), 718);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_str() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R4, 718);
        vm.registers.insert(Reg::R5, 0xFF00);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0111_100_101_111111.into(); // Str Sr=R4 BaseR=R5 offset=-1
        op.execute(&mut vm);

        assert_eq!(vm.memory.read(0xFEFF), 718);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_jmp() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R6, 0xFF00);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1100_000_110_000000.into(); // Jmp BaseR=R6
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::RPC], 0xFF00);
    }

    #[test]
    fn test_exec_jsrr() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R0, 0xFF00);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0100_0_00_000_000000.into(); // JsrR BaseR=R0
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::RPC], 0xFF00);
        assert_eq!(vm.registers[&Reg::R7], 0x3000);
    }

    #[test]
    fn test_exec_jsr() {
        let mut vm = VM::default();

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0100_1_11111111111.into(); // Jsr offset=-1
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::RPC], 0x3000 - 1);
        assert_eq!(vm.registers[&Reg::R7], 0x3000);
    }

    #[test]
    fn test_exec_br() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::RCond, 0b0000000000000100);
        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0000_100_111111111.into(); // BrN offset=-1
        op.execute(&mut vm);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000 - 1);

        let mut vm = VM::default();
        vm.registers.insert(Reg::RCond, 0b0000000000000100);
        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0000_011_111111111.into(); // BrN offset=-1
        op.execute(&mut vm);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);

        let mut vm = VM::default();
        vm.registers.insert(Reg::RCond, 0b0000000000000010);
        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0000_010_111111111.into(); // BrZ offset=-1
        op.execute(&mut vm);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000 - 1);

        let mut vm = VM::default();
        vm.registers.insert(Reg::RCond, 0b0000000000000010);
        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0000_101_111111111.into(); // BrZ offset=-1
        op.execute(&mut vm);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);

        let mut vm = VM::default();
        vm.registers.insert(Reg::RCond, 0b0000000000000001);
        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0000_001_111111111.into(); // BrP offset=-1
        op.execute(&mut vm);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000 - 1);

        let mut vm = VM::default();
        vm.registers.insert(Reg::RCond, 0b0000000000000001);
        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b0000_110_111111111.into(); // BrP offset=-1
        op.execute(&mut vm);
        assert_eq!(vm.registers[&Reg::RPC], 0x3000);
    }

    #[test]
    fn test_exec_trap_getc() {
        let mut vm = VM::default();
        vm.reader = &[0x41, 0x0A][..];

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1111000000100000.into();
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R0], 0x41); // 0x41 == A
        assert_eq!(vm.registers[&Reg::R7], 0x3000);
    }

    #[test]
    fn test_exec_trap_outc() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R0, 0x41);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1111000000100001.into();
        op.execute(&mut vm);

        assert_eq!(vm.writer, vec![0x41]);
        assert_eq!(vm.registers[&Reg::R7], 0x3000);
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
        assert_eq!(vm.registers[&Reg::R7], 0x3000);
    }

    #[test]
    fn test_exec_trap_in() {
        let mut vm = VM::default();
        vm.reader = &[0x41, 0x0A][..];

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1111000000100011.into();
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R0], 0x41); // 0x41 == A
        assert_eq!(vm.writer, vec![0x41]);
        assert_eq!(vm.registers[&Reg::R7], 0x3000);
    }

    #[test]
    fn test_exec_trap_in_u16() {
        let mut vm = VM::default();
        vm.reader = &[0x32, 0x35, 0x35, 0x0A][..]; // 255 Enter

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1111000000100110.into();
        op.execute(&mut vm);

        assert_eq!(vm.registers[&Reg::R0], 255); // R0 contains 255
        assert_eq!(vm.registers[&Reg::R7], 0x3000);
    }

    #[test]
    fn test_exec_trap_out_u16() {
        let mut vm = VM::default();
        vm.registers.insert(Reg::R0, 255);

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1111000000100111.into();
        op.execute(&mut vm);

        assert_eq!(vm.writer, vec![b'2', b'5', b'5']);
        assert_eq!(vm.registers[&Reg::R7], 0x3000);
    }

    #[test]
    fn test_exec_trap_putsp() {
        let mut vm = VM::default();

        vm.registers.insert(Reg::R0, 718);
        vm.memory.mem[718] = 0x4142; // AB
        vm.memory.mem[719] = 0x4344; // CD
        vm.memory.mem[721] = 0x0;

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1111000000100100.into();
        op.execute(&mut vm);

        assert_eq!(vm.writer, vec![0x41, 0x42, 0x43, 0x44]);
        assert_eq!(vm.registers[&Reg::R7], 0x3000);
    }

    #[test]
    fn test_exec_trap_halt() {
        let mut vm = VM::default();

        let op: Box<dyn Instruction<&[u8], Vec<u8>>> = 0b1111000000100101.into();
        op.execute(&mut vm);

        assert_eq!(vm.halt, true);
    }
}
