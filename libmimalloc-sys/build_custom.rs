use std::fs;
use std::path::Path;
use std::process::Command;

/// Check that there are no local modifications in the given folder.
fn guard_clean(folder: impl AsRef<Path>) {
    let folder = folder.as_ref();

    // Check for unstaged changes.
    let status = Command::new("git")
        .arg("diff")
        .arg("--exit-code")
        .current_dir(folder)
        .status()
        .expect("failed to execute `git diff`");
    if !status.success() {
        panic!(
            "There are uncommitted changes in {}. Commit them first.",
            folder.display()
        );
    }

    // Check for staged-but-uncommitted changes.
    let status = Command::new("git")
        .arg("diff-index")
        .arg("--quiet")
        .arg("--cached")
        .arg("HEAD")
        .current_dir(folder)
        .status()
        .expect("failed to execute `git diff-index`");
    if !status.success() {
        panic!(
            "There are staged but uncommitted changes in {}. Commit them first.",
            folder.display()
        );
    }
}

/// Iterate through all files ending with ".patch" in the given patch directory
/// and apply each patch to the target directory using the `patch` command.
fn apply_patches(target: impl AsRef<Path>, patch_dir: impl AsRef<Path>) {
    let target = target.as_ref();
    let patch_dir = patch_dir.as_ref();

    // Early exit if the patch directory doesn't exist or isn't a directory.
    if !patch_dir.exists() || !patch_dir.is_dir() {
        println!(
            "cargo:warning=No patch directory found at {}. Skipping patch application.",
            patch_dir.display()
        );
        return;
    }

    for entry in fs::read_dir(patch_dir).expect("Failed to read patch directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.extension().map(|ext| ext == "patch").unwrap_or(false) {
            // Convert the patch file to an absolute (canonical) path.
            let abs_path = fs::canonicalize(&path).expect("Failed to canonicalize patch file");
            println!("cargo:warning=Applying patch: {}", abs_path.display());
            let status = Command::new("patch")
                .arg("-p1")
                .arg("-i")
                .arg(abs_path.to_str().expect("Non UTF-8 patch file path"))
                .current_dir(target)
                .status()
                .expect("failed to execute patch command");
            if !status.success() {
                panic!("Failed to apply patch: {}", abs_path.display());
            }
        }
    }
}

pub fn check_and_apply_patches() {
    let target_folder = "c_src/mimalloc";
    let patch_folder = "patches";

    println!("cargo:rerun-if-changed={target_folder}");
    println!("cargo:rerun-if-changed={patch_folder}");

    guard_clean(target_folder);
    apply_patches(target_folder, patch_folder);
}

pub fn clean_patches() {
    println!("cargo:warning=Resetting c_src/mimalloc to pristine state");

    // Reset tracked changes.
    let reset_status = Command::new("git")
        .args(&["reset", "--hard", "HEAD"])
        .current_dir("c_src/mimalloc")
        .status()
        .expect("Failed to execute git reset");
    if !reset_status.success() {
        panic!("git reset failed in c_src/mimalloc");
    }
}
