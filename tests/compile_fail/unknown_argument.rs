use structly::Structly;

#[derive(Structly)]
struct Demo {
    #[structly(nickname = "oops")]
    label: Option<String>,
}

fn main() {}
