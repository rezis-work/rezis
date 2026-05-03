//! Library surface for `rezis-cli` (binary wraps [`run`]).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

pub use clap::Parser;

/// NestJS-style scaffolding for Rezis (`rezis new`, `rezis g …`).
#[derive(Parser, Debug)]
#[command(name = "rezis", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    /// Create a new API crate in `./<NAME>/`.
    New {
        /// Package directory name (e.g. `blog-api`).
        name: String,
        /// Overwrite existing files / directory.
        #[arg(long)]
        force: bool,
        /// Use a path dependency for `rezis` (for tests / local dev).
        #[arg(long, value_name = "PATH")]
        rezis_path: Option<PathBuf>,
    },
    /// Generate modules, stubs, or a full resource.
    #[command(alias = "g")]
    Generate {
        #[command(subcommand)]
        gen: GenCommands,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum GenCommands {
    /// `src/modules/<name>/` + `<name>_module.rs`.
    Module {
        name: String,
        #[arg(long)]
        force: bool,
    },
    /// Stub controller (requires `src/modules/<name>/`).
    Controller {
        name: String,
        #[arg(long)]
        force: bool,
    },
    /// Stub service.
    Service {
        name: String,
        #[arg(long)]
        force: bool,
    },
    /// Stub DTO with serde + validator.
    Dto {
        name: String,
        #[arg(long)]
        force: bool,
    },
    /// Controller + service + dto + module; patches `modules/mod.rs` and `app_module.rs`.
    Resource {
        name: String,
        #[arg(long)]
        force: bool,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::New {
            name,
            force,
            rezis_path,
        } => cmd_new(&name, force, rezis_path.as_deref()),
        Commands::Generate { gen } => cmd_generate(gen),
    }
}

fn cmd_new(name: &str, force: bool, rezis_path: Option<&Path>) -> anyhow::Result<()> {
    let pkg = package_label(name)?;
    let root = std::env::current_dir().context("cwd")?.join(&pkg);
    if root.exists() && !force {
        bail!(
            "destination `{}` already exists (pass `--force` to overwrite)",
            root.display()
        );
    }
    if force && root.exists() {
        fs::remove_dir_all(&root).with_context(|| format!("remove {}", root.display()))?;
    }
    fs::create_dir_all(root.join("src/modules/health")).context("create health dirs")?;
    fs::create_dir_all(root.join("src/common")).context("create common dirs")?;

    write_file(
        &root.join("Cargo.toml"),
        &cargo_toml(&pkg, rezis_path),
        force,
    )?;
    write_file(&root.join("README.md"), README_MD, force)?;
    write_file(&root.join(".env.example"), DOTENV, force)?;
    write_file(&root.join(".env"), DOTENV, force)?;
    write_file(&root.join("src/main.rs"), MAIN_RS, force)?;
    write_file(&root.join("src/app_module.rs"), APP_MODULE_RS, force)?;
    write_file(&root.join("src/modules/mod.rs"), "pub mod health;\n", force)?;
    write_file(
        &root.join("src/modules/health/mod.rs"),
        HEALTH_MOD_RS,
        force,
    )?;
    write_file(
        &root.join("src/modules/health/health_controller.rs"),
        HEALTH_CONTROLLER_RS,
        force,
    )?;
    write_file(
        &root.join("src/modules/health/health_module.rs"),
        HEALTH_MODULE_RS,
        force,
    )?;
    write_file(&root.join("src/common/error.rs"), COMMON_ERROR_RS, force)?;
    write_file(
        &root.join("src/common/response.rs"),
        COMMON_RESPONSE_RS,
        force,
    )?;
    write_file(&root.join("src/common/config.rs"), COMMON_CONFIG_RS, force)?;

    println!("Created Rezis project `{}` at {}", pkg, root.display());
    Ok(())
}

fn cargo_toml(package_name: &str, rezis_path: Option<&Path>) -> String {
    let rezis_dep = match rezis_path {
        Some(p) => {
            let path_str = normalize_path_for_manifest(p);
            format!(r#"rezis = {{ path = "{path_str}" }}"#)
        }
        None => r#"rezis = "0.1.0-alpha.1""#.to_string(),
    };
    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
{rezis_dep}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
tokio = {{ version = "1", features = ["macros", "rt-multi-thread"] }}
validator = {{ version = "0.20", features = ["derive"] }}
"#
    )
}

/// Absolute path string safe for `Cargo.toml` `path = "..."` on Windows (Cargo rejects `\\?\` verbatim paths).
fn normalize_path_for_manifest(p: &Path) -> String {
    let p = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    let raw = p.to_string_lossy();
    let without_verbatim = raw.strip_prefix(r"\\?\").unwrap_or(&raw);
    without_verbatim.replace('\\', "/")
}

fn cmd_generate(gen: GenCommands) -> anyhow::Result<()> {
    match gen {
        GenCommands::Module { name, force } => gen_module(&name, force),
        GenCommands::Controller { name, force } => gen_controller(&name, force),
        GenCommands::Service { name, force } => gen_service(&name, force),
        GenCommands::Dto { name, force } => gen_dto(&name, force),
        GenCommands::Resource { name, force } => gen_resource(&name, force),
    }
}

fn project_root(src_must_exist: bool) -> anyhow::Result<PathBuf> {
    let root = std::env::current_dir().context("cwd")?;
    let src = root.join("src");
    if src_must_exist && !src.is_dir() {
        bail!("missing `src/` — run `rezis new <name>` first (from parent directory)");
    }
    Ok(root)
}

fn gen_module(name: &str, force: bool) -> anyhow::Result<()> {
    let root = project_root(true)?;
    let sn = snake_identifier(name)?;
    let pascal = pascal_case(&sn);
    let dir = root.join("src/modules").join(&sn);
    fs::create_dir_all(&dir).with_context(|| format!("mkdir {}", dir.display()))?;

    let module_rs = format!(
        r#"use rezis::{{Module, ModuleContext}};

pub struct {pascal}Module;

impl {pascal}Module {{
    pub fn new() -> Self {{
        Self
    }}
}}

impl Module for {pascal}Module {{
    fn register(&self, _ctx: &mut ModuleContext<'_>) {{
        // Add ctx.controller(...) here
    }}
}}
"#
    );

    write_file(&dir.join(format!("{sn}_module.rs")), &module_rs, force)?;
    write_file(
        &dir.join("mod.rs"),
        &format!("pub mod {sn}_module;\n"),
        force,
    )?;

    patch_modules_mod(&root.join("src/modules/mod.rs"), &sn)?;
    patch_app_module(&root.join("src/app_module.rs"), &sn, &pascal)?;
    println!("Generated module `{sn}`");
    Ok(())
}

fn gen_controller(name: &str, force: bool) -> anyhow::Result<()> {
    let root = project_root(true)?;
    let sn = snake_identifier(name)?;
    let pascal = pascal_case(&sn);
    let dir = root.join("src/modules").join(&sn);
    if !dir.is_dir() {
        bail!(
            "missing `{}` — run `rezis g module {}` first",
            dir.display(),
            sn
        );
    }

    let ctrl = format!(
        r#"use rezis::{{json, Controller, RouteBuilder}};

#[derive(Clone, Copy)]
pub struct {pascal}Controller;

impl Controller for {pascal}Controller {{
    fn register<'a>(&self, routes: RouteBuilder<'a>) -> RouteBuilder<'a> {{
        routes.get("/{sn}", || async {{ json("ok") }})
    }}
}}
"#
    );

    write_file(&dir.join(format!("{sn}_controller.rs")), &ctrl, force)?;
    merge_mod_rs(&dir.join("mod.rs"), &format!("pub mod {sn}_controller;\n"))?;
    println!("Generated controller `{sn}_controller`");
    Ok(())
}

fn gen_service(name: &str, force: bool) -> anyhow::Result<()> {
    let root = project_root(true)?;
    let sn = snake_identifier(name)?;
    let pascal = pascal_case(&sn);
    let dir = root.join("src/modules").join(&sn);
    if !dir.is_dir() {
        bail!(
            "missing `{}` — run `rezis g module {}` first",
            dir.display(),
            sn
        );
    }

    let svc = format!(
        r#"#[derive(Clone, Default)]
pub struct {pascal}Service;

impl {pascal}Service {{
    pub fn new() -> Self {{
        Self
    }}
}}
"#
    );

    write_file(&dir.join(format!("{sn}_service.rs")), &svc, force)?;
    merge_mod_rs(&dir.join("mod.rs"), &format!("pub mod {sn}_service;\n"))?;
    println!("Generated service `{sn}_service`");
    Ok(())
}

fn gen_dto(name: &str, force: bool) -> anyhow::Result<()> {
    let root = project_root(true)?;
    let sn = snake_identifier(name)?;
    let pascal = pascal_case(&sn);
    let dir = root.join("src/modules").join(&sn);
    if !dir.is_dir() {
        bail!(
            "missing `{}` — run `rezis g module {}` first",
            dir.display(),
            sn
        );
    }

    let dto_name = format!("Create{pascal}Dto");
    let dto = format!(
        r#"use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct {dto_name} {{
    #[validate(length(min = 1))]
    pub name: String,
}}
"#
    );

    write_file(&dir.join(format!("{sn}_dto.rs")), &dto, force)?;
    merge_mod_rs(&dir.join("mod.rs"), &format!("pub mod {sn}_dto;\n"))?;
    println!("Generated dto `{sn}_dto`");
    Ok(())
}

fn gen_resource(name: &str, force: bool) -> anyhow::Result<()> {
    let root = project_root(true)?;
    let sn = snake_identifier(name)?;
    let pascal = pascal_case(&sn);
    let dir = root.join("src/modules").join(&sn);
    fs::create_dir_all(&dir).with_context(|| format!("mkdir {}", dir.display()))?;

    let dto_name = format!("Create{pascal}Dto");
    let item = format!("{pascal}Item");

    let dto = format!(
        r#"use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct {dto_name} {{
    #[validate(length(min = 1))]
    pub name: String,
}}
"#
    );

    let service = format!(
        r#"use serde::Serialize;

use super::{sn}_dto::{dto_name};

#[derive(Debug, Clone, Serialize)]
pub struct {item} {{
    pub id: u64,
    pub name: String,
}}

#[derive(Clone, Default)]
pub struct {pascal}Service;

impl {pascal}Service {{
    pub fn new() -> Self {{
        Self
    }}

    pub async fn find_all(&self) -> Vec<{item}> {{
        vec![]
    }}

    pub async fn create(&self, dto: {dto_name}) -> {item} {{
        {item} {{
            id: 1,
            name: dto.name,
        }}
    }}
}}
"#
    );

    let controller = format!(
        r#"use rezis::{{json, Controller, JsonResult, RouteBuilder, ValidatedJson}};

use super::{sn}_dto::{dto_name};
use super::{sn}_service::{{{item}, {pascal}Service}};

#[derive(Clone)]
pub struct {pascal}Controller {{
    service: {pascal}Service,
}}

impl {pascal}Controller {{
    pub fn new(service: {pascal}Service) -> Self {{
        Self {{ service }}
    }}

    async fn create_item(&self, dto: {dto_name}) -> JsonResult<{item}> {{
        Ok(json(self.service.create(dto).await))
    }}
}}

impl Controller for {pascal}Controller {{
    fn register<'a>(&self, routes: RouteBuilder<'a>) -> RouteBuilder<'a> {{
        routes
            .get("/{sn}", {{
                let c = self.clone();
                || async move {{ json(c.service.find_all().await) }}
            }})
            .post("/{sn}", {{
                let c = self.clone();
                |ValidatedJson(body)| async move {{ c.create_item(body).await }}
            }})
    }}
}}
"#
    );

    let module = format!(
        r#"use rezis::{{Module, ModuleContext}};

use super::{sn}_controller::{pascal}Controller;
use super::{sn}_service::{pascal}Service;

pub struct {pascal}Module;

impl {pascal}Module {{
    pub fn new() -> Self {{
        Self
    }}
}}

impl Module for {pascal}Module {{
    fn register(&self, ctx: &mut ModuleContext<'_>) {{
        let service = {pascal}Service::new();
        let controller = {pascal}Controller::new(service);
        ctx.controller(controller);
    }}
}}
"#
    );

    let mod_rs = format!(
        r#"pub mod {sn}_controller;
pub mod {sn}_dto;
pub mod {sn}_module;
pub mod {sn}_service;
"#
    );

    write_file(&dir.join(format!("{sn}_dto.rs")), &dto, force)?;
    write_file(&dir.join(format!("{sn}_service.rs")), &service, force)?;
    write_file(&dir.join(format!("{sn}_controller.rs")), &controller, force)?;
    write_file(&dir.join(format!("{sn}_module.rs")), &module, force)?;
    write_file(&dir.join("mod.rs"), &mod_rs, force)?;

    patch_modules_mod(&root.join("src/modules/mod.rs"), &sn)?;
    patch_app_module(&root.join("src/app_module.rs"), &sn, &pascal)?;
    println!("Generated resource `{sn}`");
    Ok(())
}

fn write_file(path: &Path, content: &str, force: bool) -> anyhow::Result<()> {
    if path.exists() && !force {
        bail!(
            "refusing to overwrite `{}` (pass `--force`)",
            path.display()
        );
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}

pub(crate) fn merge_mod_rs(path: &Path, line: &str) -> anyhow::Result<()> {
    let content = if path.exists() {
        fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?
    } else {
        String::new()
    };
    let trimmed = line.trim();
    if content.lines().any(|l| l.trim() == trimmed) {
        return Ok(());
    }
    let out = if content.is_empty() || content.ends_with('\n') {
        format!("{content}{line}")
    } else {
        format!("{content}\n{line}")
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
    }
    fs::write(path, out).with_context(|| format!("write {}", path.display()))
}

pub(crate) fn patch_modules_mod(path: &Path, module_name: &str) -> anyhow::Result<()> {
    if !path.exists() {
        bail!("missing `{}` — run `rezis new` first", path.display());
    }
    let line = format!("pub mod {module_name};\n");
    merge_mod_rs(path, &line)
}

fn patch_app_module(path: &Path, snake: &str, pascal: &str) -> anyhow::Result<()> {
    let mut content =
        fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;

    let use_line = format!("use crate::modules::{snake}::{snake}_module::{pascal}Module;");
    let needle = "ctx.module(HealthModule::new());";
    let ctx_line = format!("        ctx.module({pascal}Module::new());");

    if content.contains(&format!("{pascal}Module::new()")) {
        return Ok(());
    }

    if !content.contains(&use_line) {
        let anchor = "use rezis::{Module, ModuleContext};";
        if content.contains(anchor) {
            content = content.replacen(anchor, &format!("{anchor}\n{use_line}"), 1);
        } else {
            content = format!("{use_line}\n{content}");
        }
    }

    if content.contains(needle) && !content.contains(&ctx_line) {
        content = content.replacen(needle, &format!("{needle}\n{ctx_line}"), 1);
    } else if !content.contains(&ctx_line) {
        // Fallback: append inside register if HealthModule line missing
        bail!(
            "could not patch `app_module.rs`: expected `{}` as insertion anchor",
            needle
        );
    }

    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}

/// Cargo package / directory label (allows `-`, e.g. `blog-api`).
pub fn package_label(name: &str) -> anyhow::Result<String> {
    let s = name.trim();
    if s.is_empty() {
        bail!("name must not be empty");
    }
    if !s
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic())
        .unwrap_or(false)
    {
        bail!("project name must start with a letter");
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        bail!("project name must be ASCII alphanumeric with '-' or '_'");
    }
    Ok(s.to_string())
}

pub fn snake_identifier(name: &str) -> anyhow::Result<String> {
    let s = name.trim().replace('-', "_");
    if s.is_empty() {
        bail!("name must not be empty");
    }
    if !s
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or(false)
    {
        bail!("name must start with a letter or underscore");
    }
    if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        bail!("name must be alphanumeric ASCII or underscore");
    }
    Ok(s)
}

pub fn pascal_case(snake: &str) -> String {
    snake
        .split('_')
        .filter(|s| !s.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

const README_MD: &str = r#"# My Rezis API

Copy env template (optional):

```bash
cp .env.example .env
```

Run:

```bash
cargo run
```

`PORT` is read from `.env` via `RezisApp::listen_from_env`.
"#;

const DOTENV: &str = "PORT=3000\nRUST_LOG=info\n";

const MAIN_RS: &str = r#"mod app_module;
mod modules;

use app_module::AppModule;
use rezis::RezisApp;

#[tokio::main]
async fn main() {
    RezisApp::new()
        .module(AppModule::new())
        .listen_from_env()
        .await;
}
"#;

const APP_MODULE_RS: &str = r#"use rezis::{Module, ModuleContext};

use crate::modules::health::health_module::HealthModule;

pub struct AppModule;

impl AppModule {
    pub fn new() -> Self {
        Self
    }
}

impl Module for AppModule {
    fn register(&self, ctx: &mut ModuleContext<'_>) {
        ctx.module(HealthModule::new());
    }
}
"#;

const HEALTH_MOD_RS: &str = r#"pub mod health_controller;
pub mod health_module;
"#;

const HEALTH_CONTROLLER_RS: &str = r#"use rezis::{json, Controller, RouteBuilder};

#[derive(Clone, Copy)]
pub struct HealthController;

impl Controller for HealthController {
    fn register<'a>(&self, routes: RouteBuilder<'a>) -> RouteBuilder<'a> {
        routes.get("/health", || async {
            json(serde_json::json!({ "status": "ok" }))
        })
    }
}
"#;

const HEALTH_MODULE_RS: &str = r#"use rezis::{Module, ModuleContext};

use super::health_controller::HealthController;

pub struct HealthModule;

impl HealthModule {
    pub fn new() -> Self {
        Self
    }
}

impl Module for HealthModule {
    fn register(&self, ctx: &mut ModuleContext<'_>) {
        ctx.controller(HealthController);
    }
}
"#;

const COMMON_ERROR_RS: &str = r#"//! App-level error aliases — extend as needed.
pub use rezis::RezisError;
"#;

const COMMON_RESPONSE_RS: &str = r#"//! Shared JSON helpers.
pub use rezis::{json, ApiSuccess};
"#;

const COMMON_CONFIG_RS: &str = r#"//! Configuration helpers — uses framework defaults.
pub use rezis::RezisConfig;
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_label_keeps_hyphen() {
        assert_eq!(package_label("blog-api").unwrap(), "blog-api");
        assert!(package_label("-bad").is_err());
    }

    #[test]
    fn snake_and_pascal() {
        assert_eq!(snake_identifier("blog-api").unwrap(), "blog_api");
        assert_eq!(pascal_case("blog_api"), "BlogApi");
        assert_eq!(pascal_case("users"), "Users");
    }

    #[test]
    fn merge_mod_rs_inserts_once_and_is_idempotent() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("mod.rs");
        merge_mod_rs(&path, "pub mod foo;\n")?;
        merge_mod_rs(&path, "pub mod foo;\n")?;
        let s = fs::read_to_string(&path)?;
        assert_eq!(s.matches("pub mod foo").count(), 1);
        Ok(())
    }

    #[test]
    fn patch_modules_mod_appends_and_is_idempotent() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("mod.rs");
        fs::write(&path, "pub mod health;\n")?;
        patch_modules_mod(&path, "users")?;
        patch_modules_mod(&path, "users")?;
        let s = fs::read_to_string(&path)?;
        assert!(s.contains("pub mod health;"));
        assert_eq!(s.matches("pub mod users").count(), 1);
        Ok(())
    }
}
