use ebb_parsing_framework::*;

fn main() {
    println!();

    let args: Vec<_> = std::env::args().skip(1).collect();
    let text = args.join(" ");

    let parser = choice((
        one_of("bB").map(|_| "found a b"),
        one_of("cC").map(|_| "found a c"),
    ));

    println!("Input: {text}");

    let res: Result<_, (Span, String)> = parser.parse(&mut IterStream::new(text));
    println!("{:?}", res);
}
