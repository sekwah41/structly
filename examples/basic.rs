use structly_macros::Structly;

#[derive(Structly)]
struct Demo {
    #[structly(name = "Label", description = "This is a fun label description")]
    label: Option<String>,
}

fn main() {
    let d = Demo { label: None };
    println!("{:?}", d.validate());
}