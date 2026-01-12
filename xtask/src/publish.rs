use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use xshell::{cmd, Shell};

#[derive(Debug, Clone)]
struct Crate {
    name: String,
    path: PathBuf,
    local_deps: Vec<String>,
}

pub fn run_publish_parallel(sh: &Shell, dry_run: bool) -> Result<()> {
    println!("🚀 Starting Parallel Publish (Dry Run: {})...", dry_run);

    // 1. Load Workspace
    let crates = load_workspace()?;
    println!("📦 Found {} crates in workspace.", crates.len());

    // 2. Build Dependency Graph & Layers
    let layers = topological_sort(&crates)?;

    println!("📊 Computed {} dependency layers.", layers.len());
    for (i, layer) in layers.iter().enumerate() {
        let names: Vec<_> = layer.iter().map(|c| &c.name).collect();
        println!("   Layer {}: {:?}", i, names);
    }

    // 3. Execute Layers Sequentially
    for (i, layer) in layers.iter().enumerate() {
        println!("\n▶️  Executing Layer {} ({})", i, layer.len());

        // Execute crates in this layer in parallel
        layer
            .par_iter()
            .try_for_each(|krate| -> Result<()> { publish_crate(dry_run, krate) })?;

        if !dry_run && i < layers.len() - 1 {
            println!("⏳ Waiting 15s for crates.io index propagation...");
            std::thread::sleep(std::time::Duration::from_secs(15));
        }
    }

    println!("\n✅ Parallel Publish Complete!");
    Ok(())
}

fn publish_crate(dry_run: bool, krate: &Crate) -> Result<()> {
    // We need a thread-local Shell because Shell is not Sync/Send usually?
    // Actually xshell::Shell is !Sync. We can use std::process or create new Shell.
    // Creating new Shell is cheap.
    let sh = Shell::new()?;
    let _guard = sh.push_dir(&krate.path);

    let msg = if dry_run { "Dry Publish" } else { "PUBLISHING" };
    println!("   [{}] {}...", msg, krate.name);

    if dry_run {
        cmd!(sh, "cargo publish --dry-run --allow-dirty").run()?;
    } else {
        let mut attempts = 0;
        let max_attempts = 3;

        loop {
            attempts += 1;
            // Use std::process::Command to capture stderr and check for existing version
            let output = std::process::Command::new("cargo")
                .arg("publish")
                .current_dir(&krate.path)
                .output()?;

            if output.status.success() {
                break;
            }

            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("is already uploaded") || stderr.contains("already exists") {
                println!("   [SKIP] {} (already published)", krate.name);
                break;
            }

            if attempts >= max_attempts {
                anyhow::bail!(
                    "Command `cargo publish` failed for {} after {} attempts: {}\n{}",
                    krate.name,
                    attempts,
                    output.status,
                    stderr
                );
            }

            println!(
                "   [RETRY] {} (attempt {}/{}) - waiting 5s...",
                krate.name, attempts, max_attempts
            );
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }

    Ok(())
}

fn load_workspace() -> Result<Vec<Crate>> {
    let mut crates = Vec::new();

    // We assume standard layout: crates/* and current dir (for root/facade)
    // 1. Scan crates/ directory
    let crates_dir = Path::new("crates");
    if crates_dir.exists() {
        for entry in fs::read_dir(crates_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.join("Cargo.toml").exists() {
                crates.push(parse_crate(&path)?);
            }
        }
    }

    Ok(crates)
}

fn parse_crate(path: &Path) -> Result<Crate> {
    let manifest_path = path.join("Cargo.toml");
    let content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read {:?}", manifest_path))?;

    let doc = content.parse::<toml_edit::DocumentMut>()?;

    let name = doc["package"]["name"]
        .as_str()
        .context("Missing package name")?
        .to_string();

    let mut local_deps = Vec::new();

    // Check dependencies section
    if let Some(deps) = doc.get("dependencies").and_then(|d| d.as_table()) {
        for (dep_name, dep_info) in deps.iter() {
            // Check if it has 'path' or 'workspace = true'
            // If workspace=true, we assume it's a local crate deps?
            // Yes, checking if it is in our workspace list later is better,
            // but for now retrieve all potential internal deps.

            // Simplified: We verify later if this dep is in our `crates` list.
            local_deps.push(dep_name.to_string());
        }
    }

    Ok(Crate {
        name,
        path: path.to_path_buf(),
        local_deps,
    })
}

fn topological_sort(crates: &[Crate]) -> Result<Vec<Vec<Crate>>> {
    let mut layers = Vec::new();
    let mut remaining: HashMap<String, Crate> =
        crates.iter().map(|c| (c.name.clone(), c.clone())).collect();

    let all_names: HashSet<String> = remaining.keys().cloned().collect();

    while !remaining.is_empty() {
        // Find crates with NO remaining local dependencies
        let layer: Vec<Crate> = remaining
            .values()
            .filter(|c| {
                c.local_deps.iter().all(|dep_name| {
                    // It's satisfied if it's NOT in 'remaining' (i.e., already processed)
                    // OR if it's not a workspace crate at all (external dep)
                    !remaining.contains_key(dep_name)
                    // Note: If we filter local_deps during parsing to ONLY include workspace members,
                    // verification is simpler. But here 'local_deps' has all deps.
                    // We check if dep_name is in 'all_names'.
                    // If dep is external (e.g. 'serde'), it's not in 'all_names', so !contains_key works.
                    // If dep is internal (e.g. 'core') and unprocessed, it IS in 'remaining', so contains_key is true.
                })
            })
            .cloned()
            .collect();

        if layer.is_empty() {
            return Err(anyhow::anyhow!(
                "Cycle detected or unresolvable dependencies! Remaining: {:?}",
                remaining.keys()
            ));
        }

        // Add to result
        // Sort layer for deterministic output
        let mut sorted_layer = layer.clone();
        sorted_layer.sort_by(|a, b| a.name.cmp(&b.name));
        layers.push(sorted_layer);

        // Remove from remaining
        for c in layer {
            remaining.remove(&c.name);
        }
    }

    Ok(layers)
}
