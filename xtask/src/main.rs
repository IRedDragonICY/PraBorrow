use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::fs;
use xshell::{cmd, Shell};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "PraBorrow Development Automation Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build all crates in the workspace
    Build,
    /// Run test suite for all crates
    Test,
    /// Sync submodules: commits changes in submodules, pushes them, and updates root
    GitSync {
        /// Commit message to use for all submodules
        #[arg(short, long, default_value = "chore: sync submodule updates")]
        message: String,
        /// Push changes to remote
        #[arg(short, long, default_value_t = true)]
        push: bool,
    },
    /// Publish all crates to crates.io in the correct order
    Publish {
        /// Perform a dry-run (check packaging without uploading)
        #[arg(long)]
        dry_run: bool,
    },
    /// Run all verification steps (build + test)
    Verify,
    /// Bump version across all workspace crates
    BumpVersion {
        /// Version bump type
        #[arg(value_enum)]
        bump_type: BumpType,
    },
    /// Full release workflow: bump, build, test, commit, publish
    Release {
        /// Version bump type
        #[arg(value_enum)]
        bump_type: BumpType,
        /// Skip publish step
        #[arg(long)]
        skip_publish: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum BumpType {
    /// Increment patch version (0.5.0 -> 0.5.1)
    Patch,
    /// Increment minor version (0.5.0 -> 0.6.0)
    Minor,
    /// Increment major version (0.5.0 -> 1.0.0)
    Major,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    match cli.command {
        Commands::Build => {
            println!("{}", "🚀 Building workspace...".green().bold());
            cmd!(sh, "cargo build --workspace").run()?;
            println!("{}", "✅ Build successful".green().bold());
        }
        Commands::Test => {
            println!("{}", "🧪 Running tests...".green().bold());
            cmd!(sh, "cargo test --workspace").run()?;
            println!("{}", "✅ All tests passed".green().bold());
        }
        Commands::Verify => {
            println!("{}", "🛡️ Verifying project integrity...".green().bold());
            cmd!(sh, "cargo build --workspace").run()?;
            cmd!(sh, "cargo test --workspace").run()?;
            println!("{}", "✅ Verification complete".green().bold());
        }
        Commands::GitSync { message, push } => run_git_sync(&sh, &message, push)?,
        Commands::Publish { dry_run } => run_publish(&sh, dry_run)?,
        Commands::BumpVersion { bump_type } => run_bump_version(&sh, bump_type)?,
        Commands::Release {
            bump_type,
            skip_publish,
        } => run_release(&sh, bump_type, skip_publish)?,
    }

    Ok(())
}

fn run_git_sync(sh: &Shell, msg: &str, push: bool) -> Result<()> {
    let submodules = vec![
        "crates/praborrow-core",
        "crates/praborrow-defense",
        "crates/praborrow-lease",
        "crates/praborrow-logistics",
        "crates/praborrow-diplomacy",
        "crates/praborrow-sidl",
        "crates/praborrow-macros",
        "crates/praborrow-prover",
    ];

    println!("{}", "🔄 Syncing submodules...".cyan().bold());

    for sub in submodules {
        if !sh.path_exists(sub) {
            println!(
                "{}",
                format!("⚠️ Submodule {} not found, skipping", sub).yellow()
            );
            continue;
        }

        let _guard = sh.push_dir(sub);
        println!("{}", format!("📂 Processing {}...", sub).blue());

        // Check if there are changes
        let status = cmd!(sh, "git status --porcelain").read()?;
        if !status.is_empty() {
            println!("   Changes detected. Committing...");
            cmd!(sh, "git add .").run()?;
            cmd!(sh, "git commit -m {msg}").run()?;

            if push {
                println!("   Pushing to origin/main...");
                // Try catch push error? xshell throws error on non-zero exit.
                if let Err(e) = cmd!(sh, "git push origin main").run() {
                    println!("{}", format!("   ⚠️ Push failed for {}: {}", sub, e).red());
                }
            }
        } else {
            println!("   Clean.");
        }
    }

    // Root update
    println!("{}", "🌳 Updating root repository...".cyan().bold());
    let status = cmd!(sh, "git status --porcelain").read()?;
    if !status.is_empty() {
        cmd!(sh, "git add .").run()?;
        cmd!(sh, "git commit -m {msg}").run()?;
        if push {
            cmd!(sh, "git push origin main").run()?;
        }
    } else {
        println!("{}", "Root repository clean.".dimmed());
    }

    println!("{}", "✅ Git Sync Complete".green().bold());
    Ok(())
}

fn run_publish(sh: &Shell, dry_run: bool) -> Result<()> {
    // Topological order for publishing
    let order = vec![
        "crates/praborrow-core",      // No deps
        "crates/praborrow-macros",    // Syn/quote/proc-macro
        "crates/praborrow-defense",   // Proc-macro, uses core types in generated code
        "crates/praborrow-logistics", // Likely core dep
        "crates/praborrow-sidl",      // Macro
        "crates/praborrow-diplomacy", // Likely core dep
        "crates/praborrow-prover",    // Depends on core
        "crates/praborrow-lease",     // Depends on core
        "crates/praborrow",           // Facade - depends on all
    ];

    println!("{}", "📦 Starting Publish Workflow...".magenta().bold());
    if dry_run {
        println!("{}", "ℹ️  DRY RUN MODE".yellow());
    }

    let mut published = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for crate_path in &order {
        let _guard = sh.push_dir(crate_path);
        let crate_name = crate_path.split('/').next_back().unwrap();

        // Read version from Cargo.toml
        let cargo_toml = sh.read_file("Cargo.toml")?;
        let local_version = extract_version(&cargo_toml);

        println!(
            "{}",
            format!("\n🔍 Checking {}@{}...", crate_name, local_version).cyan()
        );

        // Check if this version already exists on crates.io
        if !dry_run {
            let search_result = cmd!(sh, "cargo search {crate_name} --limit 1")
                .read()
                .unwrap_or_default();
            if search_result.contains(&format!("{}@{}", crate_name, local_version))
                || search_result.contains(&format!("{} = \"{}\"", crate_name, local_version))
            {
                println!(
                    "{}",
                    format!(
                        "⏭️  {} v{} already exists, skipping",
                        crate_name, local_version
                    )
                    .yellow()
                );
                skipped += 1;
                continue;
            }
        }

        println!("{}", format!("🚀 Publishing {}...", crate_name).cyan());

        let result = if dry_run {
            cmd!(sh, "cargo publish --dry-run --allow-dirty").run()
        } else {
            // Always use --allow-dirty to avoid workspace inheritance issues
            cmd!(sh, "cargo publish --allow-dirty").run()
        };

        match result {
            Ok(_) => {
                println!(
                    "{}",
                    format!("✅ Published {} v{}", crate_name, local_version).green()
                );
                published += 1;

                if !dry_run {
                    // Smart propagation wait with verification
                    wait_for_index_propagation(sh, crate_name, &local_version)?;
                }
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("already exists") {
                    println!(
                        "{}",
                        format!("⏭️  {} already published, skipping", crate_name).yellow()
                    );
                    skipped += 1;
                } else {
                    println!(
                        "{}",
                        format!("❌ Failed to publish {}: {}", crate_name, e).red()
                    );
                    failed += 1;

                    // Ask user if they want to continue
                    println!("{}", "⚠️  Continuing with next crate...".yellow());
                }
            }
        }
    }

    // Summary
    println!("\n{}", "═".repeat(50).dimmed());
    println!("{}", "📊 Publish Summary".magenta().bold());
    println!("   ✅ Published: {}", published.to_string().green());
    println!("   ⏭️  Skipped:   {}", skipped.to_string().yellow());
    println!("   ❌ Failed:    {}", failed.to_string().red());
    println!("{}", "═".repeat(50).dimmed());

    if failed > 0 {
        println!(
            "{}",
            "\n⚠️  Some crates failed to publish. Check errors above.".red()
        );
    } else {
        println!(
            "{}",
            "\n🎉 Publish workflow finished successfully!"
                .magenta()
                .bold()
        );
    }

    Ok(())
}

/// Extract version from Cargo.toml content
fn extract_version(cargo_toml: &str) -> String {
    for line in cargo_toml.lines() {
        let line = line.trim();
        if line.starts_with("version") && line.contains("=") && !line.contains("workspace") {
            // version = "x.y.z"
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if start < end {
                        return line[start + 1..end].to_string();
                    }
                }
            }
        }
    }
    // If using workspace inheritance, read from root
    "0.5.0".to_string() // Fallback to current version
}

/// Wait for crates.io index to propagate the new version
fn wait_for_index_propagation(sh: &Shell, crate_name: &str, version: &str) -> Result<()> {
    let pb = ProgressBar::new(30);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    pb.set_message(format!(
        "Waiting for {} v{} to propagate...",
        crate_name, version
    ));

    // Try up to 30 seconds, checking every 2 seconds
    for i in 0..15 {
        pb.inc(2);
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Check if version is now available
        let search = cmd!(sh, "cargo search {crate_name} --limit 1")
            .read()
            .unwrap_or_default();
        if search.contains(version) {
            pb.finish_with_message(format!(
                "✓ {} v{} available on crates.io",
                crate_name, version
            ));
            return Ok(());
        }

        // Update progress message
        if i == 7 {
            pb.set_message(format!(
                "Still waiting for {} (index may be slow)...",
                crate_name
            ));
        }
    }

    pb.finish_with_message(format!(
        "Timeout - {} may take longer to appear",
        crate_name
    ));
    Ok(())
}

/// Bump version in workspace Cargo.toml
fn run_bump_version(sh: &Shell, bump_type: BumpType) -> Result<()> {
    println!("{}", "📦 Bumping workspace version...".magenta().bold());

    // Read current version from workspace Cargo.toml
    let cargo_toml_path = "Cargo.toml";
    let content = fs::read_to_string(cargo_toml_path)?;

    let current_version = extract_workspace_version(&content)
        .ok_or_else(|| anyhow::anyhow!("Could not find version in workspace Cargo.toml"))?;

    println!("   Current version: {}", current_version.cyan());

    // Parse and bump version
    let new_version = bump_semver(&current_version, bump_type)?;

    println!("   New version:     {}", new_version.green().bold());

    // Update workspace Cargo.toml
    let new_content = content.replace(
        &format!("version = \"{}\"", current_version),
        &format!("version = \"{}\"", new_version),
    );
    fs::write(cargo_toml_path, new_content)?;

    // Update dependency versions in facade crate
    update_facade_dependencies(sh, &current_version, &new_version)?;

    println!("{}", "✅ Version bumped successfully!".green().bold());
    println!("   Run `cargo xtask verify` to ensure everything builds.");

    Ok(())
}

/// Extract version from workspace Cargo.toml
fn extract_workspace_version(content: &str) -> Option<String> {
    let mut in_workspace_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace.package]" {
            in_workspace_package = true;
            continue;
        }
        if trimmed.starts_with('[') && in_workspace_package {
            break;
        }
        if in_workspace_package && trimmed.starts_with("version") {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed.rfind('"') {
                    if start < end {
                        return Some(trimmed[start + 1..end].to_string());
                    }
                }
            }
        }
    }
    None
}

/// Bump semver version
fn bump_semver(version: &str, bump_type: BumpType) -> Result<String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid semver: {}", version);
    }

    let major: u32 = parts[0].parse()?;
    let minor: u32 = parts[1].parse()?;
    let patch: u32 = parts[2].parse()?;

    let (new_major, new_minor, new_patch) = match bump_type {
        BumpType::Major => (major + 1, 0, 0),
        BumpType::Minor => (major, minor + 1, 0),
        BumpType::Patch => (major, minor, patch + 1),
    };

    Ok(format!("{}.{}.{}", new_major, new_minor, new_patch))
}

/// Update dependency versions in facade crate
fn update_facade_dependencies(_sh: &Shell, old_version: &str, new_version: &str) -> Result<()> {
    let facade_cargo = "crates/praborrow/Cargo.toml";
    let content = fs::read_to_string(facade_cargo)?;
    let new_content = content.replace(
        &format!("version = \"{}\"", old_version),
        &format!("version = \"{}\"", new_version),
    );
    fs::write(facade_cargo, new_content)?;
    Ok(())
}

/// Full release workflow
fn run_release(sh: &Shell, bump_type: BumpType, skip_publish: bool) -> Result<()> {
    println!("{}", "🚀 Starting Release Workflow...".magenta().bold());
    println!("{}", "═".repeat(50).dimmed());

    // Step 1: Bump version
    println!("\n{}", "Step 1/5: Bumping version...".cyan().bold());
    run_bump_version(sh, bump_type)?;

    // Step 2: Build
    println!("\n{}", "Step 2/5: Building workspace...".cyan().bold());
    cmd!(sh, "cargo build --workspace").run()?;
    println!("{}", "   ✅ Build successful".green());

    // Step 3: Test
    println!("\n{}", "Step 3/5: Running tests...".cyan().bold());
    cmd!(sh, "cargo test --workspace").run()?;
    println!("{}", "   ✅ All tests passed".green());

    // Get new version for commit message
    let cargo_toml = fs::read_to_string("Cargo.toml")?;
    let new_version =
        extract_workspace_version(&cargo_toml).unwrap_or_else(|| "unknown".to_string());

    // Step 4: Commit and push
    println!("\n{}", "Step 4/5: Committing changes...".cyan().bold());
    let commit_msg = format!("release: v{}", new_version);
    run_git_sync(sh, &commit_msg, true)?;

    // Step 5: Publish (optional)
    if skip_publish {
        println!(
            "\n{}",
            "Step 5/5: Skipping publish (--skip-publish)".yellow()
        );
    } else {
        println!("\n{}", "Step 5/5: Publishing to crates.io...".cyan().bold());
        run_publish(sh, false)?;
    }

    println!("\n{}", "═".repeat(50).dimmed());
    println!(
        "{}",
        format!("🎉 Release v{} complete!", new_version)
            .magenta()
            .bold()
    );

    Ok(())
}
