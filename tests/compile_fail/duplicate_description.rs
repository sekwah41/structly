use structly::Structly;

#[derive(Structly)]
struct Demo {
    #[structly(description = "one", description = "two")]
    label: Option<String>,
}

fn main() {}
