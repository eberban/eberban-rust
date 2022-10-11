use framework::*;

fn raw_letter<S, E>(choices: &[u8]) -> impl Parser<S, u8, char, Error = E> {
    choice(
        choices
            .into_iter()
            .map(|&c| {
                let pattern = [c.to_ascii_lowercase(), c.to_ascii_uppercase()];
                let out = char::from(c);

                one_of(pattern).repeated(1..).map(move |_| out)
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

fn other<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    let error = move |c1, c2| format!("{c1} cannot be followed by {c2} in an initial pair.");

    choice((
        raw_letter(b"vfkgm"),
        raw_letter(b"pb").then_peek_with(move |c1| {
            raw_letter(b"n")
                .then_error(move |span, c2| (span, error(c1, c2)))
                .or(nil())
        }),
        raw_letter(b"td").then_peek_with(move |c1| {
            raw_letter(b"nl")
                .then_error(move |span, c2| (span, error(c1, c2)))
                .or(nil())
        }),
        raw_letter(b"n").then_peek_with(move |c1| {
            liquid()
                .then_error(move |span, c2| (span, error(c1, c2)))
                .or(nil())
        }),
    ))
}

fn raw_consonant<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    raw_letter(b"nrlmpbfvtdszcjgk")
}

/// Parse a single consonant with morphology checks.
pub fn consonant<S>() -> impl Parser<S, u8, char, Error = super::Error> {
    let error1 =
        move |(c1, c2)| format!("A sibilant ({c1}) cannot be followed by another sibilant ({c2}).");
    let error2 = move |(c1, c2)| {
        format!("A voiced consonant ({c1}) cannot be followed by an unvoiced one ({c2}).")
    };
    let error3 = move |(c1, c2)| {
        format!("An unvoiced consonant ({c1}) cannot be followed by an voiced one ({c2}).")
    };

    // we use `nil().then_peek(...)` to first check for many forbidden patterns.
    nil()
        .then_peek(choice((
            sibilant()
                .then(sibilant())
                .then_error(move |span, pair| (span, error1(pair))),
            voiced()
                .then(unvoiced())
                .then_error(move |span, pair| (span, error2(pair))),
            unvoiced()
                .then(voiced())
                .then_error(move |span, pair| (span, error3(pair))),
            nil(), // No forbidden pattern found, we successfully parse `()`.
        )))
        // The we parse the actual consonant.
        .then(raw_consonant())
        // An get rid of the `()` produced by `nil`.
        .map(|(_, out)| out)
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
                .or(nil())
        })
}

pub fn medial_pair<S>() -> impl Parser<S, u8, (char, char), Error = super::Error> {
    // We first check for valid pattern.
    nil()
        .then_peek(choice((
            // Valid patterns (might contain pairs forbidden by `consonant`)
            liquid().then(raw_letter(b"n")).map(|_| ()),
            raw_letter(b"n").then(liquid()).map(|_| ()),
            raw_letter(b"fv")
                .then(raw_letter(b"m").or(plosive()))
                .map(|_| ()),
            plosive().then(raw_letter(b"fvm").or(plosive())).map(|_| ()),
            // We just don't parse and don't emit an error, as usually if this
            // is not a medial pair it is an initial pair which starts the next
            // word.
        )))
        // Actually parse the pair
        .then(consonant())
        .map(|(_, c)| c)
        .then(consonant())
        // Can be followed by a consonant in borrowings triplets, thus we don't
        // perform a check.
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
    }

    #[test]
    fn medial_pairs() {
        let initial_pairs: Vec<_> = [
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
                    initial_pairs.contains(&&[c1, c2][..]),
                    "Mismatch for {}{}",
                    char::from(c1),
                    char::from(c2)
                );
            }
        }
    }
}
