use crate::utils::{err, ok, Reader, Result, Span, Spanned};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Symbol {
    Vowel(u8),
    Consonant(u8),
    Sonorant(u8),
    H,
    Pause,
    Hyphen,
    Space,
}

fn classify_letter(c: u8) -> Symbol {
    match c {
        c if is_vowel(c) => Symbol::Vowel(c),
        c if is_sonorant(c) => Symbol::Sonorant(c),
        c if is_consonant(c) => Symbol::Consonant(c),
        b'h' => Symbol::H,
        _ => panic!("not a letter"),
    }
}

fn is_letter(c: u8) -> bool {
    b"nrlmpbfvtdszcjgkieaouh".contains(&c)
}

pub fn is_vowel(c: u8) -> bool {
    b"ieaou".contains(&c)
}

pub fn is_sonorant(c: u8) -> bool {
    b"nrl".contains(&c)
}

pub fn is_consonant(c: u8) -> bool {
    b"mpbfvtdszcjgk".contains(&c)
}

pub fn peek_hyphen(r: &mut Reader<u8>) -> Option<usize> {
    r.peek_exact_among(&[
        "-".as_bytes(), // hyphen
        "–".as_bytes(), // en dash
        "—".as_bytes(), // em dash
        "−".as_bytes(), // minus sign
    ])
}

pub fn peek_pause(r: &mut Reader<u8>) -> Option<usize> {
    r.peek_exact_among(&["'".as_bytes(), "’".as_bytes(), "`".as_bytes()])
}

pub fn parse_symbol(r: &mut Reader<u8>) -> Result<Spanned<Symbol>> {
    let start = r.cursor();

    // We start with multiple bytes characters like hyphens and pause.
    if let Some(len) = peek_hyphen(r) {
        let _ = r.consume_many(len);

        // 2 hyphens are not allowed in a row.
        if let Some(len) = peek_hyphen(r) {
            return err(start..r.cursor() + len, "2 hyphens cannot appear in a row");
        }

        return ok(start..r.cursor(), Symbol::Hyphen);
    }

    if let Some(len) = peek_pause(r) {
        let _ = r.consume_many(len);

        // 2 pauses are not allowed in a row.
        if let Some(len) = peek_pause(r) {
            return err(start..r.cursor() + len, "2 pauses cannot appear in a row");
        }

        return ok(start..r.cursor(), Symbol::Pause);
    }

    // Then we consume one char, which can be either a letter or something else
    // which will be considered a space.
    let c = r
        .consume()
        .ok_or(Span::new(start, start).wrap("End of input".into()))?
        .to_ascii_lowercase();

    if is_letter(c) {
        // We read until we don't find another instance of that same letter.
        while let Some(c2) = r.peek() {
            if c2.to_ascii_lowercase() != c {
                break;
            }
            let _ = r.consume();
        }

        return ok(start..r.cursor(), classify_letter(c.into()));
    }

    // If `c` is not a letter it is a space. We consume characters until we can
    // peak an hyphen, pause or letter.
    loop {
        if peek_hyphen(r).is_some() || peek_pause(r).is_some() {
            break;
        }

        let Some(&c) = r.peek() else {
            break;
        };

        if is_letter(c) {
            break;
        }

        r.consume();
    }

    ok(start..r.cursor(), Symbol::Space)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_symbol_works() {
        let parse = |input: &str| {
            let mut reader = Reader::new(input.as_bytes());
            parse_symbol(&mut reader)
        };

        assert_eq!(parse("ab"), ok(0..1, Symbol::Vowel(b'a')));
        assert_eq!(parse("aab"), ok(0..2, Symbol::Vowel(b'a')));
        assert_eq!(parse("aaab"), ok(0..3, Symbol::Vowel(b'a')));

        assert_eq!(parse(" a"), ok(0..1, Symbol::Space));
        assert_eq!(parse("  a"), ok(0..2, Symbol::Space));
        assert_eq!(parse("   a"), ok(0..3, Symbol::Space));
        assert_eq!(parse(".a"), ok(0..1, Symbol::Space));
        assert_eq!(parse("./a"), ok(0..2, Symbol::Space));
        assert_eq!(parse(" $#@a"), ok(0..4, Symbol::Space));

        for hyphen in &["-", "–", "—", "−"] {
            assert_eq!(
                parse(&format!("{hyphen}a")),
                ok(0..hyphen.as_bytes().len(), Symbol::Hyphen)
            );

            for hyphen2 in &["-", "–", "—", "−"] {
                assert_eq!(
                    parse(&format!("{hyphen}{hyphen2}a")),
                    err(
                        0..hyphen.as_bytes().len() + hyphen2.as_bytes().len(),
                        "2 hyphens cannot appear in a row"
                    )
                );
            }
        }

        for pause in &["'", "’", "`"] {
            assert_eq!(
                parse(&format!("{pause}a")),
                ok(0..pause.as_bytes().len(), Symbol::Pause)
            );

            for pause2 in &["'", "’", "`"] {
                assert_eq!(
                    parse(&format!("{pause}{pause2}a")),
                    err(
                        0..pause.as_bytes().len() + pause2.as_bytes().len(),
                        "2 pauses cannot appear in a row"
                    )
                );
            }
        }
    }
}
