use std::process::Command;

fn main() {
    let pkg_version = env!("CARGO_PKG_VERSION");

    let git_output = Command::new("git")
        .args(["describe", "--exact-match", "--tags", "HEAD"])
        .output();

    let version = match git_output {
        Ok(output) if output.status.success() => {
            // This is a tagged release build
            pkg_version.to_string()
        }
        _ => {
            // This is a dev build
            let git_hash_output = Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                .unwrap();
            let git_hash = String::from_utf8(git_hash_output.stdout).unwrap();
            format!("{}-{}", pkg_version, git_hash.trim())
        }
    };

    println!("cargo:rustc-env=DYNAMIC_VERSION={version}");
}
