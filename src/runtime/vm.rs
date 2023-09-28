use core::cmp::Ordering;

use super::*;

/// Error states for [`MixVM`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorCode {
    /// A generic error is issued with no details available.
    Generic,

    /// An invalid `C` part is found in current instruction.
    IllegalInstruction,

    /// An access to a non-existent memory address is found.
    InvalidAddress,

    /// An invalid `F` part is found in current instruction.
    InvalidField,

    /// An invalid `I` part is found in current instruction.
    InvalidIndex,

    /// An access to an unknown IO device is found.
    UnknownDevice,

    /// An error is issued by an IO device.
    IOError,

    /// The machine is not running and must be [`MixVM::restart()`]ed.
    Halted,
}

/// Values of the comparison indicator in [`MixVM`].
///
/// Reflects the result of [`CMPA`][Opcode::CmpA] and
/// [`CMPX`][Opcode::CmpX] instructions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompIndicator {
    /// The two operands are equal.
    Equal,

    /// The former operand is less than the latter.
    Less,

    /// The former operand is greater than the latter.
    Greater,

    /// The two floating operands are unordered.
    Unordered,
}

impl Default for CompIndicator {
    /// Get the default value of the comparison indicator.
    fn default() -> Self {
        CompIndicator::Equal
    }
}

/// The state of a MIX machine.
#[repr(C)]
pub struct VM {
    /// The register `rA`.
    pub r_a: FullWord,

    /// The register `rX`.
    pub r_x: FullWord,

    /// The register `rIn`, where `n = 1, 2, 3, 4, 5, 6`.
    /// `r_in[0]` should always used as a source of 0.
    pub r_in: [HalfWord; 7],

    /// The register `rJ`.
    pub r_j: PosHalfWord,

    /// The comparison indicator.
    pub comp: CompIndicator,

    /// The overflow toggle.
    pub overflow: bool,

    /// The machine running state.
    pub halted: bool,

    /// The instruction pointer.
    pub pc: u16,

    /// IO devices.
    pub io_devices: [Option<std::boxed::Box<dyn io::IODevice>>; 21],

    /// The memory.
    pub mem: Mem,
}

impl VM {
    /// Create a new MIX machine.
    pub fn new() -> Self {
        VM {
            r_a: Default::default(),
            r_x: Default::default(),
            r_in: Default::default(),
            r_j: Default::default(),
            comp: Default::default(),
            overflow: false,
            halted: true,
            pc: 0,
            io_devices: Default::default(),
            mem: Mem::new(),
        }
    }

    /// Reset the machine.
    ///
    /// This method resets the machine to its initial state,
    /// clearing the registers.
    pub fn reset(&mut self) {
        self.r_a = Default::default();
        self.r_x = Default::default();
        self.r_in = Default::default();
        self.r_j = Default::default();
        self.pc = 0;
        self.overflow = false;
        self.comp = Default::default();
    }

    /// Restart the machine.
    ///
    /// This function un-halts the machine.
    pub fn restart(&mut self) {
        self.halted = false;
    }

    /// Run the next instruction of the machine.
    ///
    /// # Returns
    /// * [`Ok(())`] - The machine successfully completed its operation.
    /// * [`Err(ErrorCode)`] - The machine encountered an error and is now halted.
    pub fn step(&mut self) -> Result<(), ErrorCode> {
        if self.halted {
            return Err(ErrorCode::Halted);
        }

        // Fetch the instruction.
        let instr: Instruction = self.mem[self.pc].try_into().map_err(|_| {
            self.halt();
            ErrorCode::IllegalInstruction
        })?;

        self.pc += 1;

        // Run the instruction.
        match instr.opcode {
            Opcode::Nop => self.handle_instr_nop(&instr),

            Opcode::Add => self.handle_instr_add_sub(&instr),
            Opcode::Sub => self.handle_instr_add_sub(&instr),
            Opcode::Mul => self.handle_instr_mul(&instr),
            Opcode::Div => self.handle_instr_div(&instr),

            Opcode::Special => self.handle_instr_special(&instr),
            Opcode::Shift => self.handle_instr_shift(&instr),
            Opcode::Move => self.handle_instr_move(&instr),

            Opcode::LdA => self.handle_instr_load_6b(&instr),
            Opcode::Ld1 => self.handle_instr_load_3b(&instr),
            Opcode::Ld2 => self.handle_instr_load_3b(&instr),
            Opcode::Ld3 => self.handle_instr_load_3b(&instr),
            Opcode::Ld4 => self.handle_instr_load_3b(&instr),
            Opcode::Ld5 => self.handle_instr_load_3b(&instr),
            Opcode::Ld6 => self.handle_instr_load_3b(&instr),
            Opcode::LdX => self.handle_instr_load_6b(&instr),

            Opcode::LdAN => self.handle_instr_load_neg_6b(&instr),
            Opcode::Ld1N => self.handle_instr_load_neg_3b(&instr),
            Opcode::Ld2N => self.handle_instr_load_neg_3b(&instr),
            Opcode::Ld3N => self.handle_instr_load_neg_3b(&instr),
            Opcode::Ld4N => self.handle_instr_load_neg_3b(&instr),
            Opcode::Ld5N => self.handle_instr_load_neg_3b(&instr),
            Opcode::Ld6N => self.handle_instr_load_neg_3b(&instr),
            Opcode::LdXN => self.handle_instr_load_neg_6b(&instr),

            Opcode::StA => self.handle_instr_store_6b(&instr),
            Opcode::St1 => self.handle_instr_store_3b(&instr),
            Opcode::St2 => self.handle_instr_store_3b(&instr),
            Opcode::St3 => self.handle_instr_store_3b(&instr),
            Opcode::St4 => self.handle_instr_store_3b(&instr),
            Opcode::St5 => self.handle_instr_store_3b(&instr),
            Opcode::St6 => self.handle_instr_store_3b(&instr),
            Opcode::StX => self.handle_instr_store_6b(&instr),
            Opcode::StJ => self.handle_instr_store_j(&instr),
            Opcode::StZ => self.handle_instr_store_zero(&instr),

            Opcode::Jbus => self.handle_instr_jbus_jred(&instr),
            Opcode::Ioc => self.handle_instr_ioc(&instr),
            Opcode::In => self.handle_instr_in_out(&instr),
            Opcode::Out => self.handle_instr_in_out(&instr),
            Opcode::Jred => self.handle_instr_jbus_jred(&instr),
            Opcode::Jmp => self.handle_instr_jmp(&instr),

            Opcode::JA => self.handle_instr_jmp_reg_6b(&instr),
            Opcode::J1 => self.handle_instr_jmp_reg_3b(&instr),
            Opcode::J2 => self.handle_instr_jmp_reg_3b(&instr),
            Opcode::J3 => self.handle_instr_jmp_reg_3b(&instr),
            Opcode::J4 => self.handle_instr_jmp_reg_3b(&instr),
            Opcode::J5 => self.handle_instr_jmp_reg_3b(&instr),
            Opcode::J6 => self.handle_instr_jmp_reg_3b(&instr),
            Opcode::JX => self.handle_instr_jmp_reg_6b(&instr),

            Opcode::ModifyA => self.handle_instr_modify_6b(&instr),
            Opcode::Modify1 => self.handle_instr_modify_3b(&instr),
            Opcode::Modify2 => self.handle_instr_modify_3b(&instr),
            Opcode::Modify3 => self.handle_instr_modify_3b(&instr),
            Opcode::Modify4 => self.handle_instr_modify_3b(&instr),
            Opcode::Modify5 => self.handle_instr_modify_3b(&instr),
            Opcode::Modify6 => self.handle_instr_modify_3b(&instr),
            Opcode::ModifyX => self.handle_instr_modify_6b(&instr),

            Opcode::CmpA => self.handle_instr_cmp_6b(&instr),
            Opcode::Cmp1 => self.handle_instr_cmp_3b(&instr),
            Opcode::Cmp2 => self.handle_instr_cmp_3b(&instr),
            Opcode::Cmp3 => self.handle_instr_cmp_3b(&instr),
            Opcode::Cmp4 => self.handle_instr_cmp_3b(&instr),
            Opcode::Cmp5 => self.handle_instr_cmp_3b(&instr),
            Opcode::Cmp6 => self.handle_instr_cmp_3b(&instr),
            Opcode::CmpX => self.handle_instr_cmp_6b(&instr),
        }
        .map_err(|err| {
            self.halt();
            err
        })?;

        Ok(())
    }

    /// Halt the machine.
    pub fn halt(&mut self) {
        self.halted = true;
    }

    /// Get indexed address.
    fn helper_get_eff_addr(&self, addr: i16, index: u8) -> Result<u16, ErrorCode> {
        // Direct or indirect addressing.
        // r_in[0] is always zero.
        if !(0..=6).contains(&index) {
            // We have been provided a bad index.
            return Err(ErrorCode::InvalidIndex);
        }
        let reg = self.r_in[index as usize];
        let reg_val = reg.to_i64().0;
        (reg_val + addr as i64)
            .try_into()
            .map_err(|_| ErrorCode::InvalidAddress)
    }

    /// Get indexed address. May panic or return negative value.
    fn helper_get_eff_addr_unchecked(&self, addr: i16, index: u8) -> i16 {
        let reg = self.r_in[index as usize];
        let reg_val = reg.to_i64().0;
        reg_val as i16 + addr
    }

    /// Do actual jump.
    fn helper_do_jump(&mut self, location: u16, save_r_j: bool) {
        if save_r_j {
            let pc = self.pc.to_be_bytes();
            self.r_j[1..=2].copy_from_slice(&pc);
        }
        // Do jump.
        self.pc = location;
    }

    /// Get IO device.
    fn helper_get_io_device(&self, dev_id: usize) -> Result<&dyn IODevice, ErrorCode> {
        let dev = self
            .io_devices
            .get(dev_id)
            .ok_or(ErrorCode::InvalidField)?
            .as_ref()
            .ok_or(ErrorCode::UnknownDevice)?
            .as_ref();
        Ok(dev)
    }

    /// Get IO device.
    fn helper_get_io_device_mut(
        &mut self,
        dev_id: usize,
    ) -> Result<&mut std::boxed::Box<dyn io::IODevice>, ErrorCode> {
        let dev = self
            .io_devices
            .get_mut(dev_id)
            .ok_or(ErrorCode::InvalidField)?
            .as_mut()
            .ok_or(ErrorCode::UnknownDevice)?;
        Ok(dev)
    }

    /// Handler for `NOP`.
    fn handle_instr_nop(&mut self, _: &Instruction) -> Result<(), ErrorCode> {
        // Do nothing.
        Ok(())
    }

    /// Handler for `LDA` and `LDX`.
    fn handle_instr_load_6b(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain everything.
        let (field, sign_copy_needed) = instr.field.to_range_inclusive_signless();
        let mem_cell = &self.mem[self.helper_get_eff_addr(instr.addr, instr.index)?];
        let reg = match instr.opcode {
            Opcode::LdA => &mut self.r_a,
            Opcode::LdX => &mut self.r_x,
            _ => unreachable!(),
        };
        // Zero reg before copying. Handle 'understood' positive sign too.
        reg.set_all([FullWord::POS, 0, 0, 0, 0, 0]);
        // Copy bytes shifted right.
        for (reg_cursor, mem_cursor) in (1..=5).rev().zip(field.rev()) {
            reg[reg_cursor] = mem_cell[mem_cursor];
        }
        // Copy sign byte if needed.
        if sign_copy_needed {
            reg[0] = mem_cell[0];
        }
        Ok(())
    }

    /// Handler for `LDAN` and `LDXN`.
    fn handle_instr_load_neg_6b(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain everything.
        let (field, sign_copy_needed) = instr.field.to_range_inclusive_signless();
        let mem_cell = &self.mem[self.helper_get_eff_addr(instr.addr, instr.index)?];
        let reg = match instr.opcode {
            Opcode::LdAN => &mut self.r_a,
            Opcode::LdXN => &mut self.r_x,
            _ => unreachable!(),
        };
        // Zero reg before copying. Handle 'understood' negative sign.
        reg.set_all([FullWord::POS, 0, 0, 0, 0, 0]);
        // Copy bytes shifted right.
        for (reg_cursor, mem_cursor) in (1..=5).rev().zip(field.rev()) {
            reg[reg_cursor] = mem_cell[mem_cursor];
        }
        // Copy negated sign byte if needed.
        if sign_copy_needed {
            reg[0] = mem_cell[0];
            reg.flip_sign();
        }
        Ok(())
    }

    /// Handler for `LD1-6`.
    ///
    /// Note that this instruction only sets the first sign, 4th
    /// and 5th bits of the original memory location. This prevents
    /// the said 'undefined behavior' from happening.
    fn handle_instr_load_3b(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain everything.
        let (field, sign_copy_needed) = instr.field.to_range_inclusive_signless();
        let mem_cell = &self.mem[self.helper_get_eff_addr(instr.addr, instr.index)?];
        let reg = match instr.opcode {
            Opcode::Ld1 => &mut self.r_in[1],
            Opcode::Ld2 => &mut self.r_in[2],
            Opcode::Ld3 => &mut self.r_in[3],
            Opcode::Ld4 => &mut self.r_in[4],
            Opcode::Ld5 => &mut self.r_in[5],
            Opcode::Ld6 => &mut self.r_in[6],
            _ => unreachable!(),
        };
        // We need to care about only the 4th, 5th and the sign byte.
        // So we make a temporary word and fill back the reg only the
        // 4th, 5th and the sign byte. Handle 'understood' positive sign.
        let mut temp = FullWord::from_bytes([FullWord::POS, 0, 0, 0, 0, 0]);
        // Copy bytes shifted right.
        for (reg_cursor, mem_cursor) in (1..=5).rev().zip(field.rev()) {
            temp[reg_cursor] = mem_cell[mem_cursor];
        }
        // Copy sign byte if needed.
        if sign_copy_needed {
            temp[0] = mem_cell[0];
        }
        // Fill back the reg.
        reg[0] = temp[0];
        reg[1] = temp[4];
        reg[2] = temp[5];
        Ok(())
    }

    /// Handler for `LD1-6N`.
    ///
    /// Note that this instruction only sets the first sign, 4th
    /// and 5th bits of the original memory location. This prevents
    /// the said 'undefined behavior' from happening.
    fn handle_instr_load_neg_3b(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain everything.
        let (field, sign_copy_needed) = instr.field.to_range_inclusive_signless();
        let memory_cell = &self.mem[self.helper_get_eff_addr(instr.addr, instr.index)?];
        let reg = match instr.opcode {
            Opcode::Ld1N => &mut self.r_in[1],
            Opcode::Ld2N => &mut self.r_in[2],
            Opcode::Ld3N => &mut self.r_in[3],
            Opcode::Ld4N => &mut self.r_in[4],
            Opcode::Ld5N => &mut self.r_in[5],
            Opcode::Ld6N => &mut self.r_in[6],
            _ => unreachable!(),
        };
        // We need to care about only the 4th, 5th and the sign byte.
        // So we make a temporary word and fill back the reg only the
        // 4th, 5th and the sign byte. Handle 'understood' positive sign.
        let mut temp = FullWord::from_bytes([FullWord::POS, 0, 0, 0, 0, 0]);
        // Copy bytes shifted right.
        for (reg_cursor, memory_cell_cursor) in (1..=5).rev().zip(field.rev()) {
            temp[reg_cursor] = memory_cell[memory_cell_cursor];
        }
        // Copy negated sign byte if needed.
        if sign_copy_needed {
            temp[0] = memory_cell[0];
            temp.flip_sign();
        }
        // Fill back the reg.
        reg[0] = temp[0];
        reg[1] = temp[4];
        reg[2] = temp[5];
        Ok(())
    }

    /// Handler for `JMP` and variants.
    fn handle_instr_jmp(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        let target_addr = self.helper_get_eff_addr(instr.addr, instr.index)?;
        // Match jump conditions.
        let should_jump = match instr.field {
            0 | 1 => true,
            2 => self.overflow,
            3 => !self.overflow,
            4 => self.comp == CompIndicator::Less,
            5 => self.comp == CompIndicator::Equal,
            6 => self.comp == CompIndicator::Greater,
            7 => self.comp != CompIndicator::Less,
            8 => self.comp != CompIndicator::Equal,
            9 => self.comp != CompIndicator::Greater,
            10 => self.comp != CompIndicator::Unordered,
            11 => self.comp == CompIndicator::Unordered,
            _ => return Err(ErrorCode::InvalidField),
        };
        // Clear overflow flag.
        if instr.field == 2 || instr.field == 3 {
            self.overflow = false;
        }
        if should_jump {
            self.helper_do_jump(target_addr, instr.field != 1);
        }
        Ok(())
    }

    /// Handler for `CHAR`, `NUM` and `HLT`, and various extensions.
    fn handle_instr_special(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        if instr.field == 0 {
            // NUM instruction
            let a_content = &self.r_a[1..=5];
            let x_content = &self.r_x[1..=5];
            let mut result: i64 = 0;
            // For each byte, we extract its 1st position,
            // and push it to `result`.
            for &byte in a_content.iter().chain(x_content) {
                let digit = byte % 10;
                result = result * 10 + digit as i64;
            }
            // Rebuild a word of 4 bytes.
            let result_word = FullWord::from_i64(result).0;
            // We do not modify the sign byte.
            self.r_a[1..=5].copy_from_slice(&result_word[1..=5]);
            Ok(())
        } else if instr.field == 1 {
            // CHAR instruction
            // Obtain original number.
            let mut source = self.r_a.to_i64().0.abs();
            // Extract each digit.
            for reg_i in (0..10).rev() {
                if reg_i >= 5 {
                    self.r_x[reg_i - 5 + 1] = (source % 10 + 30) as u8;
                } else {
                    self.r_a[reg_i + 1] = (source % 10 + 30) as u8;
                }
                source /= 10;
            }
            Ok(())
        } else if instr.field == 2 {
            // HLT instruction
            // Making it just like NOP if we restart the
            // machine later.
            self.halted = true;
            Ok(())
        } else if instr.field >= 3 && instr.field <= 8 {
            if instr.field == 3 {
                // F32CVTF322I4B
                let reg = &mut self.r_a;
                let orig_value = f32::from_be_bytes([reg[2], reg[3], reg[4], reg[5]]);
                reg.set_all([FullWord::POS, 0, 0, 0, 0, 0]);
                reg[0] = if orig_value.is_sign_positive() {
                    FullWord::POS
                } else {
                    FullWord::NEG
                };
                if !orig_value.is_finite()
                    || orig_value > i32::MAX as f32
                    || orig_value < i32::MIN as f32
                {
                    self.overflow = true
                }
                let result = orig_value.abs() as u32;
                reg[2..=5].copy_from_slice(&result.to_be_bytes());
                Ok(())
            } else if instr.field == 4 {
                // F32CVTF322I2B
                let reg = &mut self.r_a;
                let orig_value = f32::from_be_bytes([reg[2], reg[3], reg[4], reg[5]]);
                reg.set_all([FullWord::POS, 0, 0, 0, 0, 0]);
                reg[0] = if orig_value.is_sign_positive() {
                    FullWord::POS
                } else {
                    FullWord::NEG
                };
                if !orig_value.is_finite()
                    || orig_value > i16::MAX as f32
                    || orig_value < i16::MIN as f32
                {
                    self.overflow = true
                }
                let result = orig_value.abs() as u16;
                reg[4..=5].copy_from_slice(&result.to_be_bytes());
                Ok(())
            } else if instr.field == 5 {
                // F32CVTF322I1B
                let reg = &mut self.r_a;
                let orig_value = f32::from_be_bytes([reg[2], reg[3], reg[4], reg[5]]);
                reg.set_all([FullWord::POS, 0, 0, 0, 0, 0]);
                reg[0] = if orig_value.is_sign_positive() {
                    FullWord::POS
                } else {
                    FullWord::NEG
                };
                if !orig_value.is_finite()
                    || orig_value > i8::MAX as f32
                    || orig_value < i8::MIN as f32
                {
                    self.overflow = true
                }
                let result = orig_value.abs() as u8;
                reg[5..=5].copy_from_slice(&result.to_be_bytes());
                Ok(())
            } else {
                let reg = &mut self.r_a;
                let new_value = match instr.field {
                    // F32CVTI4B2F32
                    6 => Ok(u32::from_be_bytes([reg[2], reg[3], reg[4], reg[5]]) as f32),
                    // F32CVTI2B2F32
                    7 => Ok(u16::from_be_bytes([reg[4], reg[5]]) as f32),
                    // F32CVTI1B2F32
                    8 => Ok(u8::from_be_bytes([reg[5]]) as f32),
                    _ => unreachable!(),
                }?;
                reg.set_all([FullWord::POS, 0, 0, 0, 0, 0]);
                reg[2..=5].copy_from_slice(&new_value.to_be_bytes());
                Ok(())
            }
        } else if instr.field >= 9 && instr.field <= 12 {
            if instr.field == 9 {
                // NOT
                let reg = &mut self.r_a;
                reg.flip_sign();
                for i in 1..=5 {
                    reg[i] = !reg[i];
                }
                Ok(())
            } else {
                let addr = self.helper_get_eff_addr(instr.addr, instr.index)?;
                let reg = &mut self.r_a;
                let mem_cell = &mut self.mem[addr];
                let map_fn = match instr.field {
                    // AND
                    10 => |a: u8, b: u8| a & b,
                    // OR
                    11 => |a: u8, b: u8| a | b,
                    // XOR
                    12 => |a: u8, b: u8| a ^ b,
                    _ => unreachable!(),
                };
                for (r, m) in reg[..].iter_mut().zip(mem_cell[..].iter()) {
                    *r = map_fn(*r, *m);
                }
                Ok(())
            }
        } else {
            Err(ErrorCode::InvalidField)
        }
    }

    /// Handler for `STZ`.
    fn handle_instr_store_zero(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain everything.
        let addr = self.helper_get_eff_addr(instr.addr, instr.index)?;
        let field = instr.field.to_range_inclusive();
        let mem_cell = &mut self.mem[addr];
        // Zero the memory cell.
        for i in field {
            if i == 0 {
                // Deal with signs.
                mem_cell[0] = FullWord::POS;
            } else {
                mem_cell[i] = 0;
            }
        }
        Ok(())
    }

    /// Handler for `MOVE`.
    fn handle_instr_move(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain from address.
        let from_addr = self.helper_get_eff_addr(instr.addr, instr.index)?;
        // Obtain to address.
        let to_addr = u16::from_be_bytes([self.r_in[1][1], self.r_in[1][2]]);
        let num_words = instr.field;
        // Move each word.
        for i in 0..num_words {
            let orig_mem = self.mem[from_addr + i as u16];
            self.mem[to_addr + i as u16].clone_from(&orig_mem);
        }
        let new_r_i1_val = self.r_in[1].to_i64().0 + num_words as i64;
        let (new_r_i1, overflow) = HalfWord::from_i64(new_r_i1_val);
        self.r_in[1][..].copy_from_slice(&new_r_i1[..]);
        if overflow {
            self.overflow = overflow;
        }
        Ok(())
    }

    /// Handler for `STA` and `STX`.
    fn handle_instr_store_6b(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain everything.
        let (field, sign_copy_needed) = instr.field.to_range_inclusive_signless();
        let addr = self.helper_get_eff_addr(instr.addr, instr.index)?;
        let mem_cell = &mut self.mem[addr];
        let reg = match instr.opcode {
            Opcode::StA => &self.r_a,
            Opcode::StX => &self.r_x,
            _ => unreachable!(),
        };
        // Copy bytes shifted right.
        for (reg_cursor, mem_cursor) in (1..=5).rev().zip(field.rev()) {
            mem_cell[mem_cursor] = reg[reg_cursor];
        }
        if sign_copy_needed {
            // Copy sign bit.
            mem_cell[0] = reg[0];
        }
        Ok(())
    }

    /// Handler for `ST1-6`.
    fn handle_instr_store_3b(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain everything.
        let (field, sign_copy_needed) = instr.field.to_range_inclusive_signless();
        let addr = self.helper_get_eff_addr(instr.addr, instr.index)?;
        let mem_cell = &mut self.mem[addr];
        let reg = match instr.opcode {
            Opcode::St1 => &self.r_in[1],
            Opcode::St2 => &self.r_in[2],
            Opcode::St3 => &self.r_in[3],
            Opcode::St4 => &self.r_in[4],
            Opcode::St5 => &self.r_in[5],
            Opcode::St6 => &self.r_in[6],
            _ => unreachable!(),
        };
        let padded_reg = [reg[0], 0, 0, 0, reg[1], reg[2]];
        // Copy bytes shifted right.
        for (reg_cursor, mem_cursor) in (1..=5).rev().zip(field.rev()) {
            mem_cell[mem_cursor] = padded_reg[reg_cursor];
        }
        if sign_copy_needed {
            // Copy sign bit.
            mem_cell[0] = padded_reg[0];
        }
        Ok(())
    }

    /// Handler for `STJ`.
    fn handle_instr_store_j(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain everything.
        let (field, sign_copy_needed) = instr.field.to_range_inclusive_signless();
        let addr = self.helper_get_eff_addr(instr.addr, instr.index)?;
        let mem_cell = &mut self.mem[addr];
        let reg = &self.r_j;
        let padded_reg = [reg[0], 0, 0, 0, reg[1], reg[2]];
        // Copy bytes shifted right.
        for (reg_cursor, mem_cursor) in (1..=5).rev().zip(field.rev()) {
            mem_cell[mem_cursor] = padded_reg[reg_cursor];
        }
        if sign_copy_needed {
            // Copy sign bit.
            mem_cell[0] = padded_reg[0];
        }
        Ok(())
    }

    /// Handler for `INCA`, `DECA`, `ENTA`, `ENNA`, `INCX`,
    /// `DECX`, `ENTX` and `ENNX`.
    fn handle_instr_modify_6b(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        let addr = self.helper_get_eff_addr(instr.addr, instr.index)?;
        let reg = match instr.opcode {
            Opcode::ModifyA => &mut self.r_a,
            Opcode::ModifyX => &mut self.r_x,
            _ => unreachable!(),
        };

        if instr.field == 0 || instr.field == 1 {
            // INCx and DECx
            // Add or subtract one.
            let addr = addr as i64;
            let offset = if instr.field == 0 { addr } else { -addr };
            let value = reg.to_i64().0;
            // Convert back modified value.
            let (new_word, overflow) = FullWord::from_i64(value + offset);
            reg.clone_from(&new_word);
            if overflow {
                self.overflow = overflow;
            }
            Ok(())
        } else if instr.field == 2 || instr.field == 3 {
            // ENTx and ENNx
            let new_word = FullWord::from_i64(addr as i64).0;
            // Copy new word into reg.
            reg.clone_from(&new_word);
            if instr.field == 3 {
                reg.flip_sign();
            }
            Ok(())
        } else {
            Err(ErrorCode::InvalidField)
        }
    }

    /// Handler for `INC1-6`, `DEC1-6`, `ENT1-6`, `ENN1-6`.
    fn handle_instr_modify_3b(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        let addr = self.helper_get_eff_addr_unchecked(instr.addr, instr.index);
        let reg = match instr.opcode {
            Opcode::Modify1 => &mut self.r_in[1],
            Opcode::Modify2 => &mut self.r_in[2],
            Opcode::Modify3 => &mut self.r_in[3],
            Opcode::Modify4 => &mut self.r_in[4],
            Opcode::Modify5 => &mut self.r_in[5],
            Opcode::Modify6 => &mut self.r_in[6],
            _ => unreachable!(),
        };

        if instr.field == 0 || instr.field == 1 {
            // INCx and DECx
            // Add or subtract one.
            let addr = addr as i64;
            let offset = if instr.field == 0 { addr } else { -addr };
            let value = reg.to_i64().0;
            // Convert back modified value.
            let (new_word, overflow) = HalfWord::from_i64(value + offset);
            reg.clone_from(&new_word);
            if overflow {
                self.overflow = overflow;
            }
            Ok(())
        } else if instr.field == 2 || instr.field == 3 {
            // ENTx and ENNx
            let new_word = HalfWord::from_i64(addr as i64).0;
            // Copy new word into reg.
            reg.clone_from(&new_word);
            if instr.field == 3 {
                reg.flip_sign();
            }
            Ok(())
        } else {
            Err(ErrorCode::InvalidField)
        }
    }

    /// Handler for `ADD` and `SUB`.
    /// `F32ADD` and `F32SUB` are passed through if enabled.
    fn handle_instr_add_sub(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain V from memory.
        let target_mem = &self.mem[self.helper_get_eff_addr(instr.addr, instr.index)?];

        if instr.field == 7 {
            // F32ADD, F32SUB
            let target_value =
                f32::from_be_bytes([target_mem[2], target_mem[3], target_mem[4], target_mem[5]]);
            let orig_value =
                f32::from_be_bytes([self.r_a[2], self.r_a[3], self.r_a[4], self.r_a[5]]);
            let new_value = match instr.opcode {
                Opcode::Add => orig_value + target_value,
                Opcode::Sub => orig_value - target_value,
                _ => unreachable!(),
            };
            let new_bytes = new_value.to_be_bytes();
            let sign = if new_value.is_sign_positive() {
                FullWord::POS
            } else {
                FullWord::NEG
            };
            self.r_a.set_all([
                sign,
                0,
                new_bytes[0],
                new_bytes[1],
                new_bytes[2],
                new_bytes[3],
            ]);
            if !new_value.is_finite() {
                self.overflow = true;
            }
        } else {
            let orig_value = self.r_a.to_i64().0;
            let target_value = target_mem.to_i64_ranged(instr.field.to_range_inclusive()).0;
            // Calculate and pack new value.
            let new_value = match instr.opcode {
                Opcode::Add => orig_value + target_value,
                Opcode::Sub => orig_value - target_value,
                _ => unreachable!(),
            };
            let (new_word, overflow) = FullWord::from_i64(new_value);
            // Set new value.
            self.r_a.clone_from(&new_word);
            if overflow {
                self.overflow = overflow;
            }
        }

        Ok(())
    }

    /// Handler for `MUL` and `F32MUL`.
    fn handle_instr_mul(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain V from memory.
        let target_mem = &self.mem[self.helper_get_eff_addr(instr.addr, instr.index)?];
        if instr.field == 7 {
            // F32MUL
            let target_value =
                f32::from_be_bytes([target_mem[2], target_mem[3], target_mem[4], target_mem[5]]);
            let orig_value =
                f32::from_be_bytes([self.r_a[2], self.r_a[3], self.r_a[4], self.r_a[5]]);
            let new_value = orig_value * target_value;
            let new_bytes = new_value.to_be_bytes();
            let sign = if new_value.is_sign_positive() {
                FullWord::POS
            } else {
                FullWord::NEG
            };
            self.r_a.set_all([
                sign,
                0,
                new_bytes[0],
                new_bytes[1],
                new_bytes[2],
                new_bytes[3],
            ]);
            if !new_value.is_finite() {
                self.overflow = true;
            }
        } else {
            let orig_value = self.r_a.to_i64().0;
            let target_value = target_mem.to_i64_ranged(instr.field.to_range_inclusive()).0;
            // Copy value into registers.
            let new_val = orig_value as i128 * target_value as i128;
            let new_val_bytes = new_val.abs().to_be_bytes();
            let mut new_val_bytes_dirty = new_val_bytes.map(|b| b != 0);
            for (reg_i, byte_i) in (1..6).rev().zip((0..11).rev()) {
                self.r_a[reg_i] = new_val_bytes[byte_i];
                new_val_bytes_dirty[byte_i] = false;
            }
            for (reg_i, byte_i) in (1..6).rev().zip((11..16).rev()) {
                self.r_x[reg_i] = new_val_bytes[byte_i];
                new_val_bytes_dirty[byte_i] = false;
            }
            // Treat sign.
            let new_sign = if new_val < 0 {
                FullWord::NEG
            } else {
                FullWord::POS
            };
            self.r_a[0] = new_sign;
            self.r_x[0] = new_sign;
            let overflow = new_val_bytes_dirty.iter().any(|b| *b);
            if overflow {
                self.overflow = overflow;
            }
        }
        Ok(())
    }

    /// Handler for `DIV` and `F32DIV`.
    fn handle_instr_div(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        let target_mem = &self.mem[self.helper_get_eff_addr(instr.addr, instr.index)?];
        if instr.field == 7 {
            // F32DIV
            let target_value =
                f32::from_be_bytes([target_mem[2], target_mem[3], target_mem[4], target_mem[5]]);
            let orig_value =
                f32::from_be_bytes([self.r_a[2], self.r_a[3], self.r_a[4], self.r_a[5]]);
            let new_value = orig_value / target_value;
            let new_bytes = new_value.to_be_bytes();
            let sign = if new_value.is_sign_positive() {
                FullWord::POS
            } else {
                FullWord::NEG
            };
            self.r_a.set_all([
                sign,
                0,
                new_bytes[0],
                new_bytes[1],
                new_bytes[2],
                new_bytes[3],
            ]);
            if !new_value.is_finite() {
                self.overflow = true;
            }
        } else {
            let target_value = target_mem.to_i64_ranged(instr.field.to_range_inclusive()).0 as i128;
            let orig_value = i128::from_be_bytes([
                0,
                0,
                0,
                0,
                0,
                0,
                self.r_a[1],
                self.r_a[2],
                self.r_a[3],
                self.r_a[4],
                self.r_a[5],
                self.r_x[1],
                self.r_x[2],
                self.r_x[3],
                self.r_x[4],
                self.r_x[5],
            ]) * self.r_a.get_sign() as i128;
            // Calculate results.
            let quotient: i64 = orig_value
                .checked_div(target_value)
                .unwrap_or_else(|| {
                    self.overflow = true;
                    0
                })
                .abs()
                .try_into()
                .map_err(|_| {
                    self.overflow = true;
                })
                .unwrap_or(0);
            let remainder: i64 = orig_value
                .checked_rem(target_value)
                .unwrap_or_else(|| {
                    self.overflow = true;
                    0
                })
                .abs()
                .try_into()
                .map_err(|_| {
                    self.overflow = true;
                })
                .unwrap_or(0);
            // Copy results into registers.
            let (new_a, overflow_a) = FullWord::from_i64(quotient);
            let (new_x, overflow_x) = FullWord::from_i64(remainder);
            self.r_x[0] = self.r_a[0];
            self.r_a[0] = if orig_value.signum() == target_value.signum() {
                FullWord::POS
            } else {
                FullWord::NEG
            };
            self.r_a[1..=5].copy_from_slice(&new_a[1..=5]);
            self.r_x[1..=5].copy_from_slice(&new_x[1..=5]);
            if overflow_a || overflow_x {
                self.overflow = true;
            }
        }
        Ok(())
    }

    /// Handler for `CMPA` and `CMPX`, `F32CMPA` and `F32CMPX`.
    fn handle_instr_cmp_6b(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain CONTENT(M).
        let target_mem = &self.mem[self.helper_get_eff_addr(instr.addr, instr.index)?];
        let reg = match instr.opcode {
            Opcode::CmpA => &self.r_a,
            Opcode::CmpX => &self.r_x,
            _ => unreachable!(),
        };
        if instr.field == 7 {
            // F32CMPA and F32CMPX
            let target_value =
                f32::from_be_bytes([target_mem[2], target_mem[3], target_mem[4], target_mem[5]]);
            let reg_value = f32::from_be_bytes([reg[2], reg[3], reg[4], reg[5]]);
            self.comp = if reg_value.is_nan() || target_value.is_nan() {
                CompIndicator::Unordered
            } else {
                match reg_value.total_cmp(&target_value) {
                    Ordering::Less => CompIndicator::Less,
                    Ordering::Equal => CompIndicator::Equal,
                    Ordering::Greater => CompIndicator::Greater,
                }
            };
        } else {
            let target_value = target_mem.to_i64_ranged(instr.field.to_range_inclusive()).0;
            let reg_value = reg.to_i64_ranged(instr.field.to_range_inclusive()).0;
            // Calculate and set flags.
            self.comp = match reg_value.cmp(&target_value) {
                Ordering::Equal => CompIndicator::Equal,
                Ordering::Less => CompIndicator::Less,
                Ordering::Greater => CompIndicator::Greater,
            };
        }
        Ok(())
    }

    /// Handler for `CMP1-6`.
    fn handle_instr_cmp_3b(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Obtain CONTENT(M).
        let target_mem = &self.mem[self.helper_get_eff_addr(instr.addr, instr.index)?];
        let target_value = target_mem.to_i64_ranged(instr.field.to_range_inclusive()).0;
        let reg = match instr.opcode {
            Opcode::Cmp1 => &self.r_in[1],
            Opcode::Cmp2 => &self.r_in[2],
            Opcode::Cmp3 => &self.r_in[3],
            Opcode::Cmp4 => &self.r_in[4],
            Opcode::Cmp5 => &self.r_in[5],
            Opcode::Cmp6 => &self.r_in[6],
            _ => unreachable!(),
        };
        let padded_reg = FullWord::from_bytes([reg[0], 0, 0, 0, reg[1], reg[2]]);
        let reg_value = padded_reg.to_i64_ranged(instr.field.to_range_inclusive()).0;
        // Calculate and set flags.
        self.comp =
            if (reg_value == target_value) || (reg_value.abs() == 0 && target_value.abs() == 0) {
                // +0 and -0 are equal.
                CompIndicator::Equal
            } else if reg_value > target_value {
                CompIndicator::Greater
            } else {
                CompIndicator::Less
            };
        Ok(())
    }

    /// Handler for `JA` and `JX`.
    fn handle_instr_jmp_reg_6b(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        let target_addr = self.helper_get_eff_addr(instr.addr, instr.index)?;
        let reg = match instr.opcode {
            Opcode::JA => &self.r_a,
            Opcode::JX => &self.r_x,
            _ => unreachable!(),
        };
        let reg_value = reg.to_i64().0;
        let reg_value_sign = reg_value.signum();
        let is_odd = reg_value & 1 != 0;
        let should_jump = match instr.field {
            0 => reg_value_sign == -1,
            1 => reg_value_sign == 0,
            2 => reg_value_sign == 1,
            3 => reg_value_sign != -1,
            4 => reg_value_sign != 0,
            5 => reg_value_sign != 1,
            6 => !is_odd,
            7 => is_odd,
            _ => return Err(ErrorCode::InvalidField),
        };
        if should_jump {
            self.helper_do_jump(target_addr, true);
        }
        Ok(())
    }

    /// Handler for `J1-6`.
    fn handle_instr_jmp_reg_3b(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        let target_addr = self.helper_get_eff_addr(instr.addr, instr.index)?;
        let reg = match instr.opcode {
            Opcode::J1 => &self.r_in[1],
            Opcode::J2 => &self.r_in[2],
            Opcode::J3 => &self.r_in[3],
            Opcode::J4 => &self.r_in[4],
            Opcode::J5 => &self.r_in[5],
            Opcode::J6 => &self.r_in[6],
            _ => unreachable!(),
        };
        let reg_value_sign = reg.to_i64().0.signum();
        let should_jump = match instr.field {
            0 => reg_value_sign == -1,
            1 => reg_value_sign == 0,
            2 => reg_value_sign == 1,
            3 => reg_value_sign != -1,
            4 => reg_value_sign != 0,
            5 => reg_value_sign != 1,
            _ => return Err(ErrorCode::InvalidField),
        };
        if should_jump {
            self.helper_do_jump(target_addr, true);
        }
        Ok(())
    }

    /// Handler for `SLA`, `SRA`, `SLAX`, `SRAX`, `SLC`, `SRC`, `SLB` and `SRB`.
    fn handle_instr_shift(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        let count = self.helper_get_eff_addr(instr.addr, instr.index)?;
        if instr.field == 0 || instr.field == 1 {
            // SLA and SRA.
            // Spread original register to bytes.
            let orig_value = u64::from_be_bytes(self.r_a.to_i64().0.to_be_bytes());
            // Shift the value in bits (count * 8, count is in bytes).
            let shifted_value = match instr.field {
                0 => orig_value << (count * 8),
                1 => orig_value >> (count * 8),
                _ => unreachable!(),
            };
            // Store back.
            self.r_a[1..=5].copy_from_slice(&shifted_value.to_be_bytes()[3..=7]);
        } else if (instr.field == 2 || instr.field == 3) || (instr.field == 6 || instr.field == 7) {
            // SLAX, SRAX, SLB and SRB.
            // Spread original register to bytes.
            let orig_a_bytes = &self.r_a[1..=5];
            let orig_x_bytes = &self.r_x[1..=5];
            let orig_value = u128::from_be_bytes([
                0,
                0,
                0,
                0,
                0,
                0,
                orig_a_bytes[0],
                orig_a_bytes[1],
                orig_a_bytes[2],
                orig_a_bytes[3],
                orig_a_bytes[4],
                orig_x_bytes[0],
                orig_x_bytes[1],
                orig_x_bytes[2],
                orig_x_bytes[3],
                orig_x_bytes[4],
            ]);
            // Shift.
            let shifted_value = match instr.field {
                2 => orig_value << (count * 8),
                3 => orig_value >> (count * 8),
                6 => orig_value << count,
                7 => orig_value >> count,
                _ => unreachable!(),
            };
            // Store back.
            let shifted_bytes = shifted_value.to_be_bytes();
            self.r_a[1..=5].copy_from_slice(&shifted_bytes[6..=10]);
            self.r_x[1..=5].copy_from_slice(&shifted_bytes[11..=15]);
        } else if instr.field == 4 || instr.field == 5 {
            // SLC and SRC.
            // Spread out bytes.
            let orig_bytes = [
                self.r_a[1],
                self.r_a[2],
                self.r_a[3],
                self.r_a[4],
                self.r_a[5],
                self.r_x[1],
                self.r_x[2],
                self.r_x[3],
                self.r_x[4],
                self.r_x[5],
            ];
            // Zero the registers.
            self.r_a[1..=5].copy_from_slice(&[0; 5]);
            self.r_x[1..=5].copy_from_slice(&[0; 5]);
            // Create cyclic iterator.
            let mut orig_bytes_iter = orig_bytes.iter().cycle();
            // Get shift count.
            let offset = if instr.field == 4 {
                // SLC.
                count % 10
            } else {
                // SRC.
                10 - count % 10
            };
            for _ in 0..offset {
                // Advance the iterator by `offset` steps,
                // to simulate shifting.
                // The iterator is infinite so we don't worry about
                // panics.
                orig_bytes_iter.next().or_else(|| unreachable!());
            }
            // Write back.
            for (reg_i, &digit) in (0..10).zip(orig_bytes_iter) {
                if reg_i >= 5 {
                    self.r_x[reg_i - 5 + 1] = digit;
                } else {
                    self.r_a[reg_i + 1] = digit;
                }
            }
        } else {
            return Err(ErrorCode::InvalidField);
        }
        Ok(())
    }

    /// Handler for `JBUS` and `JRED`.
    fn handle_instr_jbus_jred(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Get device ID.
        let dev_id: usize = instr.field as usize;
        // Get device reference.
        let dev = self.helper_get_io_device(dev_id)?;
        // Call appropriate callbacks.
        let should_jump = match instr.opcode {
            Opcode::Jbus => dev.is_busy().map_err(|_| ErrorCode::IOError)?,
            Opcode::Jred => dev.is_ready().map_err(|_| ErrorCode::IOError)?,
            _ => unreachable!(),
        };
        if should_jump {
            // Do jump.
            let jump_addr = self.helper_get_eff_addr(instr.addr, instr.index)?;
            self.helper_do_jump(jump_addr, true);
        }
        Ok(())
    }

    /// Handler for `IOC`.
    fn handle_instr_ioc(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Get command.
        let command = self.helper_get_eff_addr_unchecked(instr.addr, instr.index);
        // Get device ID.
        let dev_id: usize = instr.field as usize;
        // Get device reference.
        let dev = self.helper_get_io_device_mut(dev_id)?;
        // Call appropriate callbacks.
        dev.control(command).map_err(|_| ErrorCode::IOError)?;
        Ok(())
    }

    /// Handler for `IN` and `OUT`.
    fn handle_instr_in_out(&mut self, instr: &Instruction) -> Result<(), ErrorCode> {
        // Check starting address.
        let addr_start = self.helper_get_eff_addr(instr.addr, instr.index)?;
        if !(0..Mem::SIZE as u16).contains(&addr_start) {
            return Err(ErrorCode::InvalidAddress);
        }
        // Get device ID.
        let dev_id: usize = instr.field as usize;
        // Get device reference.
        let dev = self
            .io_devices
            .get_mut(dev_id)
            .ok_or(ErrorCode::InvalidField)?
            .as_mut()
            .ok_or(ErrorCode::UnknownDevice)?;
        let dev_blk_size = dev.get_block_size();
        // Check ending address.
        let addr_end = addr_start + dev_blk_size as u16;
        if !(0..Mem::SIZE as u16).contains(&addr_end) {
            return Err(ErrorCode::InvalidAddress);
        }
        // Call appropriate callbacks.
        match instr.opcode {
            Opcode::In => {
                let slice = &mut self.mem[addr_start as usize..addr_end as usize];
                dev.read(slice).map_err(|_| ErrorCode::IOError)?;
            }
            Opcode::Out => {
                // Clone words.
                let words = &self.mem[addr_start as usize..addr_end as usize];
                dev.write(words).map_err(|_| ErrorCode::IOError)?;
            }
            _ => unreachable!(),
        };
        Ok(())
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}
