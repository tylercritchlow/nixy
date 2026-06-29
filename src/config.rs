use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub keybindings: Keybindings,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Keybindings {
    pub app: AppKeybindings,
    pub editor: EditorKeybindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppKeybindings {
    pub quit: String,
    pub quit_force: String,
}

impl Default for AppKeybindings {
    fn default() -> Self {
        Self {
            quit: "ctrl+d".to_string(),
            quit_force: "ctrl+c".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorKeybindings {
    pub newline: String,
    pub newline_alt: String,
    pub clear: String,
}

impl Default for EditorKeybindings {
    fn default() -> Self {
        Self {
            newline: "ctrl+j".to_string(),
            newline_alt: "alt+enter".to_string(),
            clear: "ctrl+u".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub code: KeyCode,
}

impl KeyBinding {
    pub fn matches(&self, key: &KeyEvent) -> bool {
        let mods = key.modifiers;
        if self.ctrl != mods.contains(KeyModifiers::CONTROL) {
            return false;
        }
        if self.alt != mods.contains(KeyModifiers::ALT) {
            return false;
        }
        if self.shift != mods.contains(KeyModifiers::SHIFT) {
            return false;
        }
        key.code == self.code
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config at {0}: {1}")]
    Read(PathBuf, #[source] std::io::Error),
    #[error("failed to parse config at {0}: {1}")]
    Parse(PathBuf, #[source] toml::de::Error),
    #[error("invalid key binding \"{binding}\": {reason}")]
    InvalidKey { binding: String, reason: String },
}

impl Config {
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("nixy").join("config.toml"))
    }

    pub fn load() -> Result<Self, ConfigError> {
        match Self::default_path() {
            Some(path) => Self::load_from(&path),
            None => Ok(Self::default()),
        }
    }

    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(contents) => toml::from_str::<Config>(&contents)
                .map_err(|e| ConfigError::Parse(path.to_path_buf(), e)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(ConfigError::Read(path.to_path_buf(), e)),
        }
    }
}

impl Keybindings {
    pub fn parse(&self) -> Result<ParsedKeybindings, ConfigError> {
        Ok(ParsedKeybindings {
            app: self.app.parse()?,
            editor: self.editor.parse()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ParsedKeybindings {
    pub app: ParsedAppKeybindings,
    pub editor: ParsedEditorKeybindings,
}

#[derive(Debug, Clone)]
pub struct ParsedAppKeybindings {
    pub quit: KeyBinding,
    pub quit_force: KeyBinding,
}

#[derive(Debug, Clone)]
pub struct ParsedEditorKeybindings {
    pub newline: KeyBinding,
    pub newline_alt: KeyBinding,
    pub clear: KeyBinding,
}

impl AppKeybindings {
    fn parse(&self) -> Result<ParsedAppKeybindings, ConfigError> {
        Ok(ParsedAppKeybindings {
            quit: parse_key(&self.quit)?,
            quit_force: parse_key(&self.quit_force)?,
        })
    }
}

impl EditorKeybindings {
    fn parse(&self) -> Result<ParsedEditorKeybindings, ConfigError> {
        Ok(ParsedEditorKeybindings {
            newline: parse_key(&self.newline)?,
            newline_alt: parse_key(&self.newline_alt)?,
            clear: parse_key(&self.clear)?,
        })
    }
}

pub fn parse_key(binding: &str) -> Result<KeyBinding, ConfigError> {
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut code_str: Option<&str> = None;

    for part in binding.split('+') {
        let lower = part.trim().to_ascii_lowercase();
        match lower.as_str() {
            "ctrl" | "control" => ctrl = true,
            "alt" => alt = true,
            "shift" => shift = true,
            _ => {
                if code_str.is_some() {
                    return Err(ConfigError::InvalidKey {
                        binding: binding.to_string(),
                        reason: "multiple key codes specified".to_string(),
                    });
                }
                code_str = Some(part.trim());
            }
        }
    }

    let raw = code_str.ok_or_else(|| ConfigError::InvalidKey {
        binding: binding.to_string(),
        reason: "no key code specified".to_string(),
    })?;

    let code = parse_code(raw).ok_or_else(|| ConfigError::InvalidKey {
        binding: binding.to_string(),
        reason: format!("unknown key: {raw}"),
    })?;

    Ok(KeyBinding {
        ctrl,
        alt,
        shift,
        code,
    })
}

fn parse_code(raw: &str) -> Option<KeyCode> {
    let lower = raw.to_ascii_lowercase();
    match lower.as_str() {
        "enter" | "return" => Some(KeyCode::Enter),
        "escape" | "esc" => Some(KeyCode::Esc),
        "backspace" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "tab" => Some(KeyCode::Tab),
        "space" => Some(KeyCode::Char(' ')),
        _ => {
            if let Some(rest) = lower.strip_prefix('f')
                && let Ok(n) = rest.parse::<u8>()
                && (1..=12).contains(&n)
            {
                return Some(KeyCode::F(n));
            }
            let mut chars = lower.chars();
            let c = chars.next()?;
            if chars.next().is_none() {
                Some(KeyCode::Char(c))
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_char() {
        let b = parse_key("a").unwrap();
        assert!(!b.ctrl && !b.alt && !b.shift);
        assert_eq!(b.code, KeyCode::Char('a'));
    }

    #[test]
    fn parse_ctrl_c() {
        let b = parse_key("ctrl+c").unwrap();
        assert!(b.ctrl && !b.alt && !b.shift);
        assert_eq!(b.code, KeyCode::Char('c'));
    }

    #[test]
    fn parse_alt_enter() {
        let b = parse_key("alt+enter").unwrap();
        assert!(!b.ctrl && b.alt && !b.shift);
        assert_eq!(b.code, KeyCode::Enter);
    }

    #[test]
    fn parse_shift_left() {
        let b = parse_key("shift+left").unwrap();
        assert!(!b.ctrl && !b.alt && b.shift);
        assert_eq!(b.code, KeyCode::Left);
    }

    #[test]
    fn parse_case_insensitive_modifiers() {
        let b = parse_key("CTRL+J").unwrap();
        assert!(b.ctrl);
        assert_eq!(b.code, KeyCode::Char('j'));
    }

    #[test]
    fn parse_function_key() {
        let b = parse_key("f5").unwrap();
        assert_eq!(b.code, KeyCode::F(5));
    }

    #[test]
    fn parse_unknown_key_errors() {
        assert!(parse_key("ctrl+xyz").is_err());
    }

    #[test]
    fn parse_no_code_errors() {
        assert!(parse_key("ctrl").is_err());
    }

    #[test]
    fn parse_multiple_codes_errors() {
        assert!(parse_key("a+b").is_err());
    }

    #[test]
    fn config_default_toml_roundtrip() {
        let cfg = Config::default();
        let s = toml::to_string(&cfg).unwrap();
        let parsed: Config = toml::from_str(&s).unwrap();
        assert_eq!(parsed.keybindings.app.quit, cfg.keybindings.app.quit);
    }

    #[test]
    fn config_partial_override_uses_defaults() {
        let s = r#"
[keybindings.app]
quit = "ctrl+q"
"#;
        let cfg: Config = toml::from_str(s).unwrap();
        assert_eq!(cfg.keybindings.app.quit, "ctrl+q");
        assert_eq!(cfg.keybindings.app.quit_force, "ctrl+c");
        assert_eq!(cfg.keybindings.editor.newline, "ctrl+j");
    }
}
