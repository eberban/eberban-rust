use ariadne::{Color, Label, Report, ReportKind, Source};
use framework::*;
use parser::morpho::{medial_pair, particle_form, root_form};

fn main() {
    println!();

    let args: Vec<_> = std::env::args().skip(1).collect();
    let text = args.join(" ");

    let parser = root_form();

    println!("Input: {text}");

    let res: Result<_, (Span, String)> =
        parser.parse(&mut IterStream::new(text.clone().into_bytes()));

    match res {
        Ok(ok) => println!("Success: {ok:?}"),
        Err((span, err)) => {
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
