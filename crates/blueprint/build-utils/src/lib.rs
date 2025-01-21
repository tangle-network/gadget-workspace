use gadget_std::env;
use gadget_std::path::PathBuf;
use gadget_std::process::Command;

/// Build the Smart contracts at the specified directories.
///
/// This function will automatically rerun the build if changes are detected in the `src`
/// directory within any of the directories specified. Due to this, it is recommended to
/// ensure that you only pass in directories that contain the `src` directory and won't be
/// modified by anything else in the build script (otherwise, the build will always rerun).
///
/// # Panics
/// - If the Cargo Manifest directory is not found.
/// - If the `forge` executable is not found.
pub fn build_contracts(contract_dirs: Vec<&str>) {
    for dir in contract_dirs.clone() {
        let dir = format!("{}/src", dir);
        println!("cargo:rerun-if-changed={dir}");
    }

    // Get the project root directory
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Try to find the `forge` executable dynamically
    let forge_executable = find_forge_executable();

    for dir in contract_dirs {
        let full_path = root.join(dir).canonicalize().unwrap_or_else(|_| {
            println!(
                "Directory not found or inaccessible: {}",
                root.join(dir).display()
            );
            root.join(dir)
        });

        if full_path.exists() {
            let status = Command::new(&forge_executable)
                .current_dir(&full_path)
                .arg("build")
                .status()
                .expect("Failed to execute Forge build");

            assert!(
                status.success(),
                "Forge build failed for directory: {}",
                full_path.display()
            );
        } else {
            println!(
                "Directory not found or does not exist: {}",
                full_path.display()
            );
        }
    }
}

/// Run soldeer's `update` command to populate the `dependencies` directory.
///
/// # Panics
/// - If the Cargo Manifest directory is not found.
/// - If the `forge` executable is not found.
/// - If forge's `soldeer` is not installed.
pub fn soldeer_update() {
    // Get the project root directory
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Try to find the `forge` executable dynamically
    let forge_executable = find_forge_executable();

    let status = Command::new(&forge_executable)
        .current_dir(&root)
        .args(["soldeer", "update", "-d"])
        .status()
        .expect("Failed to execute 'forge soldeer update'");

    assert!(status.success(), "'forge soldeer update' failed");
}

/// Returns a string with the path to the `forge` executable.
///
/// # Panics
/// - If the `forge` executable is not found i.e., if Foundry is not installed.
#[must_use]
pub fn find_forge_executable() -> String {
    // Try to find the `forge` executable dynamically
    match Command::new("which").arg("forge").output() {
        Ok(output) => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            assert!(
                !path.is_empty(),
                "Forge executable not found. Make sure Foundry is installed."
            );
            path
        }
        Err(e) => panic!("Failed to find `forge` executable: {e}"),
    }
}
