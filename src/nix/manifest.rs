use std::path::Path;

use serde_json::Value;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Manifest {
    pub system: String,
    pub flake: FlakeInfo,
    pub inputs: Vec<Input>,
    pub dev_shell: DevShell,
    pub packages: Vec<Package>,
    pub project_type: ProjectType,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlakeInfo {
    pub path: String,
    pub revision: Option<String>,
    pub last_modified: Option<u64>,
    pub url: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Input {
    pub name: String,
    pub r#type: String,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub rev: Option<String>,
    pub last_modified: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct DevShell {
    pub tools: Vec<Tool>,
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Package {
    pub attr: String,
    pub name: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub main_program: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    Unknown,
}

impl ProjectType {
    fn label(self) -> &'static str {
        match self {
            ProjectType::Rust => "Rust",
            ProjectType::Node => "Node",
            ProjectType::Python => "Python",
            ProjectType::Go => "Go",
            ProjectType::Unknown => "Unknown",
        }
    }
}

impl Manifest {
    pub fn summarize(&self) -> String {
        let rev = self
            .flake
            .revision
            .as_deref()
            .map(|r| format!(" (rev {})", short_rev(r)))
            .unwrap_or_default();
        let tools = self
            .dev_shell
            .tools
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let inputs = self
            .inputs
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "Nix flake: {url}{rev}\nSystem: {system}\nProject: {project}\nDevShell: {tools}\nInputs: {inputs}",
            url = self.flake.url,
            system = self.system,
            project = self.project_type.label(),
        )
    }
}

fn short_rev(rev: &str) -> String {
    rev.chars().take(7).collect()
}

pub fn current_system() -> String {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "x86_64-darwin".to_string(),
        ("macos", "aarch64") => "aarch64-darwin".to_string(),
        ("linux", "x86_64") => "x86_64-linux".to_string(),
        ("linux", "aarch64") => "aarch64-linux".to_string(),
        (os, arch) => format!("{arch}-{os}"),
    }
}

pub fn detect_project_type(dir: &Path) -> ProjectType {
    if dir.join("Cargo.toml").is_file() {
        ProjectType::Rust
    } else if dir.join("package.json").is_file() {
        ProjectType::Node
    } else if dir.join("pyproject.toml").is_file() || dir.join("requirements.txt").is_file() {
        ProjectType::Python
    } else if dir.join("go.mod").is_file() {
        ProjectType::Go
    } else {
        ProjectType::Unknown
    }
}

pub fn flake_info_from_metadata(value: &Value) -> FlakeInfo {
    FlakeInfo {
        path: value
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        revision: value
            .get("revision")
            .and_then(Value::as_str)
            .map(str::to_string),
        last_modified: value.get("lastModified").and_then(Value::as_u64),
        url: value
            .get("originalUrl")
            .or_else(|| value.get("url"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    }
}

pub fn inputs_from_metadata(value: &Value) -> Vec<Input> {
    let Some(nodes) = value
        .get("locks")
        .and_then(|v| v.get("nodes"))
        .and_then(Value::as_object)
    else {
        return Vec::new();
    };
    let mut inputs: Vec<Input> = nodes
        .iter()
        .map(|(name, node)| Input {
            name: name.clone(),
            r#type: node
                .get("locked")
                .and_then(|l| l.get("type"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            owner: node
                .get("locked")
                .and_then(|l| l.get("owner"))
                .and_then(Value::as_str)
                .map(str::to_string),
            repo: node
                .get("locked")
                .and_then(|l| l.get("repo"))
                .and_then(Value::as_str)
                .map(str::to_string),
            rev: node
                .get("locked")
                .and_then(|l| l.get("rev"))
                .and_then(Value::as_str)
                .map(str::to_string),
            last_modified: node
                .get("locked")
                .and_then(|l| l.get("lastModified"))
                .and_then(Value::as_u64),
        })
        .collect();
    inputs.retain(|i| !i.name.is_empty() && i.name != "root" && !i.r#type.is_empty());
    inputs.sort_by(|a, b| a.name.cmp(&b.name));
    inputs
}

pub fn dev_shell_from_tools(value: &Value) -> DevShell {
    let tools = value
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(|name| Tool {
                    name: name.to_string(),
                })
                .collect()
        })
        .unwrap_or_default();
    DevShell { tools }
}

pub fn package_from_meta(attr: &str, value: &Value) -> Package {
    Package {
        attr: attr.to_string(),
        name: value
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(attr)
            .to_string(),
        description: value
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_string),
        license: value
            .get("license")
            .and_then(|l| l.get("shortName"))
            .and_then(Value::as_str)
            .map(str::to_string),
        main_program: value
            .get("mainProgram")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch_dir(prefix: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("nixy-test-{}-{}", prefix, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn current_system_is_arch_os() {
        let sys = current_system();
        assert!(sys.contains('-'), "system should be arch-os: {sys}");
    }

    #[test]
    fn detect_rust_project() {
        let dir = scratch_dir("rust");
        fs::write(dir.join("Cargo.toml"), "").unwrap();
        assert_eq!(detect_project_type(&dir), ProjectType::Rust);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn detect_node_project() {
        let dir = scratch_dir("node");
        fs::write(dir.join("package.json"), "{}").unwrap();
        assert_eq!(detect_project_type(&dir), ProjectType::Node);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn detect_unknown_project() {
        let dir = scratch_dir("empty");
        assert_eq!(detect_project_type(&dir), ProjectType::Unknown);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn inputs_parsed_from_metadata_fixture() {
        let raw = r#"{
            "path": "/nix/store/abc-source",
            "revision": "1d3ab75d61dc803f5600bb485e545afc4bfdf89c",
            "lastModified": 1782775420,
            "originalUrl": "file:///tmp/nixy",
            "locks": {
                "nodes": {
                    "root": { "locked": null, "inputs": { "nixpkgs": "nixpkgs" } },
                    "nixpkgs": { "locked": { "type": "github", "owner": "nixos", "repo": "nixpkgs", "rev": "e73", "lastModified": 1782467914 } },
                    "naersk": { "locked": { "type": "github", "owner": "nix-community", "repo": "naersk", "rev": "9aa0", "lastModified": 1782220280 } },
                    "no-lock": { "locked": null }
                }
            }
        }"#;
        let v: Value = serde_json::from_str(raw).unwrap();
        let info = flake_info_from_metadata(&v);
        assert_eq!(info.path, "/nix/store/abc-source");
        assert_eq!(
            info.revision.as_deref(),
            Some("1d3ab75d61dc803f5600bb485e545afc4bfdf89c")
        );
        assert_eq!(info.url, "file:///tmp/nixy");

        let inputs = inputs_from_metadata(&v);
        let names: Vec<&str> = inputs.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"nixpkgs"));
        assert!(names.contains(&"naersk"));
        assert!(!names.contains(&"root"));
        assert!(!names.contains(&"no-lock")); // no locked info -> dropped
        let nixpkgs = inputs.iter().find(|i| i.name == "nixpkgs").unwrap();
        assert_eq!(nixpkgs.r#type, "github");
        assert_eq!(nixpkgs.owner.as_deref(), Some("nixos"));
    }

    #[test]
    fn dev_shell_tools_parsed_from_array() {
        let v: Value = serde_json::from_str(r#"["rust-default-1.96.0","openssl-3.6.2"]"#).unwrap();
        let shell = dev_shell_from_tools(&v);
        assert_eq!(shell.tools.len(), 2);
        assert_eq!(shell.tools[0].name, "rust-default-1.96.0");
    }

    #[test]
    fn summarize_lists_system_project_tools_inputs() {
        let m = Manifest {
            system: "x86_64-darwin".to_string(),
            flake: FlakeInfo {
                path: "/nix/store/x".to_string(),
                revision: Some("1d3ab75d61dc".to_string()),
                last_modified: None,
                url: "file:///tmp/nixy".to_string(),
            },
            inputs: vec![Input {
                name: "nixpkgs".to_string(),
                r#type: "github".to_string(),
                owner: None,
                repo: None,
                rev: None,
                last_modified: None,
            }],
            dev_shell: DevShell {
                tools: vec![Tool {
                    name: "rust-default-1.96.0".to_string(),
                }],
            },
            packages: Vec::new(),
            project_type: ProjectType::Rust,
        };
        let s = m.summarize();
        assert!(s.contains("System: x86_64-darwin"));
        assert!(s.contains("Project: Rust"));
        assert!(s.contains("DevShell: rust-default-1.96.0"));
        assert!(s.contains("Inputs: nixpkgs"));
        assert!(s.contains("rev 1d3ab75"));
    }
}
