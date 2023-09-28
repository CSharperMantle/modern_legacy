#![deny(clippy::all)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::result_unit_err)]
#![allow(dead_code)]

use modern_legacy_macros::alphabet_str;

mod runtime;
use runtime::*;

use std::io::{self, Read, Write};

struct LinePrinterDevice {
    lower_case: bool,
}

impl IODevice for LinePrinterDevice {
    fn read(&mut self, _: &mut [FullWord]) -> Result<(), ()> {
        Err(())
    }

    fn write(&mut self, data: &[FullWord]) -> Result<(), usize> {
        if data.len() != self.get_block_size() {
            return Err(0);
        }

        let mut count_written: usize = 0;
        // For each word...
        for word in data {
            // Each byte in a word...
            for &byte in &word[1..=5] {
                // Convert to char.
                let ch: char = Alphabet::try_from(byte)
                    .map_err(|_| count_written)?
                    .try_into()
                    .map_err(|_| count_written)?;
                if self.lower_case && ch.is_ascii_uppercase() {
                    print!("{}", ch.to_ascii_lowercase());
                } else {
                    print!("{}", ch);
                }
                count_written += 1;
            }
        }
        println!();
        Ok(())
    }

    fn control(&mut self, command: i16) -> Result<(), ()> {
        match command {
            0 => {
                self.lower_case = false;
                Ok(())
            }
            1 => {
                self.lower_case = true;
                Ok(())
            }
            2 => io::stdout().flush().map_err(|_| ()),
            _ => Err(()),
        }
    }

    fn is_busy(&self) -> Result<bool, ()> {
        Ok(false)
    }

    fn is_ready(&self) -> Result<bool, ()> {
        Ok(true)
    }

    fn get_block_size(&self) -> usize {
        8
    }
}

struct LineReaderDevice {}

impl IODevice for LineReaderDevice {
    fn read(&mut self, buffer: &mut [FullWord]) -> Result<(), ()> {
        if buffer.len() != self.get_block_size() {
            return Err(());
        }
        let mut buf = [0; 5];
        io::stdin().read_exact(&mut buf).map_err(|_| ())?;
        let chars: [u8; 5] = buf.map(|b| {
            u8::try_from(
                Alphabet::try_from(char::from_u32(b as u32).unwrap_or('‚'))
                    .unwrap_or(Alphabet::LowSQuote),
            )
            .unwrap_or(0xFFu8)
        });
        buffer[0][0] = FullWord::POS;
        buffer[0][1..=5].copy_from_slice(&chars);
        Ok(())
    }

    fn write(&mut self, _: &[FullWord]) -> Result<(), usize> {
        Err(0)
    }

    fn control(&mut self, _: i16) -> Result<(), ()> {
        Err(())
    }

    fn is_busy(&self) -> Result<bool, ()> {
        Ok(false)
    }

    fn is_ready(&self) -> Result<bool, ()> {
        Ok(true)
    }

    fn get_block_size(&self) -> usize {
        1
    }
}

const INPUT_WORDS: i16 = 7;

const PROGRAM_START: i16 = 0;
const CONST_START: i16 = 3000;
const TEMP_START: i16 = 3100;
const IO_START: i16 = 3200;

const LOC_XTEA: i16 = PROGRAM_START;
const LOC_XTEA_LOOP: i16 = LOC_XTEA + 4;
const LOC_XTEA_RET: i16 = LOC_XTEA + 79;

const LOC_MAIN: i16 = PROGRAM_START + 80;
const LOC_MAIN_O_W: i16 = LOC_MAIN + 3;
const LOC_MAIN_O_L: i16 = LOC_MAIN + 5;
const LOC_MAIN_O_P: i16 = LOC_MAIN + 6;
const LOC_MAIN_I_L: i16 = LOC_MAIN + 14;
const LOC_MAIN_I_R: i16 = LOC_MAIN + 15;
const LOC_MAIN_ENC: i16 = LOC_MAIN + 20;
const LOC_MAIN_VERIF_LOOP: i16 = LOC_MAIN + 26;
const LOC_MAIN_VERIF_CONT: i16 = LOC_MAIN + 30;
const LOC_MAIN_VERIF_N: i16 = LOC_MAIN + 35;
const LOC_MAIN_END: i16 = LOC_MAIN + 41;

const LOC_CONST_EQ3: i16 = CONST_START;
const LOC_CONST_DELTA: i16 = LOC_CONST_EQ3 + 1;
const LOC_CONST_WELCOME: i16 = LOC_CONST_DELTA + 1;
const LOC_CONST_ENC_LOOPS: i16 = LOC_CONST_WELCOME + 16;
const LOC_CONST_C: i16 = LOC_CONST_ENC_LOOPS + 1;
const LOC_CONST_WRONG: i16 = LOC_CONST_C + INPUT_WORDS;
const LOC_CONST_RIGHT: i16 = LOC_CONST_WRONG + 8;

const LOC_TMP_SUM: i16 = TEMP_START;
const LOC_TMP_I: i16 = TEMP_START + 1;
const LOC_TMP_1: i16 = TEMP_START + 2;
const LOC_TMP_2: i16 = TEMP_START + 3;
const LOC_TMP_3: i16 = TEMP_START + 4;
const LOC_TMP_4: i16 = TEMP_START + 5;
const LOC_TMP_5: i16 = TEMP_START + 6;
const LOC_TMP_6: i16 = TEMP_START + 7;
const LOC_TMP_7: i16 = TEMP_START + 8;
const LOC_TMP_8: i16 = TEMP_START + 9;
const LOC_TMP_9: i16 = TEMP_START + 10;

const LOC_ARG_V: i16 = IO_START;
const LOC_ARG_K: i16 = LOC_XTEA + 76;

const PROGRAM: [Instruction; 124] = [
    Instruction::new(LOC_XTEA_RET, 2, 0, Opcode::StJ),
    Instruction::new(32, 2, 0, Opcode::ModifyX),
    Instruction::new(0, 2, 0, Opcode::ModifyA),
    Instruction::new(LOC_TMP_SUM, 13, 0, Opcode::StA),
    Instruction::new(LOC_TMP_I, 5, 0, Opcode::StX),
    Instruction::new(0, 2, 0, Opcode::ModifyX),
    Instruction::new(1, 0, 0, Opcode::Modify1),
    Instruction::new(LOC_ARG_V, 13, 1, Opcode::LdA),
    Instruction::new(4, 6, 0, Opcode::Shift),
    Instruction::new(LOC_TMP_1, 13, 0, Opcode::StA),
    Instruction::new(LOC_ARG_V, 13, 1, Opcode::LdA),
    Instruction::new(5, 7, 0, Opcode::Shift),
    Instruction::new(LOC_TMP_2, 13, 0, Opcode::StA),
    Instruction::new(0, 2, 0, Opcode::ModifyX),
    Instruction::new(0, 2, 0, Opcode::ModifyA),
    Instruction::new(LOC_TMP_1, 13, 0, Opcode::LdA),
    Instruction::new(LOC_TMP_2, 12, 0, Opcode::Special),
    Instruction::new(LOC_TMP_3, 13, 0, Opcode::StA),
    Instruction::new(LOC_ARG_V, 13, 1, Opcode::LdA),
    Instruction::new(LOC_TMP_3, 13, 0, Opcode::Add),
    Instruction::new(LOC_TMP_4, 13, 0, Opcode::StA),
    Instruction::new(LOC_TMP_SUM, 13, 0, Opcode::LdA),
    Instruction::new(LOC_CONST_EQ3, 10, 0, Opcode::Special),
    Instruction::new(LOC_TMP_5, 45, 0, Opcode::StA),
    Instruction::new(LOC_TMP_5, 45, 0, Opcode::Ld2),
    Instruction::new(LOC_ARG_K, 13, 2, Opcode::LdA),
    Instruction::new(LOC_TMP_6, 13, 0, Opcode::StA),
    Instruction::new(LOC_TMP_6, 13, 0, Opcode::LdA),
    Instruction::new(LOC_TMP_SUM, 13, 0, Opcode::Add),
    Instruction::new(LOC_TMP_7, 13, 0, Opcode::StA),
    Instruction::new(LOC_TMP_7, 13, 0, Opcode::LdA),
    Instruction::new(LOC_TMP_4, 12, 0, Opcode::Special),
    Instruction::new(LOC_TMP_8, 13, 0, Opcode::StA),
    Instruction::new(1, 1, 0, Opcode::Modify1),
    Instruction::new(LOC_ARG_V, 13, 1, Opcode::LdA),
    Instruction::new(LOC_TMP_8, 13, 0, Opcode::Add),
    Instruction::new(LOC_ARG_V, 13, 1, Opcode::StA),
    Instruction::new(LOC_TMP_SUM, 13, 0, Opcode::LdA),
    Instruction::new(LOC_CONST_DELTA, 13, 0, Opcode::Add),
    Instruction::new(LOC_TMP_SUM, 13, 0, Opcode::StA),
    Instruction::new(LOC_ARG_V, 13, 1, Opcode::LdA),
    Instruction::new(4, 6, 0, Opcode::Shift),
    Instruction::new(LOC_TMP_1, 13, 0, Opcode::StA),
    Instruction::new(LOC_ARG_V, 13, 1, Opcode::LdA),
    Instruction::new(5, 7, 0, Opcode::Shift),
    Instruction::new(LOC_TMP_2, 13, 0, Opcode::StA),
    Instruction::new(0, 2, 0, Opcode::ModifyX),
    Instruction::new(0, 2, 0, Opcode::ModifyA),
    Instruction::new(LOC_TMP_1, 13, 0, Opcode::LdA),
    Instruction::new(LOC_TMP_2, 12, 0, Opcode::Special),
    Instruction::new(LOC_TMP_3, 13, 0, Opcode::StA),
    Instruction::new(LOC_ARG_V, 13, 1, Opcode::LdA),
    Instruction::new(LOC_TMP_3, 13, 0, Opcode::Add),
    Instruction::new(LOC_TMP_4, 13, 0, Opcode::StA),
    Instruction::new(LOC_TMP_SUM, 13, 0, Opcode::LdA),
    Instruction::new(11, 7, 0, Opcode::Shift),
    Instruction::new(LOC_TMP_5, 13, 0, Opcode::StA),
    Instruction::new(0, 2, 0, Opcode::ModifyX),
    Instruction::new(0, 2, 0, Opcode::ModifyA),
    Instruction::new(LOC_TMP_5, 13, 0, Opcode::LdA),
    Instruction::new(LOC_CONST_EQ3, 10, 0, Opcode::Special),
    Instruction::new(LOC_TMP_6, 45, 0, Opcode::StA),
    Instruction::new(LOC_TMP_6, 45, 0, Opcode::Ld2),
    Instruction::new(LOC_ARG_K, 13, 2, Opcode::LdA),
    Instruction::new(LOC_TMP_7, 13, 0, Opcode::StA),
    Instruction::new(LOC_TMP_7, 13, 0, Opcode::LdA),
    Instruction::new(LOC_TMP_SUM, 13, 0, Opcode::Add),
    Instruction::new(LOC_TMP_8, 13, 0, Opcode::StA),
    Instruction::new(LOC_TMP_8, 13, 0, Opcode::LdA),
    Instruction::new(LOC_TMP_4, 12, 0, Opcode::Special),
    Instruction::new(LOC_TMP_9, 13, 0, Opcode::StA),
    Instruction::new(1, 0, 0, Opcode::Modify1),
    Instruction::new(LOC_ARG_V, 13, 1, Opcode::LdA),
    Instruction::new(LOC_TMP_9, 13, 0, Opcode::Add),
    Instruction::new(LOC_ARG_V, 13, 1, Opcode::StA),
    Instruction::new(1, 1, 0, Opcode::Modify1),
    Instruction::new(LOC_TMP_I, 5, 0, Opcode::LdX),
    Instruction::new(1, 1, 0, Opcode::ModifyX),
    Instruction::new(LOC_XTEA_LOOP, 2, 0, Opcode::JX),
    Instruction::new(3999, 0, 0, Opcode::Jmp),
    Instruction::new(0, 2, 0, Opcode::ModifyA),
    Instruction::new(2, 2, 0, Opcode::ModifyX),
    Instruction::new(0, 2, 0, Opcode::Modify3),
    Instruction::new(0, 18, 0, Opcode::Ioc),
    Instruction::new(LOC_MAIN_O_W, 18, 0, Opcode::Jbus),
    Instruction::new(LOC_CONST_WELCOME, 18, 3, Opcode::Out),
    Instruction::new(0x4433, 0x22, 0x11, Opcode::Nop),
    Instruction::new(LOC_MAIN_O_P, 18, 0, Opcode::Jbus),
    Instruction::new(8, 0, 0, Opcode::Modify3),
    Instruction::new(1, 1, 0, Opcode::ModifyX),
    Instruction::new(LOC_MAIN_O_L, 2, 0, Opcode::JX),
    Instruction::new(2, 18, 0, Opcode::Ioc),
    Instruction::new(INPUT_WORDS, 2, 0, Opcode::ModifyX),
    Instruction::new(0, 2, 0, Opcode::Modify4),
    Instruction::new(LOC_ARG_V, 19, 4, Opcode::In),
    Instruction::new(LOC_MAIN_I_R, 19, 0, Opcode::Jbus),
    Instruction::new(1, 0, 0, Opcode::Modify4),
    Instruction::new(1, 1, 0, Opcode::ModifyX),
    Instruction::new(LOC_MAIN_I_L, 2, 0, Opcode::JX),
    Instruction::new(0, 2, 0, Opcode::Modify1),
    Instruction::new(LOC_XTEA, 0, 0, Opcode::Jmp),
    Instruction::new(1, 0, 0, Opcode::Modify1),
    Instruction::new(LOC_CONST_ENC_LOOPS, 5, 0, Opcode::Cmp1),
    Instruction::new(LOC_MAIN_ENC, 4, 0, Opcode::Jmp),
    Instruction::new(INPUT_WORDS, 2, 0, Opcode::ModifyX),
    Instruction::new(INPUT_WORDS - 1, 2, 0, Opcode::Modify2),
    Instruction::new(LOC_ARG_V, 13, 2, Opcode::LdA),
    Instruction::new(LOC_CONST_C, 12, 2, Opcode::Special),
    Instruction::new(LOC_MAIN_VERIF_CONT, 4, 0, Opcode::JA),
    Instruction::new(1, 1, 0, Opcode::ModifyX),
    Instruction::new(1, 1, 0, Opcode::Modify2),
    Instruction::new(LOC_MAIN_VERIF_LOOP, 3, 0, Opcode::J2),
    Instruction::new(2560, 2, 0, Opcode::ModifyA),
    Instruction::new(LOC_TMP_1, 5, 0, Opcode::StX),
    Instruction::new(LOC_MAIN_VERIF_N, 2, 0, Opcode::Jmp),
    Instruction::new(1, 2, 0, Opcode::ModifyX),
    Instruction::new(0, 2, 0, Opcode::ModifyA),
    Instruction::new(LOC_TMP_1, 5, 0, Opcode::Div),
    Instruction::new(0, 2, 0, Opcode::Modify1),
    Instruction::new(LOC_MAIN_END, 3, 0, Opcode::Jmp),
    Instruction::new(8, 0, 0, Opcode::Modify1),
    Instruction::new(LOC_CONST_WRONG, 18, 1, Opcode::Out),
    Instruction::new(2, 18, 0, Opcode::Ioc),
    Instruction::new(0, 2, 0, Opcode::Special),
];

const WELCOME: [[u8; 6]; 16] = alphabet_str!(
    "\
EXPL0RE 1960S' PAST 1N 4 PRESENT W0RLD  \
WHAT DID YOU UNCOVER, ELITE RUSTACEAN >>\
"
);

const CIPHER: [[u8; 6]; INPUT_WORDS as usize] = [
    [0, 5, 139, 14, 94, 218],
    [0, 244, 138, 250, 182, 187],
    [0, 244, 123, 251, 140, 191],
    [0, 95, 176, 194, 183, 102],
    [0, 138, 101, 40, 247, 89],
    [0, 122, 206, 163, 121, 181],
    [0, 192, 133, 13, 8, 206],
];

const WRONG: [[u8; 6]; 8] = alphabet_str!(
    "\
THAT IS NOT CORRECT. TRY AGAIN :D       \
"
);

const RIGHT: [[u8; 6]; 8] = alphabet_str!(
    "\
NOW MARCH BEYOND, AND REVIVE THE LEGACY.\
"
);

fn main() {
    let mut mix = VM::new();
    mix.reset();

    for (i, instr) in PROGRAM.iter().enumerate() {
        mix.mem[PROGRAM_START as u16 + i as u16] = FullWord::from(*instr);
    }

    mix.mem[LOC_CONST_EQ3 as u16].set_all([0, 0, 0, 0, 0, 3]);
    mix.mem[LOC_CONST_DELTA as u16].set_all([0, 0x9e, 0x38, 0x53, 0x8a, 0x49]);
    for (i, bytes) in WELCOME.iter().enumerate() {
        mix.mem[LOC_CONST_WELCOME as u16 + i as u16].set_all(*bytes);
    }
    for (i, bytes) in CIPHER.iter().enumerate() {
        mix.mem[LOC_CONST_C as u16 + i as u16].set_all(*bytes);
    }
    for (i, bytes) in WRONG.iter().enumerate() {
        mix.mem[LOC_CONST_WRONG as u16 + i as u16].set_all(*bytes);
    }
    for (i, bytes) in RIGHT.iter().enumerate() {
        mix.mem[LOC_CONST_RIGHT as u16 + i as u16].set_all(*bytes);
    }
    mix.mem[LOC_CONST_ENC_LOOPS as u16] = FullWord::from_i64(INPUT_WORDS as i64 - 1).0;

    mix.io_devices[18] = Some(Box::new(LinePrinterDevice { lower_case: false }));
    mix.io_devices[19] = Some(Box::new(LineReaderDevice {}));

    mix.pc = LOC_MAIN as u16;

    mix.restart();

    while !mix.halted {
        mix.step().unwrap();
    }

    // Flag: D3CTF(TECH-EV0LVE,EMBR@C3-PR0GR3SS)

    if cfg!(feature = "csmantle") {
        println!("--- Input after XTEA");
        for i in 0..INPUT_WORDS {
            println!("{:?}", mix.mem[LOC_ARG_V as u16 + i as u16]);
        }
        println!("--- Expected cipher");
        for i in 0..INPUT_WORDS {
            println!("{:?}", mix.mem[LOC_CONST_C as u16 + i as u16]);
        }
        println!("--- Key");
        println!("{:?}", mix.mem[LOC_ARG_K as u16 /* + 0 */]);
        println!("{:?}", mix.mem[LOC_ARG_K as u16 + 1]);
        println!("{:?}", mix.mem[LOC_ARG_K as u16 + 2]);
        println!("{:?}", mix.mem[LOC_ARG_K as u16 + 3]);
    }
}
