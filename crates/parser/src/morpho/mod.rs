use framework::*;

pub mod symbols;

use symbols::*;

use crate::SpannedErrorExt;

pub fn particle_form<S>() -> impl Parser<S, u8, String, Error = super::Error> {
    initial_consonant()
        .then(hieaou())
        // Ensure it is not followed by a sonorant or medial pair, in which case
        // it is not a particle but a root.
        .then_peek(not(choice((sonorant().discard(), medial_pair().discard()))))
        .map(|(c1, vtail)| format!("{c1}{vtail}"))
}

/// Root segment `(medial_pair / hyphen sonorant) hieaou`.
fn root_segment<S>() -> impl Parser<S, u8, String, Error = super::Error> {
    choice((
        medial_pair().map(|(c1, c2)| format!("{c1}{c2}")).then_peek(
            not(vowel())
                .then_error(|span, _| {
                    span.expand_after(1)
                        .error("Inside roots a medial pair must be followed by a vowel.")
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
            .then(root_segment().repeated(1..))
            .then(sonorant_opt())
            .map(|(((c, v), r), s)| format!("{c}{v}{}{s}", r.join(""),)),
        initial_consonant()
            .then(hieaou())
            .then(sonorant())
            .map(|((c, v), s)| format!("{c}{v}{s}")),
        initial_pair()
            .then(hieaou())
            .then(root_segment().repeated(..))
            .then(sonorant_opt())
            .map(|((((c1, c2), v), r), s)| format!("{c1}{c2}{v}{}{s}", r.join(""),)),
    ))
    .then_peek_with(move |word| {
        sonorant()
            .then_error(move |span, s| {
                span.expand_before(1).error(format!(
                    "Found word '{word}', which cannot be followed by a sonorant ({}{s} is not \
                            a medial pair).",
                    word.chars().last().expect("word is at least one char long")
                ))
            })
            .opt()
    })
}

fn pause_before_vowel<S>() -> impl Parser<S, u8, (), Error = super::Error> {
    single_pause().then_peek(not(vowel()).then_error(|span, _| {
        span.expand_before(1)
            .error("Expected a vowel after pause symbol")
    }))
}

pub fn spaces<S>() -> impl Parser<S, u8, (), Error = super::Error> {
    let hesitation = || raw_letter(b"n").then_peek(space());

    choice((
        symbols::space()
            .then(hesitation().opt())
            .repeated(1..)
            .then(pause_before_vowel().opt())
            .discard(),
        pause_before_vowel(),
        end(),
    ))
    .discard()
}

pub fn post_word_check<S>() -> impl Parser<S, u8, (), Error = super::Error> {
    nil().then_peek(choice((
        vowel()
            .then_error(|span, _| span.error("Stopped parsing word then found a vowel (bug)"))
            .opt()
            .discard(),
        pause_before_vowel(),
        nil().then_peek(not(sonorant())).then_peek(consonant()),
        spaces(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

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
