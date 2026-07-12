
use structly::{Structly, Verify};

#[allow(unused)]
#[derive(Structly)]
struct Demo {
    #[structly(name = "Label", description = "This is a fun label description")]
    label: Option<String>,

    #[structly(name = "Another Label", description = "This is a fun label description too")]
    another_label: Option<String>,
}

fn main() {
    let d = Demo { label: None, another_label: None };
    println!("{:?}", d.verify());
}