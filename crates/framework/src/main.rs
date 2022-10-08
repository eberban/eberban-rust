use ebb_parsing_framework::*;

fn main() {
    println!();

    let args: Vec<_> = std::env::args().skip(1).collect();
    let text = args.join(" ");

    // let parser = one_of::<_, _, (Span, String)>("a")
    //     .then(one_of("b"))
    //     .map(|(a, b)| format!("{a}{b}"))
    //     .then_peek_with(|before| {
    //         one_of("c")
    //             .then_error(move |span, v| (span, format!("'{before}' cannot be followed by {v}")))
    //     });

    // let parser = one_of::<_, _, (Span, String)>("a")
    //     .repeated(2..5)
    //     .or_error(|span| (span, "found less than 2 'a'".to_string()))
    //     .then_peek_with(|vec| {
    //         end().or_error(move |span| {
    //             (
    //                 span,
    //                 format!(
    //                     "'{}' is followed by something else",
    //                     vec.iter().collect::<String>()
    //                 ),
    //             )
    //         })
    //     })
    //     .map(|vec| vec.into_iter().collect::<String>());

    let rec = recursive(|rec| {
        one_of("a")
            .then(one_of("c").map(|c| c.to_string()).or(rec))
            .map(|(a, b)| format!("{a}{b}"))
    });

    let parser = one_of("a").then(rec);

    println!("Input: {text}");

    let res: Result<_, (Span, String)> = parser.parse(&mut IterStream::new(text));
    println!("{:?}", res);
}
