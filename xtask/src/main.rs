#![allow(warnings)]
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::fs;
use toml_edit::{value, DocumentMut, Value};
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
    /// Run pre-flight checks (audit, outdated, clean git)
    PreFlight,
    /// Bump version across all workspace crates
    BumpVersion {
        /// Version bump type
        #[arg(value_enum)]
        bump_type: Option<BumpType>,
    },
    /// Full release workflow: bump, build, test, commit, publish
    Release {
        /// Version bump type
        #[arg(value_enum)]
        bump_type: Option<BumpType>,
        /// Skip publish step
        #[arg(long)]
        skip_publish: bool,
        /// Dry run mode (no commits, no push, no publish)
        #[arg(long)]
        dry_run: bool,
    },
    /// Run CI checks (fmt, clippy, test, deny)
    CI,
    /// Generate bindings for Python (UniFFI) and Dart (FRB)
    #[command(name = "generate-bindings")]
    GenerateBindings,
    /// Publish Python bindings to PyPI
    #[command(name = "publish-pypi")]
    PublishPyPI,
    /// Publish WASM package to NPM
    #[command(name = "publish-npm")]
    PublishNpm,
    /// Publish WASM package to JSR.io
    #[command(name = "publish-jsr")]
    PublishJsr,
    /// Publish to ALL registries (Crates.io, PyPI, NPM, JSR)
    #[command(name = "publish-all")]
    PublishAll {
        /// Skip Crates.io publishing
        #[arg(long)]
        skip_crates: bool,
        /// Skip PyPI publishing
        #[arg(long)]
        skip_pypi: bool,
        /// Skip NPM publishing
        #[arg(long)]
        skip_npm: bool,
        /// Skip JSR publishing
        #[arg(long)]
        skip_jsr: bool,
        /// Perform a dry-run (check packaging without uploading)
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Clone, Copy, ValueEnum, Debug)]
enum BumpType {
    /// Increment patch version (0.5.0 -> 0.5.1)
    Patch,
    /// Increment minor version (0.5.0 -> 0.6.0)
    Minor,
    /// Increment major version (0.5.0 -> 1.0.0)
    Major,
}

pub mod publish;

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

        Commands::PreFlight => run_preflight(&sh)?,
        Commands::GitSync { message, push } => run_git_sync(&sh, &message, push)?,
        Commands::Publish { dry_run } => publish::run_publish_parallel(&sh, dry_run)?,
        Commands::BumpVersion { bump_type } => run_bump_version(&sh, bump_type)?,
        Commands::Release {
            bump_type,
            skip_publish,
            dry_run,
        } => run_release(&sh, bump_type, skip_publish, dry_run)?,
        Commands::CI => run_ci(&sh)?,
        Commands::GenerateBindings => run_generate_bindings(&sh)?,
        Commands::PublishPyPI => run_publish_pypi(&sh)?,
        Commands::PublishNpm => run_publish_npm(&sh)?,
        Commands::PublishJsr => run_publish_jsr(&sh)?,
        Commands::PublishAll {
            skip_crates,
            skip_pypi,
            skip_npm,
            skip_jsr,
            dry_run,
        } => run_publish_all(&sh, skip_crates, skip_pypi, skip_npm, skip_jsr, dry_run)?,
    }

    Ok(())
}

fn run_ci(sh: &Shell) -> Result<()> {
    println!("{}", "🔍 Running CI checks...".cyan().bold());

    // 1. Format Check
    println!("{}", "🎨 Checking formatting...".dimmed());
    if let Err(e) = cmd!(sh, "cargo fmt -- --check").run() {
        println!("{}", "❌ Formatting check failed".red());
        return Err(e.into());
    }
    println!("{}", "✅ Formatting OK".green());

    // 2. Clippy
    println!("{}", "📎 Running Clippy...".dimmed());
    if let Err(e) = cmd!(sh, "cargo clippy --workspace --all-targets -- -D warnings").run() {
        println!("{}", "❌ Clippy failed".red());
        return Err(e.into());
    }
    println!("{}", "✅ Clippy OK".green());

    // 3. Tests
    println!("{}", "🧪 Running tests...".dimmed());
    if let Err(e) = cmd!(sh, "cargo test --workspace").run() {
        println!("{}", "❌ Tests failed".red());
        return Err(e.into());
    }
    println!("{}", "✅ Tests OK".green());

    // 4. Deny
    if cmd!(sh, "cargo deny --version").quiet().run().is_ok() {
        println!("{}", "🚫 Checking licenses and bans...".dimmed());
        if let Err(e) = cmd!(sh, "cargo deny check").run() {
            println!("{}", "❌ cargo-deny failed".red());
            return Err(e.into());
        }
        println!("{}", "✅ Licenses & Bans OK".green());
    } else {
        println!("{}", "⚠️  cargo-deny skipped (not installed)".yellow());
    }

    println!("\n{}", "🎉 All CI checks passed!".green().bold());
    Ok(())
}

fn run_git_sync(sh: &Shell, msg: &str, push: bool) -> Result<()> {
    if msg.trim().is_empty() {
        anyhow::bail!("Commit message cannot be empty");
    }

    // Dynamic submodule parsing
    let gitmodules_content = sh.read_file(".gitmodules")?;
    let mut submodules = Vec::new();
    for line in gitmodules_content.lines() {
        if let Some(path) = line.trim().strip_prefix("path = ") {
            submodules.push(path.trim().to_string());
        }
    }

    println!("{}", "🔄 Syncing submodules...".cyan().bold());

    let mp = indicatif::MultiProgress::new();
    let style = ProgressStyle::with_template("{spinner:.green} {msg}")
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    for sub in submodules {
        let pb = mp.add(ProgressBar::new_spinner());
        pb.set_style(style.clone());
        pb.set_message(format!("Processing {}...", sub));

        if !sh.path_exists(&sub) {
            pb.finish_with_message(format!("⚠️ {} not found", sub).yellow().to_string());
            continue;
        }

        let _guard = sh.push_dir(&sub);

        // Check if there are changes
        let status = cmd!(sh, "git status --porcelain").read()?;
        if !status.is_empty() {
            pb.set_message(format!("Committing {}...", sub));
            cmd!(sh, "git add .").quiet().run()?;
            cmd!(sh, "git commit -m {msg}").quiet().run()?;

            if push {
                pb.set_message(format!("Pushing {}...", sub));
                let current_branch = cmd!(sh, "git branch --show-current").read()?;
                let remote = "origin";

                if let Err(e) = cmd!(sh, "git push {remote} {current_branch}").quiet().run() {
                    pb.finish_with_message(format!("❌ Push failed: {}", e).red().to_string());
                    continue;
                }
            }
            pb.finish_with_message(format!("✅ {} updated", sub).green().to_string());
        } else {
            pb.finish_with_message(format!("✓ {} clean", sub).dimmed().to_string());
        }
    }

    // Root update
    println!("{}", "🌳 Updating root repository...".cyan().bold());
    let status = cmd!(sh, "git status --porcelain").read()?;
    if !status.is_empty() {
        cmd!(sh, "git add .").run()?;
        cmd!(sh, "git commit -m {msg}").run()?;
        if push {
            let current_branch = cmd!(sh, "git branch --show-current").read()?;
            cmd!(sh, "git push origin {current_branch}").run()?;
        }
    } else {
        println!("{}", "Root repository clean.".dimmed());
    }

    println!("{}", "✅ Git Sync Complete".green().bold());
    Ok(())
}

fn run_preflight(sh: &Shell) -> Result<()> {
    println!("{}", "🛫 Running Pre-Flight Checks...".cyan().bold());

    // 1. Git Status
    ensure_clean_git(sh)?;
    println!("{}", "✅ Git workspace is clean".green());

    // 2. Build & Test
    println!("{}", "🏗️ Building workspace...".dimmed());
    cmd!(sh, "cargo build --workspace").run()?;
    println!("{}", "✅ Build successful".green());

    println!("{}", "🧪 Running tests...".dimmed());
    cmd!(sh, "cargo test --workspace").run()?;
    println!("{}", "✅ Tests passed".green());

    // 3. Audit (if installed)
    if cmd!(sh, "cargo audit --version").quiet().run().is_ok() {
        println!("{}", "🔒 Running security audit...".dimmed());
        cmd!(sh, "cargo audit").run()?;
        println!("{}", "✅ Security audit passed".green());
    } else {
        println!("{}", "⚠️  cargo-audit skipped (not installed)".yellow());
    }

    println!(
        "\n{}",
        "🎉 Pre-flight checks passed! Ready for takeoff."
            .green()
            .bold()
    );
    Ok(())
}

/// Bump version in workspace Cargo.toml
fn run_bump_version(sh: &Shell, bump_type: Option<BumpType>) -> Result<()> {
    ensure_clean_git(sh)?;

    let bump_type = match bump_type {
        Some(t) => t,
        None => {
            let selections = &[BumpType::Patch, BumpType::Minor, BumpType::Major];
            let selection = dialoguer::Select::new()
                .with_prompt("Select version bump type")
                .items(&["Patch", "Minor", "Major"])
                .default(0)
                .interact()?;
            selections[selection]
        }
    };

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
    let mut doc = content.parse::<DocumentMut>()?;
    if let Some(workspace) = doc.get_mut("workspace") {
        if let Some(package) = workspace.get_mut("package") {
            package["version"] = value(&new_version);
        }
    }

    // Update workspace dependencies versions for praborrow-* crates
    // This ensures that when we bump the version, all internal dependencies in the workspace
    // are also updated to point to the new version.
    if let Some(deps) = doc
        .get_mut("workspace")
        .and_then(|w| w.get_mut("dependencies"))
        .and_then(|d| d.as_table_mut())
    {
        for (key, item) in deps.iter_mut() {
            if key.starts_with("praborrow") {
                if let Some(table) = item.as_inline_table_mut() {
                    if let Some(ver) = table.get_mut("version") {
                        *ver = Value::from(new_version.as_str());
                    }
                } else if let Some(table) = item.as_table_mut() {
                    if let Some(ver) = table.get_mut("version") {
                        *ver = value(&new_version);
                    }
                }
            }
        }
    }

    fs::write(cargo_toml_path, doc.to_string())?;

    println!("{}", "✅ Version bumped successfully!".green().bold());
    println!("   Run `cargo xtask verify` to ensure everything builds.");

    Ok(())
}

/// Extract version from workspace Cargo.toml
fn extract_workspace_version(content: &str) -> Option<String> {
    let doc = content.parse::<DocumentMut>().ok()?;
    doc.get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
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

/// Full release workflow
fn run_release(
    sh: &Shell,
    bump_type: Option<BumpType>,
    skip_publish: bool,
    dry_run: bool,
) -> Result<()> {
    println!("{}", "🚀 Starting Release Workflow...".magenta().bold());
    println!("{}", "═".repeat(50).dimmed());

    if dry_run {
        println!("{}", "ℹ️  DRY RUN MODE ENABLED".yellow().bold());
        println!("   No changes will be committed or pushed.");
        println!("   No crates will be published.\n");
    }

    // Step 1: Bump version
    println!("\n{}", "Step 1/5: Bumping version...".cyan().bold());
    if dry_run {
        println!("   [Dry Run] Would bump version ({:?})", bump_type);
    } else {
        match run_bump_version(sh, bump_type) {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "{}",
                    "❌ Version bump failed. State is clean (or unchanged).".red()
                );
                return Err(e);
            }
        }
    }

    // Get new version (mock for dry run if needed, or read actual if bumped)
    let new_version = if dry_run {
        "0.0.0-dryrun".to_string()
    } else {
        let cargo_toml = fs::read_to_string("Cargo.toml")?;
        extract_workspace_version(&cargo_toml).unwrap_or_else(|| "unknown".to_string())
    };

    // Wrap subsequent steps in a closure or block to handle rollback
    let result = (|| -> Result<()> {
        // Step 2: Build
        println!("\n{}", "Step 2/5: Building workspace...".cyan().bold());
        println!("$ cargo build --workspace --exclude xtask");
        cmd!(sh, "cargo build --workspace --exclude xtask").run()?;
        println!("{}", "   ✅ Build successful".green());

        // Step 3: Test
        println!("\nStep 3/5: Running tests...");
        println!("$ cargo test --workspace --exclude xtask");
        cmd!(sh, "cargo test --workspace --exclude xtask").run()?;
        println!("{}", "   ✅ All tests passed".green());
        Ok(())
    })();

    if let Err(e) = result {
        println!("{}", "❌ Build or Test failed.".red());
        if !dry_run {
            println!("{}", "   Reverting version bump...".yellow());
            cmd!(sh, "git checkout Cargo.toml crates/praborrow/Cargo.toml").run()?;
            // Also need to revert other crates if bumped...
            // Ideally we'd valid 'git restore .' but that's risky.
            // Rely on user to check git status.
            println!(
                "{}",
                "⚠️  Please check git status and revert manual changes if needed.".yellow()
            );
        }
        return Err(e);
    }

    // Step 4: Commit and push
    println!("\n{}", "Step 4/5: Committing changes...".cyan().bold());
    let commit_msg = format!("release: v{}", new_version);

    if dry_run {
        println!("   [Dry Run] Would commit with message: {:?}", commit_msg);
        println!("   [Dry Run] Would push to origin");
    } else {
        run_git_sync(sh, &commit_msg, true)?;
    }

    // Step 5: Publish (optional)
    if skip_publish {
        println!(
            "\n{}",
            "Step 5/5: Skipping publish (--skip-publish)".yellow()
        );
    } else {
        println!("\n{}", "Step 5/5: Publishing to crates.io...".cyan().bold());
        if !dry_run {
            let confirmed = dialoguer::Confirm::new()
                .with_prompt("Ready to publish to crates.io?")
                .default(false)
                .interact()?;
            if !confirmed {
                println!("{}", "Aborted".yellow());
                return Ok(());
            }
        }
        publish::run_publish_parallel(sh, dry_run)?;
    }

    println!("\n{}", "═".repeat(50).dimmed());
    if dry_run {
        println!(
            "{}",
            format!(
                "🎉 [Dry Run] Release v{} completed successfully!",
                new_version
            )
            .magenta()
            .bold()
        );
    } else {
        println!(
            "{}",
            format!("🎉 Release v{} complete!", new_version)
                .magenta()
                .bold()
        );
    }

    Ok(())
}

fn ensure_clean_git(sh: &Shell) -> Result<()> {
    let status = cmd!(sh, "git status --porcelain").read()?;
    if !status.is_empty() {
        anyhow::bail!("Git workspace is dirty. Please commit or stash changes before releasing.");
    }
    Ok(())
}

fn run_generate_bindings(sh: &Shell) -> Result<()> {
    println!("{}", "🔗 Generating bindings...".cyan().bold());

    // 1. Python (UniFFI)
    println!("{}", "\n🐍 Generating Python bindings...".dimmed());
    if cmd!(sh, "uniffi-bindgen --version").quiet().run().is_ok() {
        // Ensure the library is built (needed for proc-macro based generation)
        println!("  Compiling praborrow-bindings...");
        cmd!(sh, "cargo build -p praborrow-bindings").quiet().run()?;

        let out_dir = "bindings/python";
        fs::create_dir_all(out_dir)?;
        
        // Determine library path (heuristics for Windows/Unix)
        let lib_name = "praborrow_bindings";
        let lib_path = if cfg!(windows) {
            format!("target/debug/{}.dll", lib_name)
        } else if cfg!(target_os = "macos") {
            format!("target/debug/lib{}.dylib", lib_name)
        } else {
            format!("target/debug/lib{}.so", lib_name)
        };

        if !sh.path_exists(&lib_path) {
             anyhow::bail!("Library not found at {}. Build failed?", lib_path);
        }

        if let Err(e) = cmd!(sh, "uniffi-bindgen generate {lib_path} --language python --out-dir {out_dir}").run() {
             println!("{}", "❌ Python binding generation failed".red());
             return Err(e.into());
        }
        println!("{}", "✅ Python bindings generated in bindings/python".green());
    } else {
        println!("{}", "⚠️  uniffi-bindgen not found. Install with: cargo install uniffi-bindgen".yellow());
    }

    // 2. Dart (Flutter Rust Bridge)
    println!("{}", "\n🎯 Generating Dart bindings...".dimmed());
    if cmd!(sh, "flutter_rust_bridge_codegen --version").quiet().run().is_ok() {
        let _guard = sh.push_dir("crates/praborrow-dart");
        if let Err(e) = cmd!(sh, "flutter_rust_bridge_codegen generate").run() {
            println!("{}", "❌ Dart binding generation failed".red());
            return Err(e.into());
        }
        println!("{}", "✅ Dart bindings generated".green());
    } else {
        println!("{}", "⚠️  flutter_rust_bridge_codegen not found. Install with: cargo install flutter_rust_bridge_codegen".yellow());
    }

    println!("\n{}", "🎉 Binding generation task complete.".green().bold());
    Ok(())
}

fn run_publish_pypi(sh: &Shell) -> Result<()> {
    println!("{}", "🐍 Publishing to PyPI...".cyan().bold());

    let bindings_dir = "crates/praborrow-bindings";
    if !sh.path_exists(bindings_dir) {
        anyhow::bail!("Directory {} not found", bindings_dir);
    }

    let _guard = sh.push_dir(bindings_dir);

    // Determine maturin command (try direct, then via python module)
    let maturin_cmd = if cmd!(sh, "maturin --version").quiet().run().is_ok() {
        vec!["maturin"]
    } else if cmd!(sh, "python -m maturin --version").quiet().run().is_ok() {
        println!("{}", "⚠️  'maturin' not in PATH, using 'python -m maturin'".yellow());
        vec!["python", "-m", "maturin"]
    } else {
        println!("{}", "❌ maturin not found. Please install: pip install maturin".red());
        anyhow::bail!("maturin not found");
    };

    // Construct command
    let mut args = maturin_cmd;
    args.push("publish");

    // Run publish
    let cmd_str = args.join(" ");
    println!("  → Running: {}", cmd_str);
    
    // xshell cmd! macro requires literal string slices or specific construction for dynamic args.
    // For dynamic command name/args, strictly using std::process::Command or xshell's strict API is needed.
    // With xshell, we can't easily spread a Vec into `cmd!`.
    // Simpler here: Just branching the xshell call or using std Command if complex.
    // Or just use the branch directly.

    if args[0] == "maturin" {
        if let Err(e) = cmd!(sh, "maturin publish").run() {
             println!("{}", "❌ PyPI publish failed".red());
             return Err(e.into());
        }
    } else {
         if let Err(e) = cmd!(sh, "python -m maturin publish").run() {
             println!("{}", "❌ PyPI publish failed".red());
             return Err(e.into());
         }
    }

    println!("{}", "✅ Successfully published to PyPI".green().bold());
    Ok(())
}

fn run_publish_npm(sh: &Shell) -> Result<()> {
    println!("{}", "📦 Publishing to NPM...".cyan().bold());

    let wasm_dir = "crates/praborrow-wasm";
    if !sh.path_exists(wasm_dir) {
        anyhow::bail!("Directory {} not found", wasm_dir);
    }

    // Build WASM first
    println!("  Building WASM package...");
    let _guard = sh.push_dir(wasm_dir);
    if let Err(e) = cmd!(sh, "wasm-pack build --target web --out-dir pkg").run() {
        println!("{}", "❌ WASM build failed".red());
        return Err(e.into());
    }

    // Update package name in pkg/package.json
    let pkg_json_path = "pkg/package.json";
    if sh.path_exists(pkg_json_path) {
        let content = fs::read_to_string(pkg_json_path)?;
        let updated = content.replace("\"praborrow-wasm\"", "\"@ireddragonicy/praborrow\"");
        fs::write(pkg_json_path, updated)?;
    }

    // Publish to NPM
    let _pkg_guard = sh.push_dir("pkg");
    if let Err(e) = cmd!(sh, "npm publish --access public").run() {
        println!("{}", "❌ NPM publish failed".red());
        return Err(e.into());
    }

    println!("{}", "✅ Successfully published to NPM".green().bold());
    Ok(())
}

fn run_publish_jsr(sh: &Shell) -> Result<()> {
    println!("{}", "🦕 Publishing to JSR.io...".cyan().bold());

    let wasm_dir = "crates/praborrow-wasm";
    if !sh.path_exists(wasm_dir) {
        anyhow::bail!("Directory {} not found", wasm_dir);
    }

    // Build WASM first (if not already built)
    let _guard = sh.push_dir(wasm_dir);
    let pkg_dir = "pkg";
    if !sh.path_exists(pkg_dir) {
        println!("  Building WASM package...");
        if let Err(e) = cmd!(sh, "wasm-pack build --target web --out-dir pkg").run() {
            println!("{}", "❌ WASM build failed".red());
            return Err(e.into());
        }
    }

    // Create mod.ts if it doesn't exist
    let mod_ts_path = "pkg/mod.ts";
    if !sh.path_exists(mod_ts_path) {
        fs::write(mod_ts_path, "// JSR entry point\nexport * from \"./praborrow_wasm.js\";\n")?;
    }

    // Copy jsr.json to pkg (if exists in root)
    if sh.path_exists("jsr.json") {
        fs::copy("jsr.json", "pkg/jsr.json")?;
    }

    // Publish to JSR
    let _pkg_guard = sh.push_dir("pkg");
    if let Err(e) = cmd!(sh, "npx jsr publish --allow-dirty").run() {
        println!("{}", "❌ JSR publish failed".red());
        return Err(e.into());
    }

    println!("{}", "✅ Successfully published to JSR.io".green().bold());
    Ok(())
}

fn run_publish_all(
    sh: &Shell,
    skip_crates: bool,
    skip_pypi: bool,
    skip_npm: bool,
    skip_jsr: bool,
    dry_run: bool,
) -> Result<()> {
    println!("{}", "🚀 Publishing to ALL registries...".cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if dry_run {
        println!("{}", "🔍 DRY RUN MODE - No actual publishing will occur\n".yellow());
    }

    let mut success_count = 0;
    let mut fail_count = 0;

    // 1. Crates.io
    if !skip_crates {
        println!("\n{}", "🦀 [1/4] Crates.io...".cyan());
        match publish::run_publish_parallel(sh, dry_run) {
            Ok(_) => {
                println!("{}", "  ✅ Crates.io publish successful!".green());
                success_count += 1;
            }
            Err(e) => {
                println!("{}", format!("  ❌ Crates.io failed: {}", e).red());
                fail_count += 1;
            }
        }
    } else {
        println!("\n{}", "🦀 [1/4] Crates.io... SKIPPED".dimmed());
    }

    // 2. PyPI
    if !skip_pypi && !dry_run {
        println!("\n{}", "🐍 [2/4] PyPI...".cyan());
        match run_publish_pypi(sh) {
            Ok(_) => {
                println!("{}", "  ✅ PyPI publish successful!".green());
                success_count += 1;
            }
            Err(e) => {
                println!("{}", format!("  ❌ PyPI failed: {}", e).red());
                fail_count += 1;
            }
        }
    } else if skip_pypi {
        println!("\n{}", "🐍 [2/4] PyPI... SKIPPED".dimmed());
    } else {
        println!("\n{}", "🐍 [2/4] PyPI... SKIPPED (dry-run not supported)".dimmed());
    }

    // 3. NPM
    if !skip_npm && !dry_run {
        println!("\n{}", "📦 [3/4] NPM...".cyan());
        match run_publish_npm(sh) {
            Ok(_) => {
                println!("{}", "  ✅ NPM publish successful!".green());
                success_count += 1;
            }
            Err(e) => {
                println!("{}", format!("  ❌ NPM failed: {}", e).red());
                fail_count += 1;
            }
        }
    } else if skip_npm {
        println!("\n{}", "📦 [3/4] NPM... SKIPPED".dimmed());
    } else {
        println!("\n{}", "📦 [3/4] NPM... SKIPPED (dry-run not supported)".dimmed());
    }

    // 4. JSR
    if !skip_jsr && !dry_run {
        println!("\n{}", "🦕 [4/4] JSR.io...".cyan());
        match run_publish_jsr(sh) {
            Ok(_) => {
                println!("{}", "  ✅ JSR publish successful!".green());
                success_count += 1;
            }
            Err(e) => {
                println!("{}", format!("  ❌ JSR failed: {}", e).red());
                fail_count += 1;
            }
        }
    } else if skip_jsr {
        println!("\n{}", "🦕 [4/4] JSR.io... SKIPPED".dimmed());
    } else {
        println!("\n{}", "🦕 [4/4] JSR.io... SKIPPED (dry-run not supported)".dimmed());
    }

    // Summary
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    if fail_count == 0 {
        println!("{}", format!("✅✅✅ ALL {} PUBLISH OPERATIONS COMPLETE! ✅✅✅", success_count).green().bold());
    } else {
        println!("{}", format!("⚠️  Publish completed with {} success, {} failures", success_count, fail_count).yellow());
    }

    Ok(())
}
