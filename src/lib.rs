use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{self, Read, Stdout, Write};

pub const PC_START: usize = 0x3000;
const MR_KBSR: u16 = 0xFE00;
const MR_KBDR: u16 = 0xFE02;

mod instructions;
pub mod unsafe_zone;
use instructions::*;

pub struct LibCReader;

impl Read for LibCReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let c_u8 = unsafe_zone::get_char();
        if buf.len() == 0 {
            return Ok(0);
        }
        match c_u8 {
            0 => Ok(0),
            c => {
                buf[0] = c;
                Ok(1)
            }
        }
    }
}

pub struct VM<R, W>
where
    R: Read,
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
    R: Read,
    W: Write,
{
    pub fn load<P>(&mut self, mut program: P)
    where
        P: Read,
    {
        let mut buf = [0; 2];
        let mut read_result = program.read_exact(&mut buf);

        let mut base_address = buf[1] as u16 | (buf[0] as u16) << 8;
        self.registers.insert(Reg::RPC, base_address);

        while read_result.is_ok() {
            read_result = program.read_exact(&mut buf);

            let instruction = buf[1] as u16 | (buf[0] as u16) << 8;
            self.memory.write(base_address, instruction);
            base_address += 1;
        }
    }

    pub fn run(&mut self) {
        let mut _i_count: u128 = 0;

        while !self.halt {
            let current_addr = self.registers[&Reg::RPC];
            let instruction = self.memory.read(current_addr);

            self.inc_rpc();

            let op: Box<dyn Instruction<R, W>> = instruction.into();

            // println!("State: {:#?}", self.registers);
            // print!("({i_count}) Instruction {current_addr:04x}: {instruction:016b}/{instruction:04x}.");
            // println!(" Decoded as {op:?}");

            op.execute(self);
            _i_count += 1;

            // if i_count % 100_000_000 == 0 {
            //     println!("{i_count} instructions executed.");
            // }
        }
        // println!("{i_count} instructions executed.");
    }

    fn inc_rpc(&mut self) -> u16 {
        let next_addr = self.registers[&Reg::RPC] + 1;
        self.registers.insert(Reg::RPC, next_addr);
        next_addr
    }

    fn get_rpc(&self) -> u16 {
        self.registers[&Reg::RPC]
    }
    fn set_nzp(&mut self, r: &Reg) {
        if self.registers[r] == 0 {
            self.registers.insert(Reg::RCond, 1 << 1);
        } else if self.registers[r] >> 15 == 1 {
            self.registers.insert(Reg::RCond, 1 << 2);
        } else {
            self.registers.insert(Reg::RCond, 1 << 0);
        }
    }
}

impl Default for VM<LibCReader, Stdout> {
    fn default() -> Self {
        let input = LibCReader;
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
                (Reg::RCond, 1 << 1),
                (Reg::RPC, PC_START as u16),
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
                (Reg::RCond, 1 << 1),
                (Reg::RPC, PC_START as u16),
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

fn get_key() -> Option<u16> {
    match unsafe_zone::get_char() {
        0 => None,
        c => Some(c as u16),
    }
}

impl Memory {
    fn read(&mut self, address: u16) -> u16 {
        if address == MR_KBSR {
            let key = get_key();
            match key {
                Some(c) => {
                    self.write(MR_KBSR, 1 << 15);
                    self.write(MR_KBDR, c);
                }
                None => self.write(MR_KBSR, 0x0),
            }
        }
        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, val: u16) -> () {
        self.mem[address as usize] = val;
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            mem: [0; u16::MAX as usize + 1],
        }
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
    RPC,
    RCond,
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

#[cfg(test)]
mod tests {

    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_load_and_run() {
        let mut vm = VM::<&[u8], Vec<u8>>::default();

        let program: &[u16] = &[
            0x3000,             // start = 0x3000; // 00110000 00000000
            0b0001001001100011, // add r1/0 and 3 in r1/3
            0b0001010010100100, // add r2/0 and 4 in r2/4
            0b0001000001000010, // add r1/3 and r2/4 in r0/7
            0b0101001001100001, // and r1/3 and 1 in r1/1
            0b0101111000000010, // and r0/7 and r2/4 in r7/4
            0b0010101000000011, // ld offset 3 DATA/718 in r5/718
            0b1111000000100101, // halt
            0,
            0b0000001011001110, // DATA/718
        ];

        let mut res: [u8; 20] = [0; 20];
        for i in 0..program.len() {
            res[i * 2] = (program[i] >> 8) as u8;
            res[i * 2 + 1] = (program[i] & 0x00FF) as u8;
        }

        let reader = BufReader::new(res.as_slice());

        vm.load(reader);

        vm.run();

        assert_eq!(vm.registers[&Reg::R0], 7);
        assert_eq!(vm.registers[&Reg::R1], 1);
        assert_eq!(vm.registers[&Reg::R2], 4);
        assert_eq!(vm.registers[&Reg::R3], 0);
        assert_eq!(vm.registers[&Reg::R4], 0);
        assert_eq!(vm.registers[&Reg::R5], 718);
        assert_eq!(vm.registers[&Reg::R6], 0);
        assert_eq!(vm.registers[&Reg::R7], 4);
    }
}
