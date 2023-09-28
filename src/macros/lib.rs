use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::iter::repeat;
use syn::parse_macro_input;
use syn::Ident;
use syn::LitStr;

fn parse_char(ch: char) -> Result<&'static str, char> {
    match ch {
        ' ' => Ok("Space"),
        'A' => Ok("A"),
        'B' => Ok("B"),
        'C' => Ok("C"),
        'D' => Ok("D"),
        'E' => Ok("E"),
        'F' => Ok("F"),
        'G' => Ok("G"),
        'H' => Ok("H"),
        'I' => Ok("I"),
        '\'' => Ok("SQuote"),
        'J' => Ok("J"),
        'K' => Ok("K"),
        'L' => Ok("L"),
        'M' => Ok("M"),
        'N' => Ok("N"),
        'O' => Ok("O"),
        'P' => Ok("P"),
        'Q' => Ok("Q"),
        'R' => Ok("R"),
        '°' => Ok("Degree"),
        '"' => Ok("DQuote"),
        'S' => Ok("S"),
        'T' => Ok("T"),
        'U' => Ok("U"),
        'V' => Ok("V"),
        'W' => Ok("W"),
        'X' => Ok("X"),
        'Y' => Ok("Y"),
        'Z' => Ok("Z"),
        '0' => Ok("Zero"),
        '1' => Ok("One"),
        '2' => Ok("Two"),
        '3' => Ok("Three"),
        '4' => Ok("Four"),
        '5' => Ok("Five"),
        '6' => Ok("Six"),
        '7' => Ok("Seven"),
        '8' => Ok("Eight"),
        '9' => Ok("Nine"),
        '.' => Ok("Dot"),
        ',' => Ok("Comma"),
        '(' => Ok("LParen"),
        ')' => Ok("RParen"),
        '+' => Ok("Plus"),
        '-' => Ok("Minus"),
        '*' => Ok("Star"),
        '/' => Ok("Slash"),
        '=' => Ok("Equal"),
        '$' => Ok("Dollar"),
        '<' => Ok("LAngle"),
        '>' => Ok("RAngle"),
        '@' => Ok("At"),
        ';' => Ok("SemiColon"),
        ':' => Ok("Colon"),
        '‚' => Ok("LowSQuote"),

        _ => Err(ch),
    }
}

#[proc_macro]
pub fn alphabet_str(input: TokenStream) -> TokenStream {
    let string = parse_macro_input!(input as LitStr).value();
    if !string.is_ascii() {
        panic!("Only ASCII characters are supported");
    }

    let chars = string.as_bytes();
    let chunks = chars.chunks(5).map(|chunk| {
        let tokens = chunk
            .iter()
            .map(|ch| {
                let ident = Ident::new(parse_char(char::from(*ch)).unwrap(), Span::call_site());
                quote! {
                    Alphabet::#ident as u8,
                }
            })
            .chain(repeat(quote! {
                Alphabet::Space as u8,
            }))
            .take(5);
        quote! {
            [
                0, // zero sign
                #(#tokens)*
            ],
        }
    });

    TokenStream::from(quote! {
        [
            #(#chunks)*
        ]
    })
}
