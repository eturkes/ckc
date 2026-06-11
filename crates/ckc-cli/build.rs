//! Build-time provenance: bakes the repository commit into every ckc-cli
//! target as `CKC_GIT_COMMIT` — the §5 `RunManifest::git_commit` value, the
//! commit the run was *built* at (a fact of the binary, not of whatever
//! directory a command later runs in). M1 builds happen only inside the
//! development checkout, so an unreadable repository is a build error.

use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
    let repo_root = Path::new(&manifest_dir)
        .ancestors()
        .nth(2)
        .expect("crates/ckc-cli sits two levels under the repo root")
        .to_path_buf();
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&repo_root)
        .output()
        .expect("git rev-parse HEAD: git must be invocable at build time");
    assert!(
        out.status.success(),
        "git rev-parse HEAD failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let commit = String::from_utf8(out.stdout)
        .expect("a commit id is ASCII hex")
        .trim()
        .to_owned();
    assert!(!commit.is_empty(), "git rev-parse HEAD returned nothing");
    println!("cargo:rustc-env=CKC_GIT_COMMIT={commit}");

    // Re-run exactly when HEAD moves: the HEAD file (branch switches,
    // detached checkouts), the checked-out branch's ref file, and
    // packed-refs (gc can move the tip there). Only existing paths are
    // watched — naming a missing path would re-run every build.
    let git_dir = repo_root.join(".git");
    println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
    if let Ok(head) = std::fs::read_to_string(git_dir.join("HEAD"))
        && let Some(target) = head.trim().strip_prefix("ref: ")
    {
        let ref_file = git_dir.join(target);
        if ref_file.exists() {
            println!("cargo:rerun-if-changed={}", ref_file.display());
        }
    }
    let packed = git_dir.join("packed-refs");
    if packed.exists() {
        println!("cargo:rerun-if-changed={}", packed.display());
    }
}
