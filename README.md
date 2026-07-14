Structly
======
A struct data validator, designed to give human-readable feedback and documentation generation.

Note: This project was initially designed for use with [relcon](https://codeberg.org/Sekwah/relcon).

We suggest to use serde to serialise and deserialise into basic types and then apply more advanced human-readable
constraints after.

To pair the default serde feedback with this, we will be looking to add a method to pass serde errors in to possibly
enhance them with documentation.

While you are welcome to use it for your own projects, just bare this in mind. Though if you need assistance migrating
your project feel free to ask any questions.

If you have an issue or want to discuss a possible feature/change to the library feel free to create an issue.
I do also check emails at contact@sekwah.com however its more likely to get responded to if it's an issue.

Installation
------

Add the library to your project:

```sh
cargo add structly
```

Install the documentation CLI (installs the `structly` command):

```sh
cargo install structly_cli
```

Generating docs
------

The `structly` CLI generates markdown documentation for a `#[derive(Structly)]`
struct by parsing your source code - it never compiles or runs your project, so
none of the documentation detail ends up in your binaries.

```sh
structly docs --struct AppConfig --path src/ --out docs/config.md
```

- `--struct <Name>` - the struct to document (required)
- `--path <file|dir>` - source file or directory to scan (default: `.`)
- `--out <file>` - output file (default: stdout) (OUTPUT.md is in .gitignored)

Fields marked `#[structly(nested)]` are resolved from the scanned sources and
documented as subsections with dotted paths (e.g. `database.cert_path`).

From this workspace it can be run with `cargo run -p structly_cli -- docs ...`.

Debugging
------

https://github.com/dtolnay/cargo-expand

cargo expand --example basic

