use crate::reader::*;

// Macro to quickly generate parsers for symbols.
macro_rules! symbols_enum {
    (
        $name:ident $display:literal
        [$(
            $variant:ident: $first_symbol:literal $($symbol:literal)*
        ),+ $(,)?]
    ) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub enum $name {
            $(
                $variant,
            )+
        }

        impl $name {
            pub fn accepted_symbols(self) -> &'static [&'static [u8]] {
                match self {
                    $(
                        Self::$variant => {
                            const CANDIDATES: &'static [&'static [u8]] = &[
                                $first_symbol.as_bytes(), $($symbol.as_bytes()),+
                            ];
                            CANDIDATES
                        }
                    ),+
                }
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                write!(f, "{}", match self {
                    $(
                        Self::$variant => $first_symbol,
                    )+
                })
            }
        }

        impl Parse<u8> for $name {
            // No errors and no state at this stage.
            type Error = ();
            type State = ();

            fn describe() -> &'static str {
                $display
            }

            fn parse(reader: &mut SpannedReader<u8>, _: &mut ()) -> Result<Self, ()> {
                let input = reader.input_after();

                $(
                    if let Some(len) = read_repeated_symbol_among(input, Self::$variant.accepted_symbols()) {
                        reader.forward(len);

                        return Ok(Self::$variant);
                    }
                )+

                Err(())
            }
        }

    }
}

symbols_enum!(Vowel "vowel" [
    I: "i" "I" "ı",
    E: "e" "E",
    A: "a" "A",
    O: "o" "O",
    U: "u" "U",
]);
symbols_enum!(Sonorant "sonorant" [
    N: "n" "N",
    R: "r" "R",
    L: "l" "L",
]);
symbols_enum!(NonSonorant "non-sonorant" [
    M: "m" "M",
    P: "p" "P",
    B: "b" "B",
    F: "f" "F",
    V: "v" "V",
    T: "t" "T",
    D: "d" "D",
    S: "s" "S",
    Z: "z" "Z",
    C: "c" "C",
    J: "j" "J",
    G: "g" "G",
    K: "k" "K",
]);
symbols_enum!(H "h" [
    H: "h" "H",
]);
symbols_enum!(Pause "pause" [
    Pause: "'" "`" "‘" "’" "′",
]);
symbols_enum!(Hyphen "hyphen" [
    Hyphen: "-" "֊" "‐" "‑",
]);

fn read_repeated_symbol_among<'input>(
    input: &'input [u8],
    candidates: &'static [&'static [u8]],
) -> Option<usize> {
    let mut len = read_symbol_among(input, candidates)?;
    while let Some(more) = read_symbol_among(&input[len..], candidates) {
        len += more;
    }
    Some(len)
}

fn read_symbol_among<'input>(
    input: &'input [u8],
    candidates: &'static [&'static [u8]],
) -> Option<usize> {
    for candidate in candidates {
        if input.starts_with(candidate) {
            return Some(candidate.len());
        }
    }
    None
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Symbol {
    Vowel(Vowel),
    Sonorant(Sonorant),
    NonSonorant(NonSonorant),
    H(H),
    Pause(Pause),
    Hyphen(Hyphen),
    Space,
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Vowel(s) => s.fmt(f),
            Self::Sonorant(s) => s.fmt(f),
            Self::NonSonorant(s) => s.fmt(f),
            Self::H(s) => s.fmt(f),
            Self::Pause(s) => s.fmt(f),
            Self::Hyphen(s) => s.fmt(f),
            Self::Space => write!(f, "_"),
        }
    }
}

pub struct TextAsSymbols(pub Vec<Spanned<Symbol>>);

fn parse_non_space(reader: &mut SpannedReader<u8>) -> Option<Symbol> {
    if let Ok(s) = Vowel::parse(reader, &mut ()) {
        Some(Symbol::Vowel(s))
    } else if let Ok(s) = Sonorant::parse(reader, &mut ()) {
        Some(Symbol::Sonorant(s))
    } else if let Ok(s) = NonSonorant::parse(reader, &mut ()) {
        Some(Symbol::NonSonorant(s))
    } else if let Ok(s) = H::parse(reader, &mut ()) {
        Some(Symbol::H(s))
    } else if let Ok(s) = Pause::parse(reader, &mut ()) {
        Some(Symbol::Pause(s))
    } else if let Ok(s) = Hyphen::parse(reader, &mut ()) {
        Some(Symbol::Hyphen(s))
    } else {
        None
    }
}

impl Parse<u8> for TextAsSymbols {
    // No errors and no state at this stage.
    type Error = ();
    type State = ();

    fn describe() -> &'static str {
        "text as symbols"
    }

    fn parse(reader: &mut SpannedReader<u8>, _: &mut ()) -> Result<Self, ()> {
        let mut symbols = vec![];

        while reader.remaining() > 0 {
            let space_start = reader.cursor();

            loop {
                let symbol_start = reader.cursor();
                match parse_non_space(reader) {
                    None => reader.forward(1),
                    Some(symbol) => {
                        if symbol_start != space_start {
                            symbols.push(Symbol::Space.spanned(space_start..symbol_start));
                        }

                        symbols.push(symbol.spanned(symbol_start..reader.cursor()));
                        break; // break out of `loop`, reseting potential space start
                    }
                }
            }
        }

        Ok(Self(symbols))
    }
}

impl TextAsSymbols {
    pub fn legend(&self, input: &[u8]) -> String {
        use unicode_segmentation::UnicodeSegmentation;

        let mut out_legend = String::new();

        for Spanned { span, inner } in &self.0 {
            let source = input[span.clone()].to_owned();
            let source = String::from_utf8(source).unwrap();
            let display_len = source.graphemes(true).count();

            out_legend.push_str(&inner.to_string());

            if display_len >= 1 {
                let dashes = display_len - 1;
                for _ in 0..dashes {
                    out_legend.push_str(" ");
                }
            }
        }

        out_legend
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let input_ = "aA! 'pLl‘@=nH‐-ııIIi".as_bytes();
        let legend = "a _ 'pl '_ nh- i    ";
        let mut reader = SpannedReader::new(&input_);
        let parsed = TextAsSymbols::parse(&mut reader).unwrap();
        assert_eq!(&parsed.legend(&input_), legend);
    }
}
