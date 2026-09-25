use std::{collections::BTreeSet, fs, path::PathBuf};

use anyhow::{Context, bail};
use clap::Args;
use serde::Deserialize;

#[derive(Args, Debug)]
pub struct FormalCoverageArgs {
    #[arg(long, default_value = "formal/specs/coverage.json")]
    coverage: PathBuf,
    #[arg(long)]
    require_complete: bool,
}

#[derive(Deserialize)]
struct Coverage {
    schema_version: u32,
    catalog_count: usize,
    summary: Summary,
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct Summary {
    kani_implementation_proved: usize,
    verus_model_proved: usize,
    verus_direct_refinement_proved: usize,
    fully_paired_directly_discharged: usize,
    pending_kani: usize,
    pending_verus: usize,
    external_gate_required: usize,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    name: String,
    kani: String,
    verus: String,
    direct_verus_refinement: String,
    external_gate_required: bool,
}

pub fn run(args: FormalCoverageArgs) -> anyhow::Result<()> {
    let bytes = fs::read(&args.coverage)
        .with_context(|| format!("failed to read {}", args.coverage.display()))?;
    let coverage: Coverage =
        serde_json::from_slice(&bytes).context("failed to parse formal coverage ledger")?;
    if coverage.schema_version != 1 {
        bail!(
            "unsupported formal coverage schema {}",
            coverage.schema_version
        );
    }
    if coverage.catalog_count != 2_533 || coverage.entries.len() != 2_533 {
        bail!("coverage ledger must contain exactly 2533 entries");
    }

    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for entry in &coverage.entries {
        if !ids.insert(&entry.id) {
            bail!("duplicate coverage id {}", entry.id);
        }
        if !names.insert(&entry.name) {
            bail!("duplicate coverage name {}", entry.name);
        }
        if !matches!(entry.kani.as_str(), "implementation_proved" | "pending") {
            bail!("invalid Kani status {} for {}", entry.kani, entry.name);
        }
        if !matches!(
            entry.verus.as_str(),
            "model_proved" | "implementation_proved" | "pending"
        ) {
            bail!("invalid Verus status {} for {}", entry.verus, entry.name);
        }
        if !matches!(
            entry.direct_verus_refinement.as_str(),
            "proved" | "pending" | "not_started"
        ) {
            bail!(
                "invalid Verus refinement status {} for {}",
                entry.direct_verus_refinement,
                entry.name
            );
        }
    }

    let kani = coverage
        .entries
        .iter()
        .filter(|e| e.kani == "implementation_proved")
        .count();
    let verus = coverage
        .entries
        .iter()
        .filter(|e| e.verus == "model_proved" || e.verus == "implementation_proved")
        .count();
    let direct_verus = coverage
        .entries
        .iter()
        .filter(|e| e.direct_verus_refinement == "proved")
        .count();
    let pending_kani = coverage
        .entries
        .iter()
        .filter(|e| e.kani == "pending")
        .count();
    let pending_verus = coverage
        .entries
        .iter()
        .filter(|e| e.verus == "pending")
        .count();
    let external = coverage
        .entries
        .iter()
        .filter(|e| e.external_gate_required)
        .count();

    if kani != coverage.summary.kani_implementation_proved
        || verus != coverage.summary.verus_model_proved
        || direct_verus != coverage.summary.verus_direct_refinement_proved
        || pending_kani != coverage.summary.pending_kani
        || pending_verus != coverage.summary.pending_verus
        || external != coverage.summary.external_gate_required
    {
        bail!("coverage summary does not match per-spec statuses");
    }

    println!(
        "formal coverage: Kani {kani}/2533 implementation proofs; Verus {verus}/2533 models; direct Verus refinement {direct_verus}/2533; paired direct {}",
        coverage.summary.fully_paired_directly_discharged,
    );

    if args.require_complete
        && (pending_kani != 0
            || pending_verus != 0
            || direct_verus != 2_533
            || coverage.summary.fully_paired_directly_discharged != 2_533)
    {
        bail!("formal coverage is incomplete");
    }
    Ok(())
}
