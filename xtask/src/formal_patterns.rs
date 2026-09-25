use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::Args;
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::ToTokens;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Args, Debug)]
pub struct FormalPatternsArgs {
    #[arg(long, value_delimiter = ',', default_value = "src,tests,formal")]
    roots: Vec<PathBuf>,
    #[arg(long, default_value_t = 3)]
    minimum_occurrences: usize,
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct PatternOccurrence {
    path: String,
    function: String,
}

#[derive(Debug, Serialize)]
struct PatternGroup {
    fingerprint: String,
    occurrences: Vec<PatternOccurrence>,
}

pub fn run(args: FormalPatternsArgs) -> anyhow::Result<()> {
    let mut groups: BTreeMap<u64, Vec<PatternOccurrence>> = BTreeMap::new();
    for root in &args.roots {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                scan_file(path, &mut groups)?;
            }
        }
    }
    let report: Vec<PatternGroup> = groups
        .into_iter()
        .filter(|(_, occurrences)| occurrences.len() >= args.minimum_occurrences)
        .map(|(fingerprint, occurrences)| PatternGroup {
            fingerprint: format!("{fingerprint:016x}"),
            occurrences,
        })
        .collect();
    if let Some(path) = args.output {
        fs::write(&path, serde_json::to_vec_pretty(&report)?)
            .with_context(|| format!("failed to write {}", path.display()))?;
    } else {
        for group in &report {
            println!("{} x{}", group.fingerprint, group.occurrences.len());
            for occurrence in &group.occurrences {
                println!("  {}::{}", occurrence.path, occurrence.function);
            }
        }
    }
    Ok(())
}

fn scan_file(path: &Path, groups: &mut BTreeMap<u64, Vec<PatternOccurrence>>) -> anyhow::Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let file = match syn::parse_file(&source) {
        Ok(file) => file,
        Err(_) => return Ok(()),
    };
    for item in file.items {
        if let syn::Item::Fn(function) = item {
            let normalized = normalize_tokens(function.block.to_token_stream());
            groups.entry(fnv1a64(normalized.as_bytes())).or_default().push(PatternOccurrence {
                path: path.display().to_string(),
                function: function.sig.ident.to_string(),
            });
        }
    }
    Ok(())
}

fn normalize_tokens(stream: TokenStream) -> String {
    fn write_stream(stream: TokenStream, output: &mut String) {
        for token in stream {
            match token {
                TokenTree::Group(group) => {
                    let (open, close) = match group.delimiter() {
                        Delimiter::Parenthesis => ('(', ')'),
                        Delimiter::Brace => ('{', '}'),
                        Delimiter::Bracket => ('[', ']'),
                        Delimiter::None => ('<', '>'),
                    };
                    output.push(open);
                    write_stream(group.stream(), output);
                    output.push(close);
                }
                TokenTree::Ident(ident) => {
                    let text = ident.to_string();
                    if is_rust_keyword(&text) {
                        output.push_str(&text);
                    } else {
                        output.push('I');
                    }
                }
                TokenTree::Punct(punct) => output.push(punct.as_char()),
                TokenTree::Literal(_) => output.push('L'),
            }
            output.push(' ');
        }
    }
    let mut output = String::new();
    write_stream(stream, &mut output);
    output
}

fn is_rust_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "async" | "await" | "break" | "const" | "continue" | "crate" | "dyn"
            | "else" | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl"
            | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub"
            | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super"
            | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while"
    )
}

const fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        index += 1;
    }
    hash
}
