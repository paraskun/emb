//! Build orchestration for multi-part firmware projects.
//!
//! Every firmware crate declares which project it belongs to and how to build
//! it in `[package.metadata.xtask]`. Crates without that table are treated as
//! host-testable libraries. Run `cargo xtask --help` for the commands.

use std::{
    collections::BTreeSet,
    fs,
    io::BufReader,
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};
use cargo_metadata::{camino::Utf8PathBuf, Message, MetadataCommand, PackageId};
use clap::{Args, Parser, Subcommand};
use serde::Deserialize;

#[derive(Parser)]
#[command(about = "Build, package and flash multi-part firmware projects")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List projects and their parts
    List,
    /// Build the selected parts
    Build(Select),
    /// Type-check the selected parts
    Check(Select),
    /// Run clippy on the selected parts (warnings are errors)
    Clippy(Select),
    /// Build images into target/dist/<project>/ as .elf, .bin and .hex
    Dist(Select),
    /// Build and flash one project's parts, in ascending `order`
    Flash(Select),
    /// Run host tests for every crate that is not a firmware part
    Test,
}

#[derive(Args)]
struct Select {
    /// Project to operate on (all projects if omitted)
    project: Option<String>,
    /// Restrict to these parts (repeatable)
    #[arg(long, short)]
    part: Vec<String>,
    /// Use the dev profile instead of release
    #[arg(long)]
    debug: bool,
}

/// Contents of `[package.metadata.xtask]` in a firmware crate.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct PartMeta {
    /// Project this part belongs to.
    project: String,
    /// Short part name; defaults to the package name.
    part: Option<String>,
    /// Target triple to build for.
    target: String,
    /// Flash order within the project (ascending).
    #[serde(default)]
    order: i32,
    /// Binary to build when the package has several.
    bin: Option<String>,
    /// Cargo features to enable.
    #[serde(default)]
    features: Vec<String>,
    /// probe-rs chip name, required for `flash`.
    chip: Option<String>,
    /// Extra arguments for `probe-rs download` (e.g. core selection).
    #[serde(default)]
    flash_args: Vec<String>,
}

struct Part {
    id: PackageId,
    package: String,
    name: String,
    meta: PartMeta,
}

impl Part {
    fn label(&self) -> String {
        format!("{}/{}", self.meta.project, self.name)
    }
}

struct Workspace {
    root: Utf8PathBuf,
    target_dir: Utf8PathBuf,
    parts: Vec<Part>,
    host_packages: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let ws = Workspace::load()?;

    match cli.cmd {
        Cmd::List => ws.list(),
        Cmd::Build(sel) => {
            for part in ws.select(&sel)? {
                let elf = ws.build(part, !sel.debug)?;
                eprintln!("    -> {elf}");
            }
        }
        Cmd::Check(sel) => {
            for part in ws.select(&sel)? {
                ws.lint("check", part, !sel.debug)?;
            }
        }
        Cmd::Clippy(sel) => {
            for part in ws.select(&sel)? {
                ws.lint("clippy", part, !sel.debug)?;
            }
        }
        Cmd::Dist(sel) => {
            for part in ws.select(&sel)? {
                ws.dist(part, !sel.debug)?;
            }
        }
        Cmd::Flash(sel) => ws.flash(&sel)?,
        Cmd::Test => ws.test()?,
    }
    Ok(())
}

impl Workspace {
    fn load() -> Result<Self> {
        let md = MetadataCommand::new()
            .no_deps()
            .exec()
            .context("running `cargo metadata`")?;

        let mut parts = Vec::new();
        let mut host_packages = Vec::new();

        for pkg in md.workspace_packages() {
            match pkg.metadata.get("xtask") {
                Some(value) => {
                    let meta: PartMeta = serde_json::from_value(value.clone()).with_context(|| {
                        format!("invalid [package.metadata.xtask] in {}", pkg.manifest_path)
                    })?;
                    parts.push(Part {
                        id: pkg.id.clone(),
                        package: pkg.name.clone(),
                        name: meta.part.clone().unwrap_or_else(|| pkg.name.clone()),
                        meta,
                    });
                }
                None if pkg.name != "xtask" => host_packages.push(pkg.name.clone()),
                None => {}
            }
        }

        parts.sort_by(|a, b| {
            (&a.meta.project, a.meta.order, &a.name).cmp(&(&b.meta.project, b.meta.order, &b.name))
        });

        Ok(Self {
            root: md.workspace_root,
            target_dir: md.target_directory,
            parts,
            host_packages,
        })
    }

    fn projects(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.parts.iter().map(|p| p.meta.project.as_str()).collect();
        names.dedup(); // parts are sorted by project
        names
    }

    fn select(&self, sel: &Select) -> Result<Vec<&Part>> {
        if let Some(project) = &sel.project {
            if !self.parts.iter().any(|p| &p.meta.project == project) {
                bail!("unknown project `{project}`; known: {}", self.projects().join(", "));
            }
        }

        let chosen: Vec<&Part> = self
            .parts
            .iter()
            .filter(|p| sel.project.as_ref().map_or(true, |name| &p.meta.project == name))
            .filter(|p| sel.part.is_empty() || sel.part.contains(&p.name))
            .collect();

        for wanted in &sel.part {
            if !chosen.iter().any(|p| &p.name == wanted) {
                bail!("no part named `{wanted}` in the selected project(s)");
            }
        }
        if chosen.is_empty() {
            bail!("no firmware parts found; add [package.metadata.xtask] to a crate");
        }
        Ok(chosen)
    }

    fn list(&self) {
        for project in self.projects() {
            println!("{project}");
            for p in self.parts.iter().filter(|p| p.meta.project == project) {
                println!(
                    "  {:<12} {:<28} order={:<3} package={}",
                    p.name, p.meta.target, p.meta.order, p.package
                );
            }
        }
        if !self.host_packages.is_empty() {
            println!("host crates: {}", self.host_packages.join(", "));
        }
    }

    /// One cargo invocation per part. Building parts together would unify
    /// their features, which breaks e.g. two HAL crates with different
    /// chip features in one invocation.
    fn cargo_for(&self, subcommand: &str, part: &Part, release: bool) -> Command {
        let mut cmd = cargo();
        cmd.current_dir(&self.root)
            .arg(subcommand)
            .args(["--package", &part.package])
            .args(["--target", &part.meta.target]);
        if release {
            cmd.arg("--release");
        }
        if !part.meta.features.is_empty() {
            cmd.args(["--features", &part.meta.features.join(",")]);
        }
        if let Some(bin) = &part.meta.bin {
            cmd.args(["--bin", bin]);
        }
        cmd
    }

    /// Builds a part and returns the path of its ELF.
    fn build(&self, part: &Part, release: bool) -> Result<Utf8PathBuf> {
        eprintln!("==> build {} ({})", part.label(), part.meta.target);

        let mut child = self
            .cargo_for("build", part, release)
            .arg("--message-format=json-render-diagnostics")
            .stdout(Stdio::piped())
            .spawn()
            .context("spawning cargo")?;

        let stdout = child.stdout.take().expect("stdout is piped");
        let mut elfs = Vec::new();
        for message in Message::parse_stream(BufReader::new(stdout)) {
            if let Message::CompilerArtifact(artifact) = message? {
                if artifact.package_id == part.id && artifact.target.is_bin() {
                    elfs.extend(artifact.executable);
                }
            }
        }

        if !child.wait()?.success() {
            bail!("build of {} failed", part.label());
        }
        match elfs.len() {
            1 => Ok(elfs.remove(0)),
            0 => bail!("{} produced no binary", part.label()),
            _ => bail!("{} has several binaries; set `bin` in its metadata", part.label()),
        }
    }

    fn lint(&self, subcommand: &str, part: &Part, release: bool) -> Result<()> {
        eprintln!("==> {subcommand} {} ({})", part.label(), part.meta.target);
        let mut cmd = self.cargo_for(subcommand, part, release);
        if subcommand == "clippy" {
            cmd.args(["--", "-D", "warnings"]);
        }
        run(cmd)
    }

    fn dist(&self, part: &Part, release: bool) -> Result<()> {
        let elf = self.build(part, release)?;
        let dir = self.target_dir.join("dist").join(&part.meta.project);
        fs::create_dir_all(&dir)?;

        let base = dir.join(&part.name);
        fs::copy(&elf, base.with_extension("elf"))?;
        for (format, ext) in [("binary", "bin"), ("ihex", "hex")] {
            let mut cmd = Command::new("rust-objcopy");
            cmd.args(["-O", format]).arg(&elf).arg(base.with_extension(ext));
            run(cmd).context(
                "rust-objcopy failed; install it with `cargo install cargo-binutils` \
                 (needs the llvm-tools component)",
            )?;
        }
        eprintln!("    -> {base}.{{elf,bin,hex}}");
        Ok(())
    }

    fn flash(&self, sel: &Select) -> Result<()> {
        let parts = self.select(sel)?;
        let projects: BTreeSet<&str> = parts.iter().map(|p| p.meta.project.as_str()).collect();
        if projects.len() != 1 {
            bail!("flash works on one project at a time; pass a project name");
        }

        // Build everything first so a compile error can't leave the device
        // with mismatched images on its cores.
        let mut images = Vec::new();
        for part in &parts {
            images.push((*part, self.build(part, !sel.debug)?));
        }

        for (part, elf) in &images {
            let chip = part
                .meta
                .chip
                .as_deref()
                .with_context(|| format!("{} has no `chip` in its metadata", part.label()))?;
            eprintln!("==> flash {} ({chip})", part.label());
            let mut cmd = Command::new("probe-rs");
            cmd.args(["download", "--chip", chip])
                .args(&part.meta.flash_args)
                .arg(elf);
            run(cmd)?;
        }

        if let Some(chip) = images.iter().find_map(|(p, _)| p.meta.chip.as_deref()) {
            let mut cmd = Command::new("probe-rs");
            cmd.args(["reset", "--chip", chip]);
            run(cmd)?;
        }
        Ok(())
    }

    fn test(&self) -> Result<()> {
        if self.host_packages.is_empty() {
            eprintln!("no host-testable crates");
            return Ok(());
        }
        let mut cmd = cargo();
        cmd.current_dir(&self.root).arg("test");
        for package in &self.host_packages {
            cmd.args(["--package", package]);
        }
        run(cmd)
    }
}

fn cargo() -> Command {
    Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
}

fn run(mut cmd: Command) -> Result<()> {
    let status = cmd.status().with_context(|| format!("spawning {cmd:?}"))?;
    if !status.success() {
        bail!("command failed ({status}): {cmd:?}");
    }
    Ok(())
}
