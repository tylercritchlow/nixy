use std::path::Path;
use std::process::Command;

use serde_json::Value;

use super::manifest::{
    DevShell, Manifest, Package, current_system, detect_project_type, dev_shell_from_tools,
    flake_info_from_metadata, inputs_from_metadata, package_from_meta,
};

#[derive(Debug, thiserror::Error)]
pub enum NixError {
    #[error("nix not found on PATH: {0}")]
    NixNotOnPath(#[source] std::io::Error),
    #[error("nix {cmd} failed{status}: {stderr}")]
    CommandFailed {
        cmd: String,
        status: String,
        stderr: String,
    },
    #[error("failed to parse nix output: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

const EXPERIMENTAL: [&str; 2] = ["--extra-experimental-features", "nix-command flakes"];

fn run_nix_json(dir: &Path, args: &[&str]) -> Result<Value, NixError> {
    let cmd_string = format!("nix {}", args.join(" "));
    let output = Command::new("nix")
        .args(EXPERIMENTAL.iter().copied())
        .args(args.iter().copied())
        .current_dir(dir)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                NixError::NixNotOnPath(e)
            } else {
                NixError::Io(e)
            }
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let status = output
            .status
            .code()
            .map(|c| format!(" (exit {c})"))
            .unwrap_or_default();
        return Err(NixError::CommandFailed {
            cmd: cmd_string,
            status,
            stderr,
        });
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(serde_json::from_str(&stdout)?)
}

fn try_run_nix_json(dir: &Path, args: &[&str]) -> Option<Value> {
    run_nix_json(dir, args).ok()
}

pub fn manifest_for(dir: &Path) -> Result<Manifest, NixError> {
    let metadata = run_nix_json(dir, &["flake", "metadata", "--json"])?;
    let flake = flake_info_from_metadata(&metadata);
    let inputs = inputs_from_metadata(&metadata);

    let system = current_system();
    let dev_shell = dev_shell_for(dir, &system)?;
    let packages = packages_for(dir, &system);
    let project_type = detect_project_type(dir);

    Ok(Manifest {
        system,
        flake,
        inputs,
        dev_shell,
        packages,
        project_type,
    })
}

fn dev_shell_for(dir: &Path, system: &str) -> Result<DevShell, NixError> {
    let attr = format!(".#devShells.{system}.default.nativeBuildInputs");
    let apply = r#"map (p: p.name or p.pname or "${p}")"#;
    let value = run_nix_json(
        dir,
        &[
            "eval",
            "--json",
            "--no-write-lock-file",
            &attr,
            "--apply",
            apply,
        ],
    )?;
    Ok(dev_shell_from_tools(&value))
}

fn packages_for(dir: &Path, system: &str) -> Vec<Package> {
    let attrs = try_run_nix_json(
        dir,
        &[
            "eval",
            "--json",
            "--no-write-lock-file",
            &format!(".#packages.{system}"),
            "--apply",
            "builtins.attrNames",
        ],
    )
    .and_then(|v| v.as_array().map(|a| a.to_vec()))
    .unwrap_or_default();

    attrs
        .iter()
        .filter_map(|a| a.as_str().map(str::to_string))
        .map(|attr| package_for(dir, system, &attr))
        .collect()
}

fn package_for(dir: &Path, system: &str, attr: &str) -> Package {
    let path = format!(".#packages.{system}.{attr}.meta");
    match try_run_nix_json(dir, &["eval", "--json", "--no-write-lock-file", &path]) {
        Some(meta) => package_from_meta(attr, &meta),
        None => Package {
            attr: attr.to_string(),
            name: attr.to_string(),
            description: None,
            license: None,
            main_program: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nix::manifest::ProjectType;

    #[test]
    #[ignore = "requires nix and runs against this repo's flake"]
    fn manifest_for_this_repo() {
        let manifest = manifest_for(Path::new(".")).expect("manifest_for should succeed");
        assert_eq!(manifest.system, current_system());
        assert_eq!(manifest.project_type, ProjectType::Rust);
        assert!(
            manifest
                .dev_shell
                .tools
                .iter()
                .any(|t| t.name.contains("rust")),
            "dev shell should expose a rust tool: {:?}",
            manifest.dev_shell.tools
        );
        assert!(
            manifest.inputs.iter().any(|i| i.name == "nixpkgs"),
            "inputs should include nixpkgs: {:?}",
            manifest.inputs
        );
    }
}
