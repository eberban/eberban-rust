use std::ops::Range;

use ariadne::{Color, Label, Report, ReportKind, Source};
use framework::*;
use parser::morpho::{hyphen_opt, initial_pair, medial_pair};

fn main() {
    println!();

    let args: Vec<_> = std::env::args().skip(1).collect();
    let text = args.join(" ");

    let parser = initial_pair();

    println!("Input: {text}");

    let res: Result<_, (Span, String)> =
        parser.parse(&mut IterStream::new(text.clone().into_bytes()));

    match res {
        Ok(ok) => println!("Success: {ok:?}"),
        Err((span, err)) => {
            Report::<Range<usize>>::build(ReportKind::Error, (), 0)
                .with_label(
                    Label::new(span.into())
                        .with_message(err)
                        .with_color(Color::Red),
                )
                .finish()
                .print(Source::from(text))
                .unwrap();
        }
    }
}
