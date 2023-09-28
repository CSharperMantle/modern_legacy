/// The common alphabet used in [`MixVM`].
///
/// See D. E. Knuth, *The Art of Computer Programming*, Volume 1, pp 140
/// for more information.
///
/// [`MixVM`]: crate::MixVM
#[derive(Clone, Copy, PartialEq, Eq, Debug, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Alphabet {
    /// The character '` `'.
    Space = 0,

    /// The character '`A`'.
    A = 1,

    /// The character '`B`'.
    B = 2,

    /// The character '`C`'.
    C = 3,

    /// The character '`D`'.
    D = 4,

    /// The character '`E`'.
    E = 5,

    /// The character '`F`'.
    F = 6,

    /// The character '`G`'.
    G = 7,

    /// The character '`H`'.
    H = 8,

    /// The character '`I`'.
    I = 9,

    /// The character '`'`'.
    SQuote = 10,

    /// The character '`J`'.
    J = 11,

    /// The character '`K`'.
    K = 12,

    /// The character '`L`'.
    L = 13,

    /// The character '`M`'.
    M = 14,

    /// The character '`N`'.
    N = 15,

    /// The character '`O`'.
    O = 16,

    /// The character '`P`'.
    P = 17,

    /// The character '`Q`'.
    Q = 18,

    /// The character '`R`'.
    R = 19,

    /// The character '`°`'.
    Degree = 20,

    /// The character '`"`'.
    DQuote = 21,

    /// The character '`S`'.
    S = 22,

    /// The character '`T`'.
    T = 23,

    /// The character '`U`'.
    U = 24,

    /// The character '`V`'.
    V = 25,

    /// The character '`W`'.
    W = 26,

    /// The character '`X`'.
    X = 27,

    /// The character '`Y`'.
    Y = 28,

    /// The character '`Z`'.
    Z = 29,

    /// The character '`0`'.
    Zero = 30,

    /// The character '`1`'.
    One = 31,

    /// The character '`2`'.
    Two = 32,

    /// The character '`3`'.
    Three = 33,

    /// The character '`4`'.
    Four = 34,

    /// The character '`5`'.
    Five = 35,

    /// The character '`6`'.
    Six = 36,

    /// The character '`7`'.
    Seven = 37,

    /// The character '`8`'.
    Eight = 38,

    /// The character '`9`'.
    Nine = 39,

    /// The character '`.`'.
    Dot = 40,

    /// The character '`,`'.
    Comma = 41,

    /// The character '`(`'.
    LParen = 42,

    /// The character '`)`'.
    RParen = 43,

    /// The character '`+`'.
    Plus = 44,

    /// The character '`-`'.
    Minus = 45,

    /// The character '`*`'.
    Star = 46,

    /// The character '`/`'.
    Slash = 47,

    /// The character '`=`'.
    Equal = 48,

    /// The character '`$`'.
    Dollar = 49,

    /// The character '`<`'.
    LAngle = 50,

    /// The character '`>`'.
    RAngle = 51,

    /// The character '`@`'.
    At = 52,

    /// The character '`;`'.
    SemiColon = 53,

    /// The character '`:`'.
    Colon = 54,

    /// The character '`‚`'.
    LowSQuote = 55,
}

impl TryFrom<Alphabet> for u8 {
    type Error = ();

    /// Converts a character in [`Alphabet`] to its numerical representation.
    ///
    /// # Returns
    /// * [`Ok(u8)`] - The converted byte.
    /// * [`Err(())`] - The conversion fails.
    fn try_from(value: Alphabet) -> Result<Self, Self::Error> {
        Ok(value as u8)
    }
}

impl TryFrom<Alphabet> for char {
    type Error = ();

    /// Converts a character in [`Alphabet`] to a [`char`].
    ///
    /// # Returns
    /// * [`Ok(char)`] - The converted [`char`].
    /// * [`Err(())`] - The conversion fails
    fn try_from(value: Alphabet) -> Result<Self, Self::Error> {
        match value {
            Alphabet::Space => Ok(' '),
            Alphabet::A => Ok('A'),
            Alphabet::B => Ok('B'),
            Alphabet::C => Ok('C'),
            Alphabet::D => Ok('D'),
            Alphabet::E => Ok('E'),
            Alphabet::F => Ok('F'),
            Alphabet::G => Ok('G'),
            Alphabet::H => Ok('H'),
            Alphabet::I => Ok('I'),
            Alphabet::SQuote => Ok('\''),
            Alphabet::J => Ok('J'),
            Alphabet::K => Ok('K'),
            Alphabet::L => Ok('L'),
            Alphabet::M => Ok('M'),
            Alphabet::N => Ok('N'),
            Alphabet::O => Ok('O'),
            Alphabet::P => Ok('P'),
            Alphabet::Q => Ok('Q'),
            Alphabet::R => Ok('R'),
            Alphabet::Degree => Ok('°'),
            Alphabet::DQuote => Ok('"'),
            Alphabet::S => Ok('S'),
            Alphabet::T => Ok('T'),
            Alphabet::U => Ok('U'),
            Alphabet::V => Ok('V'),
            Alphabet::W => Ok('W'),
            Alphabet::X => Ok('X'),
            Alphabet::Y => Ok('Y'),
            Alphabet::Z => Ok('Z'),
            Alphabet::Zero => Ok('0'),
            Alphabet::One => Ok('1'),
            Alphabet::Two => Ok('2'),
            Alphabet::Three => Ok('3'),
            Alphabet::Four => Ok('4'),
            Alphabet::Five => Ok('5'),
            Alphabet::Six => Ok('6'),
            Alphabet::Seven => Ok('7'),
            Alphabet::Eight => Ok('8'),
            Alphabet::Nine => Ok('9'),
            Alphabet::Dot => Ok('.'),
            Alphabet::Comma => Ok(','),
            Alphabet::LParen => Ok('('),
            Alphabet::RParen => Ok(')'),
            Alphabet::Plus => Ok('+'),
            Alphabet::Minus => Ok('-'),
            Alphabet::Star => Ok('*'),
            Alphabet::Slash => Ok('/'),
            Alphabet::Equal => Ok('='),
            Alphabet::Dollar => Ok('$'),
            Alphabet::LAngle => Ok('<'),
            Alphabet::RAngle => Ok('>'),
            Alphabet::At => Ok('@'),
            Alphabet::SemiColon => Ok(';'),
            Alphabet::Colon => Ok(':'),
            Alphabet::LowSQuote => Ok('‚'),
        }
    }
}

impl TryFrom<char> for Alphabet {
    type Error = ();

    /// Converts a [`char`] to a character in [`Alphabet`].
    ///
    /// # Returns
    /// * [`Ok(Alphabet)`] - The converted character in [`Alphabet`].
    /// * [`Err(())`] - The conversion fails.
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            ' ' => Ok(Alphabet::Space),
            'A' => Ok(Alphabet::A),
            'B' => Ok(Alphabet::B),
            'C' => Ok(Alphabet::C),
            'D' => Ok(Alphabet::D),
            'E' => Ok(Alphabet::E),
            'F' => Ok(Alphabet::F),
            'G' => Ok(Alphabet::G),
            'H' => Ok(Alphabet::H),
            'I' => Ok(Alphabet::I),
            '\'' => Ok(Alphabet::SQuote),
            'J' => Ok(Alphabet::J),
            'K' => Ok(Alphabet::K),
            'L' => Ok(Alphabet::L),
            'M' => Ok(Alphabet::M),
            'N' => Ok(Alphabet::N),
            'O' => Ok(Alphabet::O),
            'P' => Ok(Alphabet::P),
            'Q' => Ok(Alphabet::Q),
            'R' => Ok(Alphabet::R),
            '°' => Ok(Alphabet::Degree),
            '"' => Ok(Alphabet::DQuote),
            'S' => Ok(Alphabet::S),
            'T' => Ok(Alphabet::T),
            'U' => Ok(Alphabet::U),
            'V' => Ok(Alphabet::V),
            'W' => Ok(Alphabet::W),
            'X' => Ok(Alphabet::X),
            'Y' => Ok(Alphabet::Y),
            'Z' => Ok(Alphabet::Z),
            '0' => Ok(Alphabet::Zero),
            '1' => Ok(Alphabet::One),
            '2' => Ok(Alphabet::Two),
            '3' => Ok(Alphabet::Three),
            '4' => Ok(Alphabet::Four),
            '5' => Ok(Alphabet::Five),
            '6' => Ok(Alphabet::Six),
            '7' => Ok(Alphabet::Seven),
            '8' => Ok(Alphabet::Eight),
            '9' => Ok(Alphabet::Nine),
            '.' => Ok(Alphabet::Dot),
            ',' => Ok(Alphabet::Comma),
            '(' => Ok(Alphabet::LParen),
            ')' => Ok(Alphabet::RParen),
            '+' => Ok(Alphabet::Plus),
            '-' => Ok(Alphabet::Minus),
            '*' => Ok(Alphabet::Star),
            '/' => Ok(Alphabet::Slash),
            '=' => Ok(Alphabet::Equal),
            '$' => Ok(Alphabet::Dollar),
            '<' => Ok(Alphabet::LAngle),
            '>' => Ok(Alphabet::RAngle),
            '@' => Ok(Alphabet::At),
            ';' => Ok(Alphabet::SemiColon),
            ':' => Ok(Alphabet::Colon),
            '‚' => Ok(Alphabet::LowSQuote),
            _ => Err(()),
        }
    }
}
