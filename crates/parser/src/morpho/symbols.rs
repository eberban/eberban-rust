use framework::*;

use crate::SpannedErrorExt;

pub fn hyphen<S>() -> impl Parser<S, u8, (), Error = crate::Error> {
    let single_hyphen = || {
        choice((
            exact_utf8("\u{2010}"), // ‐ HYPHEN
            exact_utf8("\u{2014}"), // — EM DASH
            exact_utf8("\u{002D}"), // - HYPHEN-MINUS
        ))
    };

    let second_hyphen = || {
        single_hyphen()
            .then_error(|span, _| {
                span.expand_before(1)
                    .error("Only one hyphen is allowed in a row.")
            })
            .opt()
    };

    let new_lines = || one_of(b"\n\r").repeated(..);

    single_hyphen()
        .then_peek(second_hyphen())
        .then(new_lines())
        .discard()
}

pub fn hyphen_opt<S>() -> impl Parser<S, u8, (), Error = crate::Error> {
    hyphen().opt().discard()
}

pub fn raw_letter<S>(choices: &[u8]) -> impl Parser<S, u8, char, Error = crate::Error> {
    let repeat_after_hyphen = |pattern| {
        hyphen_opt()
            .then(one_of(pattern))
            .then_error(|span, _| {
                span.expand_before(1)
                    .error("The same letter cannot appear both before and after an hyphen.")
            })
            .opt()
    };

    choice(
        choices
            .into_iter()
            .map(|&c| {
                let pattern = [c.to_ascii_lowercase(), c.to_ascii_uppercase()];
                let out = char::from(c);

                one_of(pattern)
                    .repeated(1..)
                    .then_peek(repeat_after_hyphen(pattern))
                    .map(move |_| out)
            })
            .collect::<Vec<_>>(),
    )
}

pub fn vowel<S>() -> impl Parser<S, u8, char, Error = crate::Error> {
    raw_letter(b"ieaou")
}

pub fn voiced<S>() -> impl Parser<S, u8, char, Error = crate::Error> {
    raw_letter(b"bdgvzj")
}

pub fn unvoiced<S>() -> impl Parser<S, u8, char, Error = crate::Error> {
    raw_letter(b"ptkfsc")
}

pub fn sibilant<S>() -> impl Parser<S, u8, char, Error = crate::Error> {
    raw_letter(b"szcj")
}

pub fn plosive<S>() -> impl Parser<S, u8, char, Error = crate::Error> {
    raw_letter(b"tdkgpb")
}

pub fn sonorant<S>() -> impl Parser<S, u8, char, Error = crate::Error> {
    raw_letter(b"nrl")
}

pub fn liquid<S>() -> impl Parser<S, u8, char, Error = crate::Error> {
    raw_letter(b"lr")
}

pub fn letter_h<S>() -> impl Parser<S, u8, char, Error = crate::Error> {
    raw_letter(b"h")
        .spanned()
        .then_peek_with(move |(span_h, _)| {
            one_of(b"-")
                .then_error(move |_, _| {
                    span_h.expand_after(1).error(
                        "An hyphen cannot appear after an 'h', and should appear before instead.",
                    )
                })
                .opt()
        })
        .map(|(_, h)| h)
}

pub fn other<S>() -> impl Parser<S, u8, char, Error = crate::Error> {
    let error = move |c1, c2| format!("{c1} cannot be followed by {c2} in an initial pair.");

    choice((
        raw_letter(b"vfkgm"),
        raw_letter(b"pb").then_peek_with(move |c1| {
            hyphen_opt().then(
                raw_letter(b"n")
                    .then_error(move |span, c2| span.error(error(c1, c2)))
                    .opt(),
            )
        }),
        raw_letter(b"td").then_peek_with(move |c1| {
            hyphen_opt().then(
                raw_letter(b"nl")
                    .then_error(move |span, c2| span.error(error(c1, c2)))
                    .opt(),
            )
        }),
        raw_letter(b"n").then_peek_with(move |c1| {
            hyphen_opt().then(
                liquid()
                    .then_error(move |span, c2| span.error(error(c1, c2)))
                    .opt(),
            )
        }),
    ))
}

pub fn raw_consonant<S>() -> impl Parser<S, u8, char, Error = crate::Error> {
    raw_letter(b"nrlmpbfvtdszcjgk")
}

/// Parse a single consonant with morphology checks.
pub fn consonant<S>() -> impl Parser<S, u8, char, Error = crate::Error> {
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
                .then_error(move |span, pair| span.error(error1(pair))),
            voiced()
                .then(hyphen_opt())
                .then(unvoiced())
                .then_error(move |span, pair| span.error(error2(pair))),
            unvoiced()
                .then(hyphen_opt())
                .then(voiced())
                .then_error(move |span, pair| span.error(error3(pair))),
            nil(), // No forbidden pattern found, we successfully parse `()`.
        )))
        // The we parse the actual consonant.
        .then(raw_consonant())
        // An get rid of the `()` produced by `nil`.
        .map(|(_, out)| out)
}

pub fn initial_consonant<S>() -> impl Parser<S, u8, char, Error = crate::Error> {
    nil()
        .then_peek(
            sonorant()
                .then_error(|span, _| span.error("A word cannot start with a sonorant."))
                .opt(),
        )
        .then(consonant())
        .map(|(_, c)| c)
}

pub fn initial_pair<S>() -> impl Parser<S, u8, (char, char), Error = crate::Error> {
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
            plosive().or(raw_letter(b"fv")).then(sibilant()).discard(),
            sibilant().then(other().or(sonorant())).discard(),
            other().then(sonorant()).discard(),
            // Produce an error if it is a medial pair.
            medial_pair().then_error(move |span, pair| span.error(err_medial(pair))),
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
                .then_error(move |span, _| span.error(err_triplet(pair)))
                .opt()
        })
        // Forbid a following hyphen
        .then_peek_with(move |pair| {
            one_of(b"-")
                .then_error(move |span, _| span.error(err_hyphen(pair)))
                .opt()
        })
}

pub fn medial_pair<S>() -> impl Parser<S, u8, (char, char), Error = crate::Error> {
    // We first check for valid pattern.
    nil()
        .then_peek(choice((
            // Valid patterns (might contain pairs forbidden by `consonant`)
            liquid().then(hyphen_opt()).then(raw_letter(b"n")).discard(),
            raw_letter(b"n").then(hyphen_opt()).then(liquid()).discard(),
            raw_letter(b"fv")
                .then(hyphen_opt())
                .then(raw_letter(b"m").or(plosive()))
                .discard(),
            plosive()
                .then(hyphen_opt())
                .then(raw_letter(b"fvm").or(plosive()))
                .discard(),
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

pub fn consonant_triplet<S>() -> impl Parser<S, u8, (char, char, char), Error = crate::Error> {
    let consonant_pattern_1 = nil()
        .then_peek(medial_pair())
        .then_peek(not(sonorant()))
        .then(consonant())
        .then(hyphen_opt())
        .then(initial_pair());


    nil()
        .then_peek(choice((consonant_pattern_1,)))
        .then(consonant())
        .then(consonant())
        .then(consonant())
        .map(|(((_, c1), c2), c3)| (c1, c2, c3))
        .spanned()
        .then_peek_with(|(span, (c1, c2, c3))| {
            not(vowel()).then_error(move |_, _| {
                span.error(format!(
                    "'{c1}{c2}{c3}' is a valid triplet, but it should be followed by a vowel."
                ))
            })
        })
        .map(|(_, triplet)| triplet)
}

pub fn hieaou<S>() -> impl Parser<S, u8, String, Error = crate::Error> {
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

pub fn pause<S>() -> impl Parser<S, u8, (), Error = crate::Error> {
    choice((exact_utf8("'"), exact_utf8("’"), exact_utf8("`"))).discard()
}

pub fn single_pause<S>() -> impl Parser<S, u8, (), Error = crate::Error> {
    pause().then_peek(
        pause()
            .then_error(|span, _| {
                span.expand_before(1)
                    .error("Only a single pause symbol can be used in a row.")
            })
            .opt(),
    )
}

pub fn digit<S>() -> impl Parser<S, u8, u8, Error = crate::Error> {
    one_of(b'0'..=b'9').map(|digit| digit - b'0')
}

pub fn space<S>() -> impl Parser<S, u8, (), Error = crate::Error> {
    nil()
        .then_peek(not(choice((
            pause(),
            digit().discard(),
            hyphen(),
            one_of(b'a'..=b'z').discard(),
            one_of(b'A'..=b'Z').discard(),
        ))))
        .then(any())
        .discard()
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
    fn medial_pairs_with_hyphens() {
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
    fn medial_pairs_with_hyphens_and_line_breaks() {
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
                        .parse(&mut IterStream::new([c1, b'-', b'\n', c2]))
                        .option_in_err()
                        .is_ok(),
                    medial_pairs.contains(&&[c1, c2][..]),
                    "Mismatch for {}-[line break]{}",
                    char::from(c1),
                    char::from(c2)
                );
            }
        }
    }
}
