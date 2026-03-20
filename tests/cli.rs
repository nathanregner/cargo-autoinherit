use assert_cmd::Command;
use cargo_manifest::{Dependency, Manifest};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn autoinherit_cmd(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("cargo-autoinherit").unwrap();
    cmd.arg("autoinherit").current_dir(dir.path());
    cmd
}

fn copy_fixture(fixture_name: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture_name);
    copy_dir_recursive(&fixture_path, dir.path());
    dir
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path).unwrap();
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).unwrap();
        }
    }
}

fn read_manifest(path: &Path) -> Manifest {
    let contents = fs::read_to_string(path).unwrap();
    toml::from_str(&contents).unwrap()
}

fn is_inherited(dep: &Dependency) -> bool {
    matches!(dep, Dependency::Inherited(_))
}

fn is_simple_version(dep: &Dependency, expected_version: &str) -> bool {
    matches!(dep, Dependency::Simple(v) if v == expected_version)
}

#[test]
fn shared_only_skips_single_use_deps() {
    let dir = copy_fixture("shared_only");

    autoinherit_cmd(&dir).arg("--shared-only").assert().success();

    let root = read_manifest(&dir.path().join("Cargo.toml"));
    let workspace_deps = root.workspace.unwrap().dependencies.unwrap();
    assert!(
        workspace_deps.contains_key("serde"),
        "serde should be in workspace.dependencies"
    );
    assert!(
        !workspace_deps.contains_key("rand"),
        "rand should NOT be in workspace.dependencies with --shared-only"
    );

    let pkg_a = read_manifest(&dir.path().join("pkg_a/Cargo.toml"));
    let pkg_a_deps = pkg_a.dependencies.unwrap();
    assert!(
        is_inherited(pkg_a_deps.get("serde").unwrap()),
        "serde in pkg_a should be inherited"
    );
    assert!(
        is_simple_version(pkg_a_deps.get("rand").unwrap(), "0.8"),
        "rand in pkg_a should remain a direct dependency"
    );

    let pkg_b = read_manifest(&dir.path().join("pkg_b/Cargo.toml"));
    let pkg_b_deps = pkg_b.dependencies.unwrap();
    assert!(
        is_inherited(pkg_b_deps.get("serde").unwrap()),
        "serde in pkg_b should be inherited"
    );
}

#[test]
fn without_shared_only_inherits_all_deps() {
    let dir = copy_fixture("shared_only");

    autoinherit_cmd(&dir).assert().success();

    let root = read_manifest(&dir.path().join("Cargo.toml"));
    let workspace_deps = root.workspace.unwrap().dependencies.unwrap();
    assert!(
        workspace_deps.contains_key("serde"),
        "serde should be in workspace.dependencies"
    );
    assert!(
        workspace_deps.contains_key("rand"),
        "rand should be in workspace.dependencies without --shared-only"
    );

    let pkg_a = read_manifest(&dir.path().join("pkg_a/Cargo.toml"));
    let pkg_a_deps = pkg_a.dependencies.unwrap();
    assert!(
        is_inherited(pkg_a_deps.get("serde").unwrap()),
        "serde in pkg_a should be inherited"
    );
    assert!(
        is_inherited(pkg_a_deps.get("rand").unwrap()),
        "rand in pkg_a should be inherited"
    );
}

#[test]
fn shared_only_counts_workspace_inherited_deps() {
    let dir = copy_fixture("already_inherited");

    autoinherit_cmd(&dir).arg("--shared-only").assert().success();

    let root = read_manifest(&dir.path().join("Cargo.toml"));
    let workspace_deps = root.workspace.unwrap().dependencies.unwrap();
    assert!(
        workspace_deps.contains_key("serde"),
        "serde should be in workspace.dependencies (multi-use)"
    );
    assert!(
        workspace_deps.contains_key("tokio"),
        "tokio should remain in workspace.dependencies"
    );

    let pkg_a = read_manifest(&dir.path().join("pkg_a/Cargo.toml"));
    let pkg_a_deps = pkg_a.dependencies.unwrap();
    assert!(
        is_inherited(pkg_a_deps.get("tokio").unwrap()),
        "tokio in pkg_a should be inherited"
    );
    assert!(
        is_inherited(pkg_a_deps.get("serde").unwrap()),
        "serde in pkg_a should be inherited"
    );

    let pkg_b = read_manifest(&dir.path().join("pkg_b/Cargo.toml"));
    let pkg_b_deps = pkg_b.dependencies.unwrap();
    assert!(
        is_inherited(pkg_b_deps.get("serde").unwrap()),
        "serde in pkg_b should be inherited"
    );
}

#[test]
fn prune_removes_unused_workspace_deps() {
    let dir = copy_fixture("unused_workspace_dep");

    autoinherit_cmd(&dir).assert().success();

    let root = read_manifest(&dir.path().join("Cargo.toml"));
    let workspace_deps = root.workspace.unwrap().dependencies.unwrap();
    assert!(
        workspace_deps.contains_key("serde"),
        "serde should remain in workspace.dependencies (it's used)"
    );
    assert!(
        !workspace_deps.contains_key("unused_dep"),
        "unused_dep should be removed from workspace.dependencies"
    );
}

#[test]
fn prune_false_keeps_unused_workspace_deps() {
    let dir = copy_fixture("unused_workspace_dep");

    autoinherit_cmd(&dir).arg("--prune=false").assert().success();

    let root = read_manifest(&dir.path().join("Cargo.toml"));
    let workspace_deps = root.workspace.unwrap().dependencies.unwrap();
    assert!(
        workspace_deps.contains_key("serde"),
        "serde should remain in workspace.dependencies"
    );
    assert!(
        workspace_deps.contains_key("unused_dep"),
        "unused_dep should remain with --prune=false"
    );
}
