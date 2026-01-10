use clap::{Parser, Subcommand};
use xshell::{cmd, Shell};
use anyhow::Result;
use owo_colors::OwoColorize;
use indicatif::{ProgressBar, ProgressStyle};

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
            println!("{}", format!("⚠️ Submodule {} not found, skipping", sub).yellow());
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
        "crates/praborrow-core",     // No deps
        "crates/praborrow-macros",   // Syn/quote/proc-macro
        "crates/praborrow-defense",  // Proc-macro, uses core types in generated code
        "crates/praborrow-logistics",// Likely core dep
        "crates/praborrow-sidl",     // Macro
        "crates/praborrow-diplomacy",// Likely core dep
        "crates/praborrow-prover",   // Depends on core
        "crates/praborrow-lease",    // Depends on core
        "crates/praborrow",          // Facade - depends on all
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
        let crate_name = crate_path.split('/').last().unwrap();
        
        // Read version from Cargo.toml
        let cargo_toml = sh.read_file("Cargo.toml")?;
        let local_version = extract_version(&cargo_toml);
        
        println!("{}", format!("\n🔍 Checking {}@{}...", crate_name, local_version).cyan());

        // Check if this version already exists on crates.io
        if !dry_run {
            let search_result = cmd!(sh, "cargo search {crate_name} --limit 1").read().unwrap_or_default();
            if search_result.contains(&format!("{}@{}", crate_name, local_version)) || 
               search_result.contains(&format!("{} = \"{}\"", crate_name, local_version)) {
                println!("{}", format!("⏭️  {} v{} already exists, skipping", crate_name, local_version).yellow());
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
                println!("{}", format!("✅ Published {} v{}", crate_name, local_version).green());
                published += 1;
                
                if !dry_run {
                    // Smart propagation wait with verification
                    wait_for_index_propagation(sh, crate_name, &local_version)?;
                }
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("already exists") {
                    println!("{}", format!("⏭️  {} already published, skipping", crate_name).yellow());
                    skipped += 1;
                } else {
                    println!("{}", format!("❌ Failed to publish {}: {}", crate_name, e).red());
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
        println!("{}", "\n⚠️  Some crates failed to publish. Check errors above.".red());
    } else {
        println!("{}", "\n🎉 Publish workflow finished successfully!".magenta().bold());
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
                        return line[start+1..end].to_string();
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
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {msg}")
        .unwrap()
        .progress_chars("#>-"));
    pb.set_message(format!("Waiting for {} v{} to propagate...", crate_name, version));
    
    // Try up to 30 seconds, checking every 2 seconds
    for i in 0..15 {
        pb.inc(2);
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        // Check if version is now available
        let search = cmd!(sh, "cargo search {crate_name} --limit 1").read().unwrap_or_default();
        if search.contains(version) {
            pb.finish_with_message(format!("✓ {} v{} available on crates.io", crate_name, version));
            return Ok(());
        }
        
        // Update progress message
        if i == 7 {
            pb.set_message(format!("Still waiting for {} (index may be slow)...", crate_name));
        }
    }
    
    pb.finish_with_message(format!("Timeout - {} may take longer to appear", crate_name));
    Ok(())
}
