use crate::{
    symbol::{is_sonorant, parse_symbol, Symbol},
    utils::{err, Reader, Result, Span, Spanned},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lexeme {
    Particle(String),
    Root(String),
    Borrowing(String),
    FreeformVariable(String),
    Compound(Vec<Lexeme>),
    ForeignQuote { delimiter: String, quote: Vec<u8> },
    SpellingQuote(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexerState {
    /// Start of the lexing stage, or after a word ending with a space.
    Space,
    /// Encountered one consonant.
    C(usize, u8),
    /// Encountered an initial consonant pair.
    CI(usize, u8, u8),
    /// After a vowel have been processed.    
    V(usize),
    /// Hyphen after a vowel
    VH(usize),
    /// Sonorant after a vowel
    VS(usize, u8),
}

pub struct Lexer<'a> {
    reader: &'a mut Reader<'a, u8>,
    state: LexerState,
    current_word: Vec<u8>,
    word_start: usize,
    mode: Mode,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    ParticleOrRoot,
    Root,
    Freeform { borrowing: bool },
}

impl Mode {
    fn root_pattern_detected(&mut self) {
        if *self == Self::ParticleOrRoot {
            *self = Self::Root;
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(reader: &'a mut Reader<'a, u8>) -> Self {
        Self {
            word_start: reader.cursor(),
            reader,
            state: LexerState::Space,
            current_word: Vec::new(),
            mode: Mode::ParticleOrRoot,
        }
    }

    pub fn next_lexeme(&mut self) -> Result<Option<Spanned<Lexeme>>> {
        loop {
            if self.reader.peek().is_none() {
                let end = self.reader.cursor();
                let span: Span = (self.word_start..end).into();

                match self.state {
                    LexerState::Space => return Ok(None),
                    LexerState::V(_) => (),
                    LexerState::C(_, _) => return err(span, "TODO: Text ending with consonant"),
                    LexerState::VS(_, s) => {
                        self.current_word.push(s);
                        self.mode.root_pattern_detected();
                    },
                    LexerState::CI(s, _, _) => return err(s..end, "Text ended with incomplete lexeme: lexemes cannot end with a consonant pair."),
                    LexerState::VH(s) => return err(s..end, "Text ended with incomplete lexeme: lexemes cannot end with H.")
                }

                let lexeme = self.end_lexeme(0);

                self.state = LexerState::Space;
                self.word_start = end;

                return Ok(Some(span.wrap(lexeme)));
            }

            let Spanned { value, span } = parse_symbol(self.reader)?;
            let Span { start, end } = span;

            println!("    {:?} - {span:?} - {value:?}", self.state);

            self.state = match (self.state, value) {
            (LexerState::Space, Symbol::Space) => {
                self.word_start = end;
                LexerState::Space
            },
            (LexerState::Space, Symbol::Consonant(c)) => LexerState::C(span.start, c),
            (LexerState::Space, Symbol::Vowel(_v)) => return err(span, "todo space + vowel"),
            (LexerState::Space, Symbol::Pause) => return err(span, "todo space + pause"),
            (LexerState::Space, Symbol::Sonorant(_)) => return err(span, "A word cannot start with a sonorant."),
            (LexerState::Space, Symbol::Hyphen) => return err(span, "A word cannot start with an hyphen."),
            (LexerState::Space, Symbol::H) => return err(span, "A word cannot start with an H."),
            // C
            (LexerState::C(s, c1), Symbol::Consonant(c2)) if is_initial_pair(c1, c2) => {
                self.mode.root_pattern_detected();
                LexerState::CI(s, c1, c2)
            }
            (LexerState::C(s, c), Symbol::Vowel(v)) => {
                self.current_word.extend_from_slice(&[c, v]);
                self.word_start = s;
                LexerState::V(start)
            }
            (LexerState::C(s, _), Symbol::H) => return err(s.. end, "An H cannot appear after the first consonant of a word."),
            (LexerState::C(s, _), Symbol::Hyphen) => return err(s.. end, "An hyphen cannot appear after the first consonant of a word."),
            (LexerState::C(s, _), Symbol::Space) => return err(s.. end, "A space cannot appear after the first consonant of a word."),
            (LexerState::C(s, _), Symbol::Pause) => return err(s..end, "A pause cannot appear after the first consonant of a word."),
            (LexerState::C(s, c1), Symbol::Consonant(c2)) => {
                return err(s..end, format!("A word can only start with 2 consonants if they form an initial pair, which '{}{}' is not.", c1 as char, c2 as char))
            }
            (LexerState::C(s, c1), Symbol::Sonorant(c2)) => {
                if !is_initial_pair(c1, c2) {
                    return err(s..end, format!("A word can only start with a consonant followed by a sonorant if they form an initial pair, which '{}{}' is not.", c1 as char, c2 as char))
                }

                return err(s..end, format!("TODO: CS"))
            }
            // CI
            (LexerState::CI(_, c1,c2), Symbol::Vowel(v)) => {
                self.current_word.extend_from_slice(&[c1, c2, v]);
                LexerState::V(start)
            }
            (LexerState::CI(s, _,_), Symbol::H) => return err(s..end, "An H cannot appear after an initial pair."),
            (LexerState::CI(s, _,_), Symbol::Hyphen) => return err(s..end, "An hyphen cannot appear after an initial pair."),
            (LexerState::CI(s, _,_), Symbol::Space) => return err(s..end, "A space cannot appear after an initial pair."),
            (LexerState::CI(s, _,_), Symbol::Consonant(_)) => return err(s..end, "A consonant cannot appear after an initial pair."),
            (LexerState::CI(s, _,_), Symbol::Sonorant(_)) => return err(s..end, "A sonorant cannot appear after an initial pair."),
            (LexerState::CI(s, _,_), Symbol::Pause) => return err(s..end, "A pause cannot appear after an initial pair."),
            // V
            (LexerState::V(_), Symbol::Vowel(v)) => {
                self.current_word.push(v);
                LexerState::V(start)
            }
            (LexerState::V(_), Symbol::Consonant(_)) => return err(span, "todo vowel + consonant"),
            (LexerState::V(s), Symbol::Sonorant(c)) => {
                self.mode.root_pattern_detected();
                LexerState::VS(s, c)
            },
            (LexerState::V(_), Symbol::Pause) => return err(span, "todo vowel + pause"),
            (LexerState::V(_), Symbol::Hyphen) => return err(span, "todo vowel + hyphen"),
            (LexerState::V(_), Symbol::Space) => {
                let span: Span = (self.word_start..start).into();
                let lexeme = self.end_lexeme(0);

                self.state = LexerState::Space;
                self.word_start = start;

                return Ok(Some(span.wrap(lexeme)));
            },
            (LexerState::V(s), Symbol::H) => LexerState::VH(s),
            // VH
            (LexerState::VH(_), Symbol::Vowel(v)) => {
                self.current_word.extend_from_slice(&[b'h', v]);
                LexerState::V(start)
            }
            (LexerState::VH(s), _) => return err(s..end, "An H can only be followed by a vowel."),
            // VS
            (LexerState::VS(_, c), Symbol::Vowel(v)) => {
                self.current_word.extend_from_slice(&[c, v]);
                LexerState::V(start)
            }
            (LexerState::VS(_, c), Symbol::Consonant(c1)) => {
                self.current_word.extend_from_slice(&[c]);
                let span: Span = (self.word_start..start).into();
                let lexeme = self.end_lexeme(0);

                self.state = LexerState::C(start, c1);
                self.word_start = start;

                return Ok(Some(span.wrap(lexeme)));
            }
            (LexerState::VS(_, c1), Symbol::Sonorant(c2)) if is_medial_pair(c1, c2) => {
                return err(span, "todo sonorant medial pair");
            }
            (LexerState::VS(s, c1), Symbol::Sonorant(c2)) => {
                return err(s..end, format!("2 sonorants are only allowed in a row if they form a medial pair, which is not the case of '{}{}'", c1 as char, c2 as char));
            }
            (LexerState::VS(s, _), Symbol::H) => return err(s..end, "An H can only appear between 2 vowels."),
            (LexerState::VS(_, _), Symbol::Pause) => return err(span, "todo sonorant followed by pause"),
            (LexerState::VS(_, _), Symbol::Hyphen) => return err(span, "todo sonorant followed by hyphen"),
            (LexerState::VS(_, c), Symbol::Space) => {
                self.current_word.extend_from_slice(&[c]);
                let span: Span = (self.word_start..start).into();
                let lexeme = self.end_lexeme(0);

                self.state = LexerState::Space;
                self.word_start = start;

                return Ok(Some(span.wrap(lexeme)));
            }
            }
        }
    }

    fn end_lexeme(&mut self, leftover: usize) -> Lexeme {
        // We extract the last N characters.
        let mut split = self
            .current_word
            .split_off(self.current_word.len() - leftover);

        // Those last N characters are what are left in the current_word buffer.
        std::mem::swap(&mut split, &mut self.current_word);

        let lexeme = String::from_utf8(split).expect("constructed manually, should always be utf8");

        let lexeme = match self.mode {
            Mode::ParticleOrRoot => Lexeme::Particle(lexeme),
            Mode::Root => Lexeme::Root(lexeme),
            Mode::Freeform { borrowing: true } => Lexeme::Borrowing(lexeme),
            Mode::Freeform { borrowing: false } => Lexeme::FreeformVariable(lexeme),
        };

        self.mode = Mode::ParticleOrRoot;

        lexeme
    }
}

fn is_sibilant(c: u8) -> bool {
    b"csjz".contains(&c)
}

fn is_plosive(c: u8) -> bool {
    b"tdkgpb".contains(&c)
}

fn is_other(c: u8) -> bool {
    b"pbtdvfkgmn".contains(&c)
}

fn is_liquid(c: u8) -> bool {
    b"lr".contains(&c)
}

fn is_voiced(c: u8) -> bool {
    b"bdgvzj".contains(&c)
}

fn is_unvoiced(c: u8) -> bool {
    b"ptkfsc".contains(&c)
}

fn is_initial_pair(c1: u8, c2: u8) -> bool {
    if c1 == c2 {
        return false;
    }

    if (is_voiced(c1) && is_unvoiced(c2)) || (is_unvoiced(c1) && is_voiced(c2)) {
        return false;
    }

    if (is_plosive(c1) || b"fv".contains(&c1)) && is_sibilant(c2) {
        return true;
    }

    if is_sibilant(c1) && (is_other(c2) || is_sonorant(c2)) {
        return true;
    }

    if (b"pb".contains(&c1) && c2 == b'n')
        || (c1 == b'n' && is_liquid(c2))
        || (b"td".contains(&c1) && b"nl".contains(&c2))
    {
        return false;
    }

    if is_other(c1) && is_sonorant(c2) {
        return true;
    }

    return false;
}

fn is_medial_pair(_c1: u8, _c2: u8) -> bool {
    todo!()
}
