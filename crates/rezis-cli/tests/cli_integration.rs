use std::path::PathBuf;
use std::process::Command;

#[test]
fn new_project_files_exist() {
    let dir = tempfile::tempdir().unwrap();
    let rezis_lib = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../rezis")
        .canonicalize()
        .unwrap();

    let ok = Command::new(env!("CARGO_BIN_EXE_rezis"))
        .args(["new", "my-api", "--rezis-path"])
        .arg(&rezis_lib)
        .current_dir(dir.path())
        .status()
        .unwrap()
        .success();
    assert!(ok, "rezis new failed");

    let proj = dir.path().join("my-api");
    assert!(proj.join("Cargo.toml").is_file());
    assert!(proj.join("src/main.rs").is_file());
    assert!(proj.join("src/app_module.rs").is_file());
    assert!(proj
        .join("src/modules/health/health_controller.rs")
        .is_file());
}

#[test]
fn new_project_cargo_check() {
    let dir = tempfile::tempdir().unwrap();
    let rezis_lib = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../rezis")
        .canonicalize()
        .unwrap();

    assert!(Command::new(env!("CARGO_BIN_EXE_rezis"))
        .args(["new", "chk-api", "--rezis-path"])
        .arg(&rezis_lib)
        .current_dir(dir.path())
        .status()
        .unwrap()
        .success());

    let proj = dir.path().join("chk-api");
    assert!(
        Command::new("cargo")
            .args(["check", "--manifest-path"])
            .arg(proj.join("Cargo.toml"))
            .status()
            .unwrap()
            .success(),
        "cargo check failed for generated project"
    );
}

#[test]
fn generate_resource_patches_modules() {
    let dir = tempfile::tempdir().unwrap();
    let rezis_lib = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../rezis")
        .canonicalize()
        .unwrap();

    assert!(Command::new(env!("CARGO_BIN_EXE_rezis"))
        .args(["new", "gen-api", "--rezis-path"])
        .arg(&rezis_lib)
        .current_dir(dir.path())
        .status()
        .unwrap()
        .success());

    let proj = dir.path().join("gen-api");
    assert!(Command::new(env!("CARGO_BIN_EXE_rezis"))
        .args(["g", "resource", "widgets"])
        .current_dir(&proj)
        .status()
        .unwrap()
        .success());

    assert!(proj
        .join("src/modules/widgets/widgets_controller.rs")
        .is_file());
    let mods = std::fs::read_to_string(proj.join("src/modules/mod.rs")).unwrap();
    assert!(mods.contains("pub mod widgets;"));
    let app = std::fs::read_to_string(proj.join("src/app_module.rs")).unwrap();
    assert!(app.contains("WidgetsModule"));
}
