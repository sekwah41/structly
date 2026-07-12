use structly::Structly;

#[derive(Structly)]
struct Demo {
    #[structly(mode = "all", mode = "any")]
    label: Option<String>,
}

fn main() {}
