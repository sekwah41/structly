//! `structly docs`: parse a project's source with syn and generate markdown
//! documentation for a `#[derive(Structly)]` struct. Never compiles or runs the
//! target code, so it adds nothing to any application binary.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use structly_core::{build_struct_doc, derives_structly, render_markdown, StructDoc};

struct Args {
    path: PathBuf,
    target: String,
    out: Option<PathBuf>,
}

fn print_usage() {
    eprintln!(
        "Usage: structly docs --struct <Name> [--path <file|dir>] [--out <file>]\n\
         \n\
         Generate markdown documentation for a #[derive(Structly)] struct.\n\
         \n\
         Options:\n\
         \x20 --struct <Name>    Struct to document (required)\n\
         \x20 --path <file|dir>  Source file or directory to scan (default: .)\n\
         \x20 --out <file>       Output file (default: stdout)"
    );
}

fn parse_args() -> Result<Args, String> {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("docs") => {}
        Some(other) => return Err(format!("unknown command `{other}` - expected `docs`")),
        None => return Err("missing command - expected `docs`".to_string()),
    }

    let mut path = PathBuf::from(".");
    let mut target = None;
    let mut out = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--path" => path = PathBuf::from(args.next().ok_or("`--path` requires a value")?),
            "--struct" => target = Some(args.next().ok_or("`--struct` requires a value")?),
            "--out" => out = Some(PathBuf::from(args.next().ok_or("`--out` requires a value")?)),
            other => return Err(format!("unknown argument `{other}`")),
        }
    }

    Ok(Args {
        path,
        target: target.ok_or("`--struct` is required")?,
        out,
    })
}

/// Recursively collect `.rs` files, skipping `target/` and hidden directories.
fn collect_rs_files(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path.to_path_buf());
        }
        return;
    }

    let Ok(entries) = std::fs::read_dir(path) else {
        eprintln!("warning: could not read directory {}", path.display());
        return;
    };
    for entry in entries.flatten() {
        let entry_path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if entry_path.is_dir() {
            if name == "target" || name.starts_with('.') {
                continue;
            }
            collect_rs_files(&entry_path, files);
        } else if name.ends_with(".rs") {
            files.push(entry_path);
        }
    }
}

/// Index every `#[derive(Structly)]` struct by name, recursing into inline modules.
fn collect_structs(items: &[syn::Item], index: &mut HashMap<String, syn::ItemStruct>) {
    for item in items {
        match item {
            syn::Item::Struct(item_struct) if derives_structly(item_struct) => {
                index.insert(item_struct.ident.to_string(), item_struct.clone());
            }
            syn::Item::Mod(module) => {
                if let Some((_, items)) = &module.content {
                    collect_structs(items, index);
                }
            }
            _ => {}
        }
    }
}

fn nested_type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    }
}

/// Build the doc for `name`, recursing into nested sections with a cycle guard.
fn doc_for(
    name: &str,
    index: &HashMap<String, syn::ItemStruct>,
    visited: &mut Vec<String>,
) -> Option<StructDoc> {
    if visited.iter().any(|seen| seen == name) {
        eprintln!("warning: nested cycle at `{name}` - not recursing further");
        return None;
    }
    let item = index.get(name)?;

    visited.push(name.to_string());
    let doc = build_struct_doc(&item.clone(), &mut |ty| {
        let Some(nested_name) = nested_type_name(ty) else {
            eprintln!("warning: unsupported nested field type - expected a plain struct type");
            return None;
        };
        let doc = doc_for(&nested_name, index, visited);
        if doc.is_none() && !index.contains_key(&nested_name) {
            eprintln!("warning: nested type `{nested_name}` is not a #[derive(Structly)] struct in the parsed sources");
        }
        doc
    });
    visited.pop();

    match doc {
        Ok(doc) => Some(doc),
        Err(err) => {
            eprintln!("error: failed to document `{name}`: {err}");
            None
        }
    }
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("error: {err}\n");
            print_usage();
            return ExitCode::from(2);
        }
    };

    let mut files = Vec::new();
    collect_rs_files(&args.path, &mut files);
    if files.is_empty() {
        eprintln!("error: no .rs files found under {}", args.path.display());
        return ExitCode::FAILURE;
    }

    let mut index = HashMap::new();
    for file in &files {
        let source = match std::fs::read_to_string(file) {
            Ok(source) => source,
            Err(err) => {
                eprintln!("warning: skipping {} ({err})", file.display());
                continue;
            }
        };
        match syn::parse_file(&source) {
            Ok(parsed) => collect_structs(&parsed.items, &mut index),
            Err(err) => eprintln!("warning: skipping {} (parse error: {err})", file.display()),
        }
    }

    if !index.contains_key(&args.target) {
        eprintln!(
            "error: no #[derive(Structly)] struct named `{}` found under {}",
            args.target,
            args.path.display()
        );
        let mut available: Vec<_> = index.keys().cloned().collect();
        available.sort();
        if available.is_empty() {
            eprintln!("(no Structly structs were found at all)");
        } else {
            eprintln!("available structs: {}", available.join(", "));
        }
        return ExitCode::FAILURE;
    }

    let mut visited = Vec::new();
    let Some(doc) = doc_for(&args.target, &index, &mut visited) else {
        return ExitCode::FAILURE;
    };

    // Lead with the command that produced the file as a markdown comment to help updating it.
    let mut command = format!(
        "structly docs --struct {} --path {}",
        args.target,
        args.path.display()
    );
    if let Some(out_path) = &args.out {
        command.push_str(&format!(" --out {}", out_path.display()));
    }
    let markdown = format!("[//]: # ({command})\n\n{}", render_markdown(&doc));

    match &args.out {
        Some(out_path) => {
            if let Some(parent) = out_path.parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            if let Err(err) = std::fs::write(out_path, &markdown) {
                eprintln!("error: could not write {}: {err}", out_path.display());
                return ExitCode::FAILURE;
            }
            eprintln!("wrote {}", out_path.display());
        }
        None => print!("{markdown}"),
    }

    ExitCode::SUCCESS
}
