use std::fmt::format;

use framework::*;

fn hyphen_opt<S>() -> impl Parser<S, u8, (), Error = super::Error> {
    let pattern = [b'-'];
    one_of::<_, u8, super::Error>(pattern)
        .then_peek(
            one_of(pattern)
                .then_error(|span, _| {
                    (
                        span.expand_before(1),
                        "Only one hyphen is allowed in a row.".to_string(),
                    )
                })
                .opt(),
        )
        .opt()
        .map(|_| ())
}

fn raw_letter<S>(choices: &[u8]) -> impl Parser<S, u8, char, Error = super::Error> {
    choice(
        choices
            .into_iter()
            .map(|&c| {
                let pattern = [c.to_ascii_lowercase(), c.to_ascii_uppercase()];
                let out = char::from(c);

                one_of(pattern).repeated(1..)
                    .then_peek(
                        hyphen_opt().then(one_of(pattern))
                            .then_error(|span,_| {
                                (span.expand_before(1), "The same letter cannot appear both before and after an hyphen.".to_string())
                            })
                            .opt()
                    )
                    .map(move |_| out)
            })
            .collect::<Vec<_>>(),
    )
}

pub fn vowel<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    raw_letter(b"ieaou")
}

fn voiced<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    raw_letter(b"bdgvzj")
}

fn unvoiced<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    raw_letter(b"ptkfsc")
}

fn sibilant<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    raw_letter(b"szcj")
}

fn plosive<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    raw_letter(b"tdkgpb")
}

fn sonorant<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    raw_letter(b"nrl")
}

fn liquid<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    raw_letter(b"lr")
}

fn letter_h<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    raw_letter(b"h")
        .spanned()
        .then_peek_with(move |(span_h, _)| {
            one_of([b'-'])
                .then_error(move |_, _| {
                    (
                        span_h.expand_after(1),
                        "An hyphen cannot appear after an 'h', and should appear before instead."
                            .to_string(),
                    )
                })
                .opt()
        })
        .map(|(_, h)| h)
}

fn other<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    let error = move |c1, c2| format!("{c1} cannot be followed by {c2} in an initial pair.");

    choice((
        raw_letter(b"vfkgm"),
        raw_letter(b"pb").then_peek_with(move |c1| {
            hyphen_opt().then(
                raw_letter(b"n")
                    .then_error(move |span, c2| (span, error(c1, c2)))
                    .opt(),
            )
        }),
        raw_letter(b"td").then_peek_with(move |c1| {
            hyphen_opt().then(
                raw_letter(b"nl")
                    .then_error(move |span, c2| (span, error(c1, c2)))
                    .opt(),
            )
        }),
        raw_letter(b"n").then_peek_with(move |c1| {
            hyphen_opt().then(
                liquid()
                    .then_error(move |span, c2| (span, error(c1, c2)))
                    .opt(),
            )
        }),
    ))
}

fn raw_consonant<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    raw_letter(b"nrlmpbfvtdszcjgk")
}

/// Parse a single consonant with morphology checks.
pub fn consonant<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    let error1 = move |((c1, _), c2)| {
        format!("A sibilant ({c1}) cannot be followed by another sibilant ({c2}).")
    };
    let error2 = move |((c1, _), c2)| {
        format!("A voiced consonant ({c1}) cannot be followed by an unvoiced one ({c2}).")
    };
    let error3 = move |((c1, _), c2)| {
        format!("An unvoiced consonant ({c1}) cannot be followed by an voiced one ({c2}).")
    };

    // we use `nil().then_peek(...)` to first check for many forbidden patterns.
    nil()
        .then_peek(choice((
            sibilant()
                .then(hyphen_opt())
                .then(sibilant())
                .then_error(move |span, pair| (span, error1(pair))),
            voiced()
                .then(hyphen_opt())
                .then(unvoiced())
                .then_error(move |span, pair| (span, error2(pair))),
            unvoiced()
                .then(hyphen_opt())
                .then(voiced())
                .then_error(move |span, pair| (span, error3(pair))),
            nil(), // No forbidden pattern found, we successfully parse `()`.
        )))
        // The we parse the actual consonant.
        .then(raw_consonant())
        // An get rid of the `()` produced by `nil`.
        .map(|(_, out)| out)
}

fn initial_consonant<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    nil()
        .then_peek(
            sonorant()
                .then_error(|span, _| (span, "A word cannot start with a sonorant.".to_string()))
                .opt(),
        )
        .then(consonant())
        .map(|(_, c)| c)
}

pub fn initial_pair<S>() -> impl Parser<S, u8, (char, char), Error = super::Error> {
    let err_medial = |(c1, c2)| {
        format!("Expected an initial consonant pair but found '{c1}{c2}' which is a medial consonant pair.")
    };
    let err_triplet = |(c1, c2)| {
        format!(
            "'{c1}{c2}' is a valid initial pair, but it must not be followed by another consonant."
        )
    };
    let err_hyphen = |(c1, c2)| {
        format!(
            "'{c1}{c2}' is a valid initial pair, but it cannot be followed by an hyphen (only a vowel)."
        )
    };

    // We first check for valid pattern.
    nil()
        .then_peek(choice((
            // Valid patterns (might contain pairs forbidden by `consonant`)
            plosive().or(raw_letter(b"fv")).then(sibilant()).map(|_| ()),
            sibilant().then(other().or(sonorant())).map(|_| ()),
            other().then(sonorant()).map(|_| ()),
            // Produce an error if it is a medial pair.
            medial_pair().then_error(move |span, pair| (span, err_medial(pair))),
            // We don't produce an error for remaining invalid pairs, as they
            // are forbidden by rules expressed in `consonant()` which will give
            // a more precise explanation.
        )))
        // Actually parse the pair
        .then(consonant())
        .map(|(_, c)| c)
        .then(consonant())
        // Forbid a following consonant
        .then_peek_with(move |pair| {
            raw_consonant()
                .then_error(move |span, _| (span, err_triplet(pair)))
                .opt()
        })
        // Forbid a following hyphen
        .then_peek_with(move |pair| {
            one_of([b'-'])
                .then_error(move |span, _| (span, err_hyphen(pair)))
                .opt()
        })
}

pub fn medial_pair<S>() -> impl Parser<S, u8, (char, char), Error = super::Error> {
    // We first check for valid pattern.
    nil()
        .then_peek(choice((
            // Valid patterns (might contain pairs forbidden by `consonant`)
            liquid()
                .then(hyphen_opt())
                .then(raw_letter(b"n"))
                .map(|_| ()),
            raw_letter(b"n")
                .then(hyphen_opt())
                .then(liquid())
                .map(|_| ()),
            raw_letter(b"fv")
                .then(hyphen_opt())
                .then(raw_letter(b"m").or(plosive()))
                .map(|_| ()),
            plosive()
                .then(hyphen_opt())
                .then(raw_letter(b"fvm").or(plosive()))
                .map(|_| ()),
            // We just don't parse and don't emit an error, as usually if this
            // is not a medial pair it is an initial pair which starts the next
            // word.
        )))
        // Actually parse the pair
        .then(consonant())
        .map(|(_, c)| c)
        .then(hyphen_opt())
        .map(|(c, _)| c)
        .then(consonant())
    // Can be followed by a consonant in borrowings triplets, thus we don't
    // perform a check.
}

fn hieaou<S>() -> impl Parser<S, u8, String, Error = super::Error> {
    vowel()
        .then(
            hyphen_opt()
                .then(letter_h().opt())
                .then(vowel())
                .map(|((_hyphen, h), vowel)| format!("{}{vowel}", h.map(|_| "h").unwrap_or("")))
                .repeated(..),
        )
        .map(|(v1, tail)| format!("{v1}{}", tail.join("")))
}

pub fn particle_form<S>() -> impl Parser<S, u8, String, Error = super::Error> {
    initial_consonant()
        .then(hieaou())
        // Ensure it is not followed by a sonorant or medial pair, in which case
        // it is not a particle but a root.
        .then_peek(not(choice((
            sonorant().map(|_| ()),
            medial_pair().map(|_| ()),
        ))))
        .map(|(c1, vtail)| format!("{c1}{vtail}"))
}

/// Root segment `(medial_pair / hyphen sonorant) hieaou`.
fn root_segment<S>() -> impl Parser<S, u8, String, Error = super::Error> {
    choice((
        medial_pair().map(|(c1, c2)| format!("{c1}{c2}")).then_peek(
            not(vowel())
                .then_error(|span, _| {
                    (
                        span.expand_after(1),
                        "Inside roots a medial pair must be followed by a vowel.".to_string(),
                    )
                })
                .opt(),
        ),
        hyphen_opt().then(sonorant()).map(|(_, v)| format!("{v}")),
    ))
    .then(hieaou())
    .map(|(c, v)| format!("{c}{v}"))
}

/// Optional final sonorant
fn sonorant_opt<S>() -> impl Parser<S, u8, String, Error = super::Error> {
    sonorant()
        .opt()
        .map(|s| s.map(|s| format!("{s}")).unwrap_or(String::from("")))
}

pub fn root_form<S>() -> impl Parser<S, u8, String, Error = super::Error> {
    choice((
        initial_consonant()
            .then(hieaou())
            .then(sonorant())
            .map(|((c, v), s)| format!("{c}{v}{s}")),
        initial_consonant()
            .then(hieaou())
            .then(root_segment().repeated(1..))
            .then(sonorant_opt())
            .map(|(((c, v), r), s)| format!("{c}{v}{}{s}", r.join(""),)),
        initial_pair()
            .then(hieaou())
            .then(root_segment().repeated(..))
            .then(sonorant_opt())
            .map(|((((c1, c2), v), r), s)| format!("{c1}{c2}{v}{}{s}", r.join(""),)),
    ))
    .then_peek_with(move |word| {
        sonorant()
            .then_error(move |span, s| {
                (
                    span.expand_before(1),
                    format!(
                        "Found word '{word}', which cannot be followed by a sonorant ({}{s} is not \
                            a medial pair).",
                        word.chars().last().expect("word is at least one char long")
                    ),
                )
            })
            .opt()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_pairs() {
        let initial_pairs: Vec<_> = [
            "bj", "bz", "bl", "br", "cf", "ck", "cm", "cp", "ct", "cn", "cl", "cr", "dj", "dz",
            "dr", "fc", "fs", "fn", "fl", "fr", "gj", "gz", "gn", "gl", "gr", "jb", "jd", "jg",
            "jm", "jv", "jn", "jl", "jr", "kc", "ks", "kn", "kl", "kr", "mn", "ml", "mr", "pc",
            "ps", "pl", "pr", "sf", "sk", "sm", "sp", "st", "sn", "sl", "sr", "tc", "ts", "tr",
            "vj", "vz", "vn", "vl", "vr", "zb", "zd", "zg", "zm", "zv", "zn", "zl", "zr",
        ]
        .map(|word| word.as_bytes())
        .into_iter()
        .collect();

        let parser = initial_pair().then_peek(end());

        for c1 in b'a'..=b'z' {
            for c2 in b'a'..=b'z' {
                assert_eq!(
                    parser
                        .parse(&mut IterStream::new([c1, c2]))
                        .option_in_err()
                        .is_ok(),
                    initial_pairs.contains(&&[c1, c2][..]),
                    "Mismatch for {}{}",
                    char::from(c1),
                    char::from(c2)
                );
            }
        }

        // Initial pair cannot contain hyphen.
        let parser = initial_pair().then_peek(end());

        for c1 in b'a'..=b'z' {
            for c2 in b'a'..=b'z' {
                assert_eq!(
                    parser
                        .parse(&mut IterStream::new([c1, b'-', c2]))
                        .option_in_err()
                        .is_ok(),
                    false,
                    "Mismatch for {}-{}, initial pair cannot contain hyphen",
                    char::from(c1),
                    char::from(c2)
                );
            }
        }
    }

    #[test]
    fn medial_pairs() {
        let medial_pairs: Vec<_> = [
            "bd", "bg", "bm", "bv", "db", "dg", "dm", "dv", "fk", "fm", "fp", "ft", "gb", "gd",
            "gm", "gv", "kf", "km", "kp", "kt", "pf", "pk", "pm", "pt", "tf", "tk", "tm", "tp",
            "vb", "vd", "vg", "vm", "nl", "nr", "ln", "rn",
        ]
        .map(|word| word.as_bytes())
        .into_iter()
        .collect();

        let parser = medial_pair().then_peek(end());

        for c1 in b'a'..=b'z' {
            for c2 in b'a'..=b'z' {
                assert_eq!(
                    parser
                        .parse(&mut IterStream::new([c1, c2]))
                        .option_in_err()
                        .is_ok(),
                    medial_pairs.contains(&&[c1, c2][..]),
                    "Mismatch for {}{}",
                    char::from(c1),
                    char::from(c2)
                );
            }
        }

        let parser = medial_pair().then_peek(end());

        for c1 in b'a'..=b'z' {
            for c2 in b'a'..=b'z' {
                assert_eq!(
                    parser
                        .parse(&mut IterStream::new([c1, b'-', c2]))
                        .option_in_err()
                        .is_ok(),
                    medial_pairs.contains(&&[c1, c2][..]),
                    "Mismatch for {}-{}",
                    char::from(c1),
                    char::from(c2)
                );
            }
        }
    }

    #[test]
    fn particle_forms() {
        let parser = particle_form();

        let success = |pat, out| {
            assert_eq!(
                parser.parse(&mut IterStream::new(pat)),
                Ok(Some(String::from(out)))
            )
        };

        success(&b"pa"[..], "pa");
        success(&b"pai"[..], "pai");
        success(&b"pahi"[..], "pahi");
        success(&b"pa-i"[..], "pai");
        success(&b"pa-hi"[..], "pahi");

        success(&b"papa"[..], "pa");
        success(&b"papla"[..], "pa");

        assert_eq!(parser.parse(&mut IterStream::new(&b"p"[..])), Ok(None));
        assert_eq!(parser.parse(&mut IterStream::new(&b"pafka"[..])), Ok(None));

        assert!(parser.parse(&mut IterStream::new(&b"pah-i"[..])).is_err());
        assert!(parser.parse(&mut IterStream::new(&b"pa-ai"[..])).is_err());
        assert!(parser.parse(&mut IterStream::new(&b"na"[..])).is_err());
    }
}
