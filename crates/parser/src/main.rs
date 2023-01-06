use {
    ariadne::{Color, Label, Report, ReportKind, Source},
    framework::*,
    parser::{
        morpho::{particle_form, post_word_check, root_form, spaces},
        Error,
    },
};

#[derive(Debug, Clone)]
pub enum Word {
    Root(String),
    Particle(String),
}

fn main() {
    println!();

    let args: Vec<_> = std::env::args().skip(1).collect();
    let text = args.join(" ");

    let parser = choice((
        root_form().map(Word::Root),
        particle_form().map(Word::Particle),
    ))
    .then(post_word_check())
    .then(spaces().opt())
    .map(|((w, _), _)| w)
    .repeated(..);

    println!("Input: {text}");

    let res: Result<_, Error> = parser
        .spanned()
        .parse(&mut IterStream::new(text.clone().into_bytes()));

    match res {
        Ok(ok) => println!("Success: {ok:?}"),
        Err(Error { span, text: err }) => {
            Report::build(ReportKind::Error, "input", 0)
                .with_label(
                    Label::new(("input", span.into()))
                        .with_message(err)
                        .with_color(Color::Red),
                )
                .finish()
                .print(("input", Source::from(text)))
                .unwrap();
        }
    }
}
