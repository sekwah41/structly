
use structly::{Structly, Verify};

// Just to stop it complaining for now
#[allow(unused)]
#[derive(Structly)]
struct Demo {
    #[structly(name = "Label", description = "This is a fun label description")]
    label: Option<String>,
}

fn main() {
    let d = Demo { label: None };
    println!("{:?}", d.verify());
}