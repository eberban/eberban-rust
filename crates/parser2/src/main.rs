use {easy_ext::ext, framework::*};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub span: Span,
    pub text: String,
}

#[ext(SpannedErrorExt)]
pub impl Span {
    fn error(self, text: impl ToString) -> Error {
        Error {
            span: self,
            text: text.to_string(),
        }
    }
}

rules!(
    /// `other` group for consonant pairs.
    initial_other[u8 => char] = nil()
        .then_peek(deny_initial_other_forbidden_patterns())
        .then(letter(b"pbtdvfkgmn"))
        .map(|(_, out)| out);
    deny_initial_other_forbidden_patterns[u8 => ()] = deny(
        choice((
            hyphen_pair(letter(b"pb"), letter(b"n")),
            hyphen_pair(letter(b"td"), letter(b"nl")),
            hyphen_pair(letter(b"n"), liquid()),
        )),
        move |span, (c1,c2)| span.error(format!("{c1} cannot be followed by {c2} in an initial pair."))
    );

    /// A consonant that is checked to not be followed by forbidden consonants.
    pub checked_consonant[u8 => char] = {
        nil()
            .then_peek(choice((deny_unvoiced_voiced(), deny_voiced_unvoiced(), deny_sibilant_pair())))
            .then(consonant())
            .map(|(_nil, c)| c)
    };
    deny_unvoiced_voiced[u8 => ()] = deny(
        hyphen_pair(unvoiced(), voiced()),
        move |span, (c1,c2)| span.error(format!("An unvoiced consonant ({c1}) cannot be followed by a voiced one ({c2})."))
    );
    deny_voiced_unvoiced[u8 => ()] = deny(
        hyphen_pair(voiced(), unvoiced()),
        move |span, (c1,c2)| span.error(format!("A voiced consonant ({c1}) cannot be followed by an unvoiced one ({c2})."))
    );
    deny_sibilant_pair[u8 => ()] = deny(
        hyphen_pair(sibilant(), sibilant()),
        move |span, (c1,c2)| span.error(format!("A sibilant ({c1}) cannot be followed by another sibilant ({c2})."))
    );

    // Common groups of letters.
    pub vowel[u8 => char] = letter(b"ieaou");
    pub consonant[u8 => char] = letter(b"nrlmpbfvtdszcjgk");

    pub voiced[u8 => char] = letter(b"bdgvzj");
    pub unvoiced[u8 => char] = letter(b"ptkfsc");
    pub sibilant[u8 => char] = letter(b"szcj");
    pub plosive[u8 => char] = letter(b"tdkgpb");
    pub sonorant[u8 => char] = letter(b"nrl");
    pub liquid[u8 => char] = letter(b"lr");

    /// Letter H.
    /// Ensure that it is not followed by a hyphen.
    pub h[u8 => char] = letter(b"h").then_peek(deny_hyphen_after_h());
    deny_hyphen_after_h[u8 => ()] = deny(
        one_of(b"-"),
        |span, _| span.error("An hyphen cannot appear after an 'h', and should appear before instead.")
    );

    /// A letter in an Eberban, among one of the `choices`.
    /// Support the same letter being repeated, both in uppercase or lowercase.
    /// Ensure that if followed by an hyphen, the same letter is not repeated
    /// again.
    pub letter(choices: &[u8])[u8 => char] = choice(
        choices
            .iter()
            .map(|&c| {
                let pattern = [c.to_ascii_lowercase(), c.to_ascii_uppercase()];
                let out = char::from(c);

                one_of(pattern)
                    .repeated(1..)
                    .then_peek(deny_letter_repeat_after_pattern(pattern))
                    .map(move |_| out)
            })
            .collect::<Vec<_>>(),
    );
    deny_letter_repeat_after_pattern(pattern: [u8; 2])[u8 => ()] = deny(
        hyphen().then(one_of(pattern)),
        |span, _| span.error("The same letter cannot appear both before and after an hyphen.")
    );

    /// Support for hyphenation. Parse if there is either zero or 1 hyphen symbol.
    pub hyphen[u8 => ()] = single_hyphen().then_peek(deny_repeated_hyphen()).then(line_break()).opt().discard();
    deny_repeated_hyphen[u8 => ()] = deny(
        single_hyphen(),
        |span, _| span.error("Only one hyphen is allowed in a row.")
    );
    single_hyphen[u8 => ()] = choice((
        exact_utf8("\u{2010}"), // ‐ HYPHEN
        exact_utf8("\u{2014}"), // — EM DASH
        exact_utf8("\u{002D}"), // - HYPHEN-MINUS
    )).discard();

    /// Line breaks. Any combinaison of `\n` and `\r` is supported.
    pub line_break[u8 => ()] = one_of(b"\n\r").repeated(..).discard();
);

fn hyphen_pair<S, A, B>(first: A, second: B) -> impl Parser<S, u8, (char, char), Error = Error>
where
    A: Parser<S, u8, char, Error = Error>,
    B: Parser<S, u8, char, Error = Error>,
{
    first
        .then(hyphen())
        .then(second)
        .map(|((c1, _), c2)| (c1, c2))
}

fn main() {
    println!("Hello, world!");
}
