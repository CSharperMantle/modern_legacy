use core::ops::Index;
use core::ops::IndexMut;
use core::ops::Range;
use core::ops::RangeFull;
use core::ops::RangeInclusive;

use super::instr::Instruction;

/// A generic word in [`MixVM`] with `N` bytes in it.
///
/// Word are the basic unit of memory in MIX. A normal word
/// contains 5 bytes and a sign byte. Note, however, that a
/// byte may contain *arbitrary* amount of bits. A proper MIX
/// program should run regardless of the number of bytes in a
/// word. It is thus impossible to tell the content of individual
/// bytes if several bytes are joined to represent a single
/// scalar.
///
/// A byte should be able to represent a scalar no less than
/// decimal `60`.
///
/// # Generic Parameters
/// * `N` - The number of bytes in the word, including sign.
/// * `P` - Whether the sign byte is always positive.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Word<const N: usize, const P: bool> {
    data: [u8; N],
}

impl<const N: usize, const P: bool> Word<N, P> {
    /// Negative sign byte content.
    pub const NEG: u8 = 1;

    /// Positive sign byte content.
    pub const POS: u8 = 0;

    /// Create a new word with default values.
    ///
    /// Equivalent to [`Word<N, P>::default()`].
    pub const fn new() -> Self {
        let mut w: Word<N, P> = Word { data: [0; N] };
        w.data[0] = if P { Self::POS } else { w.data[0] };
        w
    }

    /// Create a new word from the given content.
    ///
    /// Sign byte settings of `P` will be honored.
    ///
    /// # Arguments
    /// * `bytes` - The content of the word.
    pub const fn from_bytes(bytes: [u8; N]) -> Self {
        let mut w = Word { data: bytes };
        w.data[0] = if P { Self::POS } else { w.data[0] };
        w
    }

    /// Create a new word from an `i64`.
    ///
    /// The function stores big-endian representation of the
    /// given `i64` shifted to right. It means that if we have
    /// a `Word<6, false>` only 5 bytes starting from right will
    /// be stored. The sign byte is always `1` if `P` is `true`.
    ///
    /// # Arguments
    /// * `value` - The value to initialize the word with.
    ///
    /// # Returns
    /// * [`Word`] - The initialized word.
    /// * [`bool`] - `true` if the given `i64` is too large, `false` otherwise.
    pub fn from_i64(value: i64) -> (Self, bool) {
        let mut word = Self::new();
        let bytes = value.abs().to_be_bytes();
        // See if we have something not copied.
        // Bytes marked 'dirty' have not been copied yet.
        let overflow = N - 1 < 8 && bytes[0..8 - (N - 1)].iter().any(|&b| b != 0);
        word[0] = if !P && value < 0 {
            Self::NEG
        } else {
            Self::POS
        };
        for (word_i, bytes_i) in (1..N).rev().zip((0..8).rev()) {
            word[word_i] = bytes[bytes_i];
        }
        // If we have left some data behind, we have overflowed.
        (word, overflow)
    }

    /// Set the content of the whole word to given array.
    ///
    /// # Arguments
    /// * `value` - The value to set.
    pub fn set_all(&mut self, value: [u8; N]) {
        self.data.copy_from_slice(&value);
        if P {
            self.data[0] = Self::POS;
        }
    }

    /// Check if the word is positive.
    ///
    /// # Returns
    /// * `true` - If the word is positive, `word[0] == Self::POS`.
    /// * `false` - If the word is negative, `word[0] != Self::POS`.
    pub const fn is_positive(&self) -> bool {
        self.data[0] == Self::POS
    }

    /// Get sign adjustment coefficient for the word.
    ///
    /// # Returns
    /// * `1` - If the word is positive.
    /// * `-1` - If the word is negative.
    pub const fn get_sign(&self) -> i8 {
        if self.is_positive() {
            1
        } else {
            -1
        }
    }

    /// Flip the sign of the word.
    ///
    /// This method has no effect if the word is always positive,
    /// i.e. `P == true`.
    pub fn flip_sign(&mut self) {
        self.data[0] = if !P && self.is_positive() {
            Self::NEG
        } else {
            Self::POS
        };
    }

    /// Convert the word to an `i64`.
    ///
    /// This method squashes big-endian representation of the bytes
    /// into a single quantity, ignoring too significant bytes.
    ///
    /// # Returns
    /// * [`i64`] - The converted value.
    /// * [`bool`] - `true` if the word overflows. Higher zero bytes do not count as overflow.
    pub fn to_i64(self) -> (i64, bool) {
        let sign = self.get_sign() as i64;
        let mut bytes: [u8; 8] = [0; 8];
        let overflow = N - 1 > 8 && self.data[8..N].iter().any(|&b| b != 0);
        // Bytes marked 'dirty' have not been copied yet.
        for (bytes_i, data_i) in (0..8).rev().zip((1..N).rev()) {
            bytes[bytes_i] = self.data[data_i];
        }
        let value = i64::from_be_bytes(bytes);
        (value * sign, overflow)
    }

    /// Convert the corresponding range of an word to an `i64`.
    ///
    /// # Arguments
    /// * `field` - The field to convert. Value: `F <- L * 8 + R`.
    ///
    /// # Returns
    /// * [`i64`] - The converted value.
    /// * [`bool`] - `true` if the word overflows.
    pub fn to_i64_ranged(self, field: RangeInclusive<usize>) -> (i64, bool) {
        // Move sign byte out.
        let sign_included = *field.start() == 0;
        let new_start = if sign_included {
            *field.start() + 1
        } else {
            *field.start()
        };
        let field = new_start..=*field.end();
        // Get sliced data.
        let data = &self.data[field];
        // If the range is empty, fast-fail.
        if data.is_empty() {
            return (0, false);
        }
        // Find sign.
        let sign = if !sign_included { 1 } else { self.get_sign() } as i64;
        let mut result_bytes: [u8; 8] = [0; 8];
        // Get count of bytes that is needed to copy.
        let data_bytes_nonzero_count = data.iter().filter(|&&b| b != 0).count();
        // Copy bytes from the slice.
        // Ranges are chained by zip, and the shorter range is
        // iterated over in order to prevent out-of-bound indices.
        // Filling starts from the LSB.
        for (bytes_i, data_i) in (0..8).rev().zip((0..data.len()).rev()) {
            result_bytes[bytes_i] = data[data_i];
        }
        let value = i64::from_be_bytes(result_bytes);

        (value * sign, data_bytes_nonzero_count > 8)
    }
}

impl<const N: usize, const P: bool> Default for Word<N, P> {
    /// Create a new word with default value.
    ///
    /// Equivalent to [`Word<N, P>::new()`].
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, const P: bool> Index<RangeInclusive<usize>> for Word<N, P> {
    type Output = [u8];

    /// Access the content of the word with the given range.
    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        &self.data[index]
    }
}

impl<const N: usize, const P: bool> IndexMut<RangeInclusive<usize>> for Word<N, P> {
    /// Mutably access the content of the word with the given range.
    fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<const N: usize, const P: bool> Index<RangeFull> for Word<N, P> {
    type Output = [u8];

    /// Access the whole word.
    fn index(&self, index: RangeFull) -> &Self::Output {
        &self.data[index]
    }
}

impl<const N: usize, const P: bool> IndexMut<RangeFull> for Word<N, P> {
    /// Mutably access the whole word.
    fn index_mut(&mut self, index: RangeFull) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<const N: usize, const P: bool> Index<usize> for Word<N, P> {
    type Output = u8;

    /// Access the content of the word with the given index.
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const N: usize, const P: bool> IndexMut<usize> for Word<N, P> {
    /// Mutably Access the content of the word with the given index.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl From<Instruction> for Word<6, false> {
    /// Convert an [`Instruction`] to a [`Word<6, false>`].
    ///
    /// # Arguments
    /// * `source` - The instruction to convert.
    ///
    /// # Returns
    /// * [`Word<6, false>`] - Converted [`Word`].
    fn from(source: Instruction) -> Self {
        let source_addr = source.addr.unsigned_abs().to_be_bytes();
        Self::from_bytes([
            u8::from(source.addr < 0),
            source_addr[0],
            source_addr[1],
            source.index,
            source.field,
            source.opcode as u8,
        ])
    }
}

/// Alias for a 6-byte [`Word`] including a sign byte.
pub type FullWord = Word<6, false>;

/// Alias for a 3-byte [`Word`] including a sign byte.
pub type HalfWord = Word<3, false>;

/// Alias for a 3-byte [`Word`] including a sign byte,
/// which is always equal to [`Word::POS`].
pub type PosHalfWord = Word<3, true>;

/// The memory area of a [`MixVM`] comprised of [`FullWord`]s.
///
/// [`MixVM`]: crate::MixVM
#[derive(Clone, Debug)]
pub struct Mem {
    /// The memory area.
    data: [FullWord; Self::SIZE],
}

impl Mem {
    /// Create a new memory area with all-zero words.
    ///
    /// Equivalent to [`Mem::default()`].
    ///
    /// # Returns
    /// * [`Mem`] - The new memory area.
    pub const fn new() -> Self {
        Mem {
            data: [FullWord::new(); 4000],
        }
    }

    /// Number of words in the memory area.
    pub const SIZE: usize = 4000;
}

impl Index<u16> for Mem {
    type Output = FullWord;

    /// Access the word at a memory location.
    fn index(&self, index: u16) -> &Self::Output {
        &self.data[index as usize]
    }
}

impl IndexMut<u16> for Mem {
    /// Access the mutable word at a memory location.
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.data[index as usize]
    }
}

impl Index<Range<usize>> for Mem {
    type Output = [FullWord];

    /// Access the word at a range.
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<Range<usize>> for Mem {
    /// Access the mutable word at a range.
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl Default for Mem {
    /// Create a clean memory area.
    ///
    /// Equivalent to [`Mem::new()`].
    fn default() -> Self {
        Self::new()
    }
}
