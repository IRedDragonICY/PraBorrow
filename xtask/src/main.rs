use clap::{Parser, Subcommand};
use xshell::{cmd, Shell};
use anyhow::Result;
use owo_colors::OwoColorize;

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
        "crates/praborrow-defense",  // Depends on core? No, macro uses core types in generated code but crate itself is proc-macro.
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

    for crate_path in order {
        let _guard = sh.push_dir(crate_path);
        let crate_name = crate_path.split('/').last().unwrap();
        
        println!("{}", format!("🚀 Publishing {}...", crate_name).cyan());

        let mut cmd = cmd!(sh, "cargo publish");
        if dry_run {
            cmd = cmd.arg("--dry-run");
            cmd = cmd.arg("--allow-dirty"); // Dry run often needs this locally if git state isn't perfect
        } else {
            // Give system time to propagate index changes between publishes
            // We can't easily wait for index propagation in xtask without querying crates.io API.
            // But we can sleep a bit if we want safety, though manually verifying is safer.
            // For now, we assume standard publish.
        }

        if let Err(e) = cmd.run() {
             println!("{}", format!("❌ Failed to publish {}: {}", crate_name, e).red());
             println!("⚠️ Continuing with next crate... (check manually)");
        } else {
             println!("{}", format!("✅ Published {}", crate_name).green());
             if !dry_run {
                 println!("   Waiting 15s for index propagation...");
                 std::thread::sleep(std::time::Duration::from_secs(15));
             }
        }
    }

    println!("{}", "🎉 Publish workflow finished!".magenta().bold());
    Ok(())
}
