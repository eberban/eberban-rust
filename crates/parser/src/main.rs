use framework::*;
use parser::morpho::initial_pair;

fn main() {
    println!();

    let args: Vec<_> = std::env::args().skip(1).collect();
    let text = args.join(" ");

    let parser = initial_pair();

    println!("Input: {text}");

    let res: Result<_, (Span, String)> =
        parser.parse(&mut IterStream::new(text.clone().into_bytes()));
    println!("{:?}", res);
}
