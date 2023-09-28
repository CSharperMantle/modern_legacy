use core::ops::RangeInclusive;

use super::mem::FullWord;

/// An instruction in [`MixVM`].
///
/// Instructions are represented in [`FullWord`]s,
/// thus it can be converted from such type after validation.
///
/// [`MixVM`]: crate::MixVM
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Instruction {
    /// The signed address, `A`, read big-endian.
    pub addr: i16,

    /// The field, `F`.
    pub field: u8,

    /// The index, `I`.
    pub index: u8,

    /// The operation code, `C`.
    ///
    /// See [`Opcode`] for a list of supported operations.
    pub opcode: Opcode,
}

impl Instruction {
    /// Create a new instruction.
    ///
    /// # Arguments
    /// * `addr` - The signed address, `A`, read big-endian.
    /// * `field` - The field, `F`.
    /// * `index` - The index, `I`.
    /// * `opcode` - The operation code, `C`.
    pub const fn new(addr: i16, field: u8, index: u8, opcode: Opcode) -> Self {
        Instruction {
            addr,
            field,
            index,
            opcode,
        }
    }
}

impl TryFrom<FullWord> for Instruction {
    type Error = ();

    /// Convert a [`FullWord`] to an [`Instruction`].
    ///
    /// # Arguments
    /// * `source` - The [`FullWord`] to convert.
    ///
    /// # Returns
    /// * [`Ok(Instruction)`] - The conversion was successful.
    /// * [`Err(())`] - The conversion failed.
    fn try_from(source: FullWord) -> Result<Self, Self::Error> {
        let sign = source.get_sign() as i16;
        let addr = sign * i16::from_be_bytes([source[1], source[2]]);
        let opcode = Opcode::try_from(source[5..=5][0]).map_err(|_| ())?;
        Ok(Instruction {
            opcode,
            field: source[4..=4][0],
            index: source[3..=3][0],
            addr,
        })
    }
}

/// Operation codes in [`MixVM`].
///
/// In MIX literature, an opcode is represented in the form
/// of `OP(F)`, where `OP` is the mnemonic and `F` is the `F`
/// field associated with this instruction. One opcode could map
/// to multiple operations, using `F` to distinguish among.
///
/// [`MixVM`]: crate::MixVM
#[derive(Clone, Copy, PartialEq, Eq, Debug, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    /// * `NOP(0)` - No operation.
    Nop = 0,

    /// * `ADD(0:5)` - Integer addition.
    /// * `F32ADD(7)` (`x-ieee754`) - IEEE 754 `binary32` addition.
    ///
    /// ```text
    /// rA <- rA + V
    /// ```
    Add = 1,

    /// * `SUB(0:5)` - Integer subtraction.
    /// * `F32SUB(7)` (`x-ieee754`) - IEEE 754 `binary32` subtraction.
    ///
    /// ```text
    /// rA <- rA - V
    /// ```
    Sub = 2,

    /// * `MUL(0:5)` - Integer multiplication.
    ///
    /// ```text
    /// rAX <- rA * V
    /// ```
    ///
    /// ------
    ///
    /// * `F32MUL(7)` (`x-ieee754`) - IEEE 754 `binary32` multiplication.
    ///
    /// ```text
    /// rA <- rA * V
    /// ```
    Mul = 3,

    /// * `DIV(0:5)` - Integer division.
    ///
    /// ```text
    /// rA <- rAX / V; rX <- rAX % V
    /// ```
    ///
    /// ------
    ///
    /// * `F32DIV(7)` (`x-ieee754`) - IEEE 754 `binary32` division.
    ///
    /// ```text
    /// rA <- rA / V
    /// ```
    Div = 4,

    /// * `NUM(0)` - Char to number.
    /// * `CHAR(1)` - Number to char.
    /// * `HLT(2)` - Halt.
    ///
    /// ------
    ///
    /// * `F32CVTF322I4B(3)` (`x-ieee754`) - Convert and round IEEE 754 `binary32` to 4-bytes integer.
    /// * `F32CVTF322I2B(4)` (`x-ieee754`) - Convert and round IEEE 754 `binary32` to 2-bytes integer.
    /// * `F32CVTF322I1B(5)` (`x-ieee754`) - Convert and round IEEE 754 `binary32` to 1-byte integer.
    /// * `F32CVTI4B2F32(6)` (`x-ieee754`) - Convert 4-bytes integer to IEEE 754 `binary32`.
    /// * `F32CVTI2B2F32(7)` (`x-ieee754`) - Convert 2-bytes integer to IEEE 754 `binary32`.
    /// * `F32CVTI1B2F32(8)` (`x-ieee754`) - Convert 1-byte integer to IEEE 754 `binary32`.
    ///
    /// ```text
    /// rA <- convert(rA)
    /// ```
    /// 
    /// ------
    /// 
    /// * `NOT(9)` (`x-binarith`): Perform bitwise NOT on `rA`, then store result in `rA`.
    /// 
    /// ```text
    /// rA <- !rA
    /// ```
    /// 
    /// * `AND(10)` (`x-binarith`): Perform bitwise AND on `V` and `rA`, then store result in `rA`.
    /// * `OR(11)` (`x-binarith`): Perform bitwise OR on `V` and `rA`, then store result in `rA`.
    /// * `XOR(12)` (`x-binarith`): Perform bitwise XOR on `V` and `rA`, then store result in `rA`.
    /// 
    /// ```text
    /// rA <- rA OP V
    /// ```
    Special = 5,

    /// * `SLA(0)` - Shift left `rA`.
    /// * `SRA(1)` - Shift right `rA`.
    /// * `SLAX(2)` - Shift left `rAX`.
    /// * `SRAX(3)` - Shift right `rAX`.
    /// * `SLC(4)` - Shift circularly left `rAX`.
    /// * `SRC(5)` - Shift circularly right `rAX`.
    /// * `SLB(6)` (`x-binary`) - Shift left `rAX` binary.
    /// * `SRB(7)` (`x-binary`) - Shift right `rAX` binary.
    Shift = 6,

    /// * `MOVE(1)` - Move `F` words from `M` to `rI1`.
    Move = 7,

    /// * `LDA(0:5)` - Load `rA`.
    ///
    /// ```text
    /// rA <- V
    /// ```
    LdA = 8,

    /// * `LD1(0:5)` - Load `rI1`.
    ///
    /// ```text
    /// rI1 <- V
    /// ```
    Ld1 = 9,

    /// * `LD2(0:5)` - Load `rI2`.
    ///
    /// ```text
    /// rI2 <- V
    /// ```
    Ld2 = 10,

    /// * `LD3(0:5)` - Load `rI3`.
    ///
    /// ```text
    /// rI3 <- V
    /// ```
    Ld3 = 11,

    /// * `LD4(0:5)` - Load `rI4`.
    ///
    /// ```text
    /// rI4 <- V
    /// ```
    Ld4 = 12,

    /// * `LD5(0:5)` - Load `rI5`.
    ///
    /// ```text
    /// rI5 <- V
    /// ```
    Ld5 = 13,

    /// * `LD6(0:5)` - Load `rI6`.
    ///
    /// ```text
    /// rI6 <- V
    /// ```
    Ld6 = 14,

    /// * `LDX(0:5)` - Load `rX`.
    ///
    /// ```text
    /// rX <- V
    /// ```
    LdX = 15,

    /// * `LDAN(0:5)` - Load `rA` negative.
    ///
    /// ```text
    /// rA <- -V
    /// ```
    LdAN = 16,

    /// * `LD1N(0:5)` - Load `rI1` negative.
    ///
    /// ```text
    /// rI1 <- -V
    /// ```
    Ld1N = 17,

    /// * `LD2N(0:5)` - Load `rI2` negative.
    ///
    /// ```text
    /// rI2 <- -V
    /// ```
    Ld2N = 18,

    /// * `LD3N(0:5)` - Load `rI3` negative.
    ///
    /// ```text
    /// rI3 <- -V
    /// ```
    Ld3N = 19,

    /// * `LD4N(0:5)` - Load `rI4` negative.
    ///
    /// ```text
    /// rI4 <- -V
    /// ```
    Ld4N = 20,

    /// * `LD5N(0:5)` - Load `rI5` negative.
    ///
    /// ```text
    /// rI5 <- -V
    /// ```
    Ld5N = 21,

    /// * `LD6N(0:5)` - Load `rI6` negative.
    ///
    /// ```text
    /// rI6 <- -V
    /// ```
    Ld6N = 22,

    /// * `LDXN(0:5)` - Load `rX` negative.
    ///
    /// ```text
    /// rX <- -V
    /// ```
    LdXN = 23,

    /// * `STA(0:5)` - Store `rA`.
    ///
    /// ```text
    /// M(F) <- rA
    /// ```
    StA = 24,

    /// * `ST1(0:5)` - Store `rI1`.
    ///
    /// ```text
    /// M(F) <- rI1
    /// ```
    St1 = 25,

    /// * `ST2(0:5)` - Store `rI2`.
    ///
    /// ```text
    /// M(F) <- rI2
    /// ```
    St2 = 26,

    /// * `ST3(0:5)` - Store `rI3`.
    ///
    /// ```text
    /// M(F) <- rI3
    /// ```
    St3 = 27,

    /// * `ST4(0:5)` - Store `rI4`.
    ///
    /// ```text
    /// M(F) <- rI4
    /// ```
    St4 = 28,

    /// * `ST5(0:5)` - Store `rI5`.
    ///
    /// ```text
    /// M(F) <- rI5
    /// ```
    St5 = 29,

    /// * `ST6(0:5)` - Store `rI6`.
    ///
    /// ```text
    /// M(F) <- rI6
    /// ```
    St6 = 30,

    /// * `STX(0:5)` - Store `rX`.
    ///
    /// ```text
    /// M(F) <- rX
    /// ```
    StX = 31,

    /// * `STJ(0:2)` - Store `rJ`.
    ///
    /// ```text
    /// M(F) <- rJ
    /// ```
    StJ = 32,

    /// * `STZ(0:5)` - Store `0`.
    ///
    /// ```text
    /// M(F) <- 0
    /// ```
    StZ = 33,

    /// * `JBUS(0)` - Jump if unit `F` busy.
    Jbus = 34,

    /// * `IOC(0)` - Control unit `F`.
    Ioc = 35,

    /// * `IN(0)` - Input from unit `F`.
    In = 36,

    /// * `OUT(0)` - Output to unit `F`.
    Out = 37,

    /// * `JRED(0)` - Jump if unit `F` ready.
    Jred = 38,

    /// * `JMP(0)` - Jump to `M`.
    /// * `JSJ(1)` - Jump to `M` without changing `rJ`.
    /// * `JOV(2)` - Jump on overflow.
    /// * `JNOV(3)` - Jump on no overflow.
    /// * `JL(4)` - Jump on less.
    /// * `JE(5)` - Jump on equal.
    /// * `JG(6)` - Jump on greater.
    /// * `JGE(7)` - Jump on greater-or-equal.
    /// * `JNE(8)` - Jump on not equal.
    /// * `JLE(9)` - Jump on less-or-equal.
    ///
    /// ------
    ///
    /// * `F32JORD(10)` (`x-ieee754`) - Jump on ordered.
    /// * `F32JUNORD(11)` (`x-ieee754`) - Jump on unordered.
    Jmp = 39,

    /// * `JAN(0)`
    /// * `JAZ(1)`
    /// * `JAP(2)`
    /// * `JANN(3)`
    /// * `JANZ(4)`
    /// * `JANP(5)`
    ///
    /// ```text
    /// rA : 0; jump
    /// ```
    ///
    /// ------
    ///
    /// * `JAE(6)` (`x-binary`) - Jump `rA` even
    /// * `JAO(7)` (`x-binary`) - Jump `rA` odd
    ///
    /// ```text
    /// rA & 1 : 0; jump
    /// ```
    ///
    /// ------
    ///
    /// See also [`Opcode::Jmp`].
    JA = 40,

    /// * `J1N(0)`
    /// * `J1Z(1)`
    /// * `J1P(2)`
    /// * `J1NN(3)`
    /// * `J1NZ(4)`
    /// * `J1NP(5)`
    ///
    /// ```text
    /// rI1 : 0; jump
    /// ```
    ///
    /// ------
    ///
    /// See also [`Opcode::Jmp`].
    J1 = 41,

    /// * `J2N(0)`
    /// * `J2Z(1)`
    /// * `J2P(2)`
    /// * `J2NN(3)`
    /// * `J2NZ(4)`
    /// * `J2NP(5)`
    ///
    /// ```text
    /// rI2 : 0; jump
    /// ```
    ///
    /// ------
    ///
    /// See also [`Opcode::Jmp`].
    J2 = 42,

    /// * `J3N(0)`
    /// * `J3Z(1)`
    /// * `J3P(2)`
    /// * `J3NN(3)`
    /// * `J3NZ(4)`
    /// * `J3NP(5)`
    ///
    /// ```text
    /// rI3 : 0; jump
    /// ```
    ///
    /// ------
    ///
    /// See also [`Opcode::Jmp`].
    J3 = 43,

    /// * `J4N(0)`
    /// * `J4Z(1)`
    /// * `J4P(2)`
    /// * `J4NN(3)`
    /// * `J4NZ(4)`
    /// * `J4NP(5)`
    ///
    /// ```text
    /// rI4 : 0; jump
    /// ```
    ///
    /// ------
    ///
    /// See also [`Opcode::Jmp`].
    J4 = 44,

    /// * `J5N(0)`
    /// * `J5Z(1)`
    /// * `J5P(2)`
    /// * `J5NN(3)`
    /// * `J5NZ(4)`
    /// * `J5NP(5)`
    ///
    /// ```text
    /// rI5 : 0; jump
    /// ```
    ///
    /// ------
    ///
    /// See also [`Opcode::Jmp`].
    J5 = 45,

    /// * `J6N(0)`
    /// * `J6Z(1)`
    /// * `J6P(2)`
    /// * `J6NN(3)`
    /// * `J6NZ(4)`
    /// * `J6NP(5)`
    ///
    /// ```text
    /// rI6 : 0; jump
    /// ```
    ///
    /// ------
    ///
    /// See also [`Opcode::Jmp`].
    J6 = 46,

    /// * `JXN(0)`
    /// * `JXZ(1)`
    /// * `JXP(2)`
    /// * `JXNN(3)`
    /// * `JXNZ(4)`
    /// * `JXNP(5)`
    ///
    /// ```text
    /// rX : 0; jump
    /// ```
    ///
    /// ------
    ///
    /// * `JXE(6)` (`x-binary`) - Jump `rX` even
    /// * `JXO(7)` (`x-binary`) - Jump `rX` odd
    ///
    /// ```text
    /// rA & 1 : 0; jump
    /// ```
    ///
    /// ------
    ///
    /// See also [`Opcode::Jmp`].
    JX = 47,

    /// * `INCA(0)` - Increase `rA` by 1.
    /// * `DECA(1)` - Decrease `rA` by 1.
    /// * `ENTA(2)` - Load an immediate value into `rA`.
    /// * `ENNA(3)` - Load a negative immediate value into `rA`.
    ///
    /// ```text
    /// rA <- [rA]? +- M
    /// ```
    ModifyA = 48,

    /// * `INC1(0)` - Increase `rI1` by 1.
    /// * `DEC1(1)` - Decrease `rI1` by 1.
    /// * `ENT1(2)` - Load an immediate value into `rI1`.
    /// * `ENN1(3)` - Load a negative immediate value into `rI1`.
    ///
    /// ```text
    /// rI1 <- [rI1]? +- M
    /// ```
    Modify1 = 49,

    /// * `INC2(0)` - Increase `rI2` by 1.
    /// * `DEC2(1)` - Decrease `rI2` by 1.
    /// * `ENT2(2)` - Load an immediate value into `rI2`.
    /// * `ENN2(3)` - Load a negative immediate value into `rI2`.
    ///
    /// ```text
    /// rI2 <- [rI2]? +- M
    /// ```
    Modify2 = 50,

    /// * `INC3(0)` - Increase `rI3` by 1.
    /// * `DEC3(1)` - Decrease `rI3` by 1.
    /// * `ENT3(2)` - Load an immediate value into `rI3`.
    /// * `ENN3(3)` - Load a negative immediate value into `rI3`.
    ///
    /// ```text
    /// rI3 <- [rI3]? +- M
    /// ```
    Modify3 = 51,

    /// * `INC4(0)` - Increase `rI4` by 1.
    /// * `DEC4(1)` - Decrease `rI4` by 1.
    /// * `ENT4(2)` - Load an immediate value into `rI4`.
    /// * `ENN4(3)` - Load a negative immediate value into `rI4`.
    ///
    /// ```text
    /// rI4 <- [rI4]? +- M
    /// ```
    Modify4 = 52,

    /// * `INC5(0)` - Increase `rI5` by 1.
    /// * `DEC5(1)` - Decrease `rI5` by 1.
    /// * `ENT5(2)` - Load an immediate value into `rI5`.
    /// * `ENN5(3)` - Load a negative immediate value into `rI5`.
    ///
    /// ```text
    /// rI5 <- [rI5]? +- M
    /// ```
    Modify5 = 53,

    /// * `INC6(0)` - Increase `rI6` by 1.
    /// * `DEC6(1)` - Decrease `rI6` by 1.
    /// * `ENT6(2)` - Load an immediate value into `rI6`.
    /// * `ENN6(3)` - Load a negative immediate value into `rI6`.
    ///
    /// ```text
    /// rI6 <- [rI6]? +- M
    /// ```
    Modify6 = 54,

    /// * `INCX(0)` - Increase `rX` by 1.
    /// * `DECX(1)` - Decrease `rX` by 1.
    /// * `ENTX(2)` - Load an immediate value into `rX`.
    /// * `ENNX(3)` - Load a negative immediate value into `rX`.
    ///
    /// ```text
    /// rX <- [rX]? +- M
    /// ```
    ModifyX = 55,

    /// * `CMPA(0:5)` - Compare `rA` with `V`.
    ///
    /// ```text
    /// CI <- rA(F) : V
    /// ```
    ///
    /// ------
    ///
    /// * `F32CMPA(7)` (`x-ieee754`) - Compare `rA` with `V` as `binary32` values.
    ///
    /// ```text
    /// CI <- rA : V
    /// ```
    CmpA = 56,

    /// * `CMP1(0:5)` - Compare `rI1` with `V`.
    ///
    /// ```text
    /// CI <- rI1(F) : V
    /// ```
    Cmp1 = 57,

    /// * `CMP2(0:5)` - Compare `rI2` with `V`.
    ///
    /// ```text
    /// CI <- rI2(F) : V
    /// ```
    Cmp2 = 58,

    /// * `CMP3(0:5)` - Compare `rI3` with `V`.
    ///
    /// ```text
    /// CI <- rI3(F) : V
    /// ```
    Cmp3 = 59,

    /// * `CMP4(0:5)` - Compare `rI4` with `V`.
    ///
    /// ```text
    /// CI <- rI4(F) : V
    /// ```
    Cmp4 = 60,

    /// * `CMP5(0:5)` - Compare `rI5` with `V`.
    ///
    /// ```text
    /// CI <- rI5(F) : V
    /// ```
    Cmp5 = 61,

    /// * `CMP6(0:5)` - Compare `rI6` with `V`.
    ///
    /// ```text
    /// CI <- rI6(F) : V
    /// ```
    Cmp6 = 62,

    /// * `CMPX(0:5)` - Compare `rX` with `V`.
    ///
    /// ```text
    /// CI <- rX(F) : V
    /// ```
    ///
    /// ------
    ///
    /// * `F32CMPX(7)` (`x-ieee754`) - Compare `rX` with `V` as `binary32` values.
    ///
    /// ```text
    /// CI <- rX : V
    /// ```
    CmpX = 63,
}

/// Used when converting a type to a [`RangeInclusive<T>`].
pub trait ToRangeInclusive<T> {
    /// Convert some value to [`RangeInclusive<T>`].
    fn to_range_inclusive(self) -> RangeInclusive<T>;

    /// Convert some value to a [`RangeInclusive<T>`], but
    /// removing sign byte from range if necessary.
    fn to_range_inclusive_signless(self) -> (RangeInclusive<T>, bool);
}

/// Implements conversion from byte-packed `F` value to
/// [`RangeInclusive<usize>`].
///
/// ```text
/// F <- 8 * L + R
/// ```
impl ToRangeInclusive<usize> for u8 {
    /// Convert [`u8`] to [`RangeInclusive<usize>`].
    ///
    /// # Returns
    /// * [`RangeInclusive<usize>`]
    fn to_range_inclusive(self) -> RangeInclusive<usize> {
        ((self / 8) as usize)..=((self % 8) as usize)
    }

    /// Convert [`u8`] to [`RangeInclusive<usize>`], but
    /// removing sign byte from range if necessary.
    ///
    /// In this sense, consider the byte-packed range `1`, which
    /// represents `0..=1`. Since byte 0 is the sign byte, this
    /// method returns `(1..=1, true)` to indicate a sign byte
    /// has been discarded.
    ///
    /// # Returns
    /// * [`RangeInclusive<usize>`]
    /// * [`bool`]: `true` if the sign bit is taken into consideration.
    fn to_range_inclusive_signless(self) -> (RangeInclusive<usize>, bool) {
        let orig_range = self.to_range_inclusive();
        let has_sign = *orig_range.start() == 0;
        let new_start = if has_sign {
            *orig_range.start() + 1
        } else {
            *orig_range.start()
        };
        (new_start..=*orig_range.end(), has_sign)
    }
}
