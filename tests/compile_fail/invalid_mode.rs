use structly::Structly;

#[derive(Structly)]
struct Demo {
    #[structly(mode = "sometimes")]
    label: Option<String>,
}

fn main() {}
