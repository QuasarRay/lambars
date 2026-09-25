use std::{collections::BTreeSet, fs, path::PathBuf};

use anyhow::{Context, bail};
use clap::{Args, ValueEnum};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FormalCatalogAction {
    Validate,
    Export,
}

#[derive(Args, Debug)]
pub struct FormalCatalogArgs {
    #[arg(value_enum, default_value_t = FormalCatalogAction::Validate)]
    action: FormalCatalogAction,
    #[arg(long, default_value = "formal/specs/catalog.json")]
    catalog: PathBuf,
    #[arg(long, default_value = "target/formal-spec-catalog.json")]
    output: PathBuf,
}

#[derive(Deserialize)]
struct Catalog {
    schema_version: u32,
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
    verification: String,
}

pub fn run(args: FormalCatalogArgs) -> anyhow::Result<()> {
    match args.action {
        FormalCatalogAction::Validate => validate(&args.catalog),
        FormalCatalogAction::Export => {
            let bytes = fs::read(&args.catalog)?;
            if let Some(parent) = args.output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&args.output, bytes)?;
            println!("exported {}", args.output.display());
            Ok(())
        }
    }
}

fn validate(path: &PathBuf) -> anyhow::Result<()> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let catalog: Catalog =
        serde_json::from_slice(&bytes).context("failed to parse formal catalog")?;
    if catalog.schema_version != 1 {
        bail!(
            "unsupported formal catalog schema version {}",
            catalog.schema_version
        );
    }
    if catalog.count != 2_533 || catalog.specs.len() != 2_533 {
        bail!("formal catalog must contain exactly 2533 specs");
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for spec in &catalog.specs {
        if !ids.insert(&spec.id) {
            bail!("duplicate formal spec id {}", spec.id);
        }
        if !names.insert(&spec.name) {
            bail!("duplicate formal spec name {}", spec.name);
        }
        if spec.catalog_file.is_empty() || spec.section.is_empty() || spec.requirement.is_empty() {
            bail!("formal spec {} contains empty metadata", spec.name);
        }
        if spec.verification != "BothBackends" {
            bail!("formal spec {} does not require both backends", spec.name);
        }
    }
    println!("validated {} formal specifications", catalog.specs.len());
    Ok(())
}
