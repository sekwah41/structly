use structly::Structly;

#[derive(Structly)]
struct Demo {
    #[structly(name = "First", name = "Second")]
    label: Option<String>,
}

fn main() {}
