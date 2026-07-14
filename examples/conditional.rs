use structly::{Structly, Verify};

#[allow(unused)]
#[derive(Structly)]
struct Config {
    #[structly_if(when = self.enabled, reason = "name required when enabled")]
    name: Option<String>,

    enabled: bool,
}

fn main() {
    let c = Config { name: None, enabled: true };
    println!("{:?}", c.verify());
}
