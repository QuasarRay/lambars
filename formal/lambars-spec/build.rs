use std::{env, fs, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
struct Catalog {
    count: usize,
    specs: Vec<Spec>,
}

#[derive(Deserialize)]
struct Spec {
    id: String,
    name: String,
    catalog_file: String,
    section: String,
    requirement: String,
    kind: String,
    pattern: String,
}

fn main() {
    println!("cargo:rerun-if-changed=../specs/catalog.json");
    let json = fs::read_to_string("../specs/catalog.json").expect("read formal catalog");
    let catalog: Catalog = serde_json::from_str(&json).expect("parse formal catalog");
    assert_eq!(catalog.count, 2_533);
    assert_eq!(catalog.specs.len(), 2_533);

    let mut rust = String::from("pub static SPECS: &[FormalSpec] = &[\n");
    for spec in &catalog.specs {
        rust.push_str("FormalSpec {\n");
        rust.push_str(&format!("id: {:?},\n", spec.id));
        rust.push_str(&format!("name: {:?},\n", spec.name));
        rust.push_str(&format!("catalog_file: {:?},\n", spec.catalog_file));
        rust.push_str(&format!("section: {:?},\n", spec.section));
        rust.push_str(&format!("requirement: {:?},\n", spec.requirement));
        rust.push_str(&format!("kind: SpecKind::{},\n", spec.kind));
        rust.push_str(&format!("pattern: PatternKind::{},\n", spec.pattern));
        rust.push_str("required_backends: BackendSet::BOTH,\n},\n");
    }
    rust.push_str("];\n");
    rust.push_str(&format!(
        "pub const SPEC_COUNT: usize = {};\n",
        catalog.specs.len()
    ));

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    fs::write(out.join("formal_catalog.rs"), rust).expect("write generated formal catalog");
}
