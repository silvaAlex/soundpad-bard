use anyhow::{anyhow,Result};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Hotkey {
    normalized: String,
    original: String,
}

impl Hotkey {
    pub fn new(input: &str) -> Result<Self> {
        let normalized = Self::normalize(input)?;

        Ok(Self {
            normalized,
            original: input.to_string(),
        })
    }

    fn normalize(raw: &str) -> Result<String> {
        if raw.trim().is_empty() {
            return Err(anyhow!("hotkey vazia"));
        }

        let parts: Vec<String> = raw
            .split('+')
            .map(|p| p.trim().to_uppercase())
            .filter(|p| !p.is_empty())
            .collect();

        if parts.is_empty() {
            return Err(anyhow!("hotkey invalida: {}", raw));
        }

        let mut modifiers = Vec::new();
        let mut key = String::new();

        for part in &parts {
            match part.as_str() {
                "CTRL" | "CONTROL" => modifiers.push("Ctrl"),
                "ALT" => modifiers.push("Alt"),
                "SHIFT" => modifiers.push("Shift"),
                "SUPER" | "WIN" | "META" => modifiers.push("Super"),
                _ if key.is_empty() => {
                    key = part.clone();
                }
                _ => {
                    return Err(anyhow!("modificador invalido: {} em '{}'", part, raw));
                }
            }
        }

        if key.is_empty() {
            return Err(anyhow!("hotkey deve conter uma tecla: {}", raw));
        }

        Self::validate_key(&key)?;

        modifiers.sort();
        modifiers.dedup();

        let result = if modifiers.is_empty() {
            key
        } else {
            format!("{}+{}", modifiers.join("+"), key)
        };

        Ok(result)
    }

    fn validate_key(key: &str) -> Result<()> {
        match key {
            "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9" | "F10" | "F11"
            | "F12" => Ok(()),

            k if k.len() == 1 && k.chars().all(|c| c.is_alphanumeric()) => Ok(()),
            k if k.len() == 1 && k.chars().all(|c| c.is_ascii_digit()) => Ok(()),

            "ENTER" | "BACKSPACE" | "SPACE" | "TAB" | "CAPS" | "ESCAPE" | "NUMLOCK"
            | "SCROLLLOCK" | "PAUSE" | "PRINTSCREEN" | "INSERT" | "DELETE" | "HOME" | "END"
            | "PAGEUP" | "PAGEDOWN" | "UP" | "DOWN" | "LEFT" | "RIGHT" => Ok(()),

            k if k.starts_with("NUMPAD") => Ok(()),

            _ => Err(anyhow!("key invalida: {}", key)),
        }
    }

    pub fn normalized(&self) -> &str {
        &self.normalized
    }

    pub fn as_str(&self) -> &str {
        &self.original
    }

    pub fn conflict_with(&self, other: &Hotkey) -> bool {
        self.normalized == other.normalized
    }
}

impl PartialEq for Hotkey {
    fn eq(&self, other: &Self) -> bool {
        self.normalized == other.normalized
    }
}

impl Eq for Hotkey {}

impl Hash for Hotkey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.normalized.hash(state)
    }
}

impl std::fmt::Display for Hotkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.original)
    }
}

// ─────── TESTES ───────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_hotkey() {
        let hk = Hotkey::new("Ctrl+Alt+A").unwrap();
        assert_eq!(hk.as_str(), "Ctrl+Alt+A");
        assert_eq!(hk.normalized(), "Alt+Ctrl+A");
    }

    #[test]
    fn normalize_reorders_modifiers() {
        let hk1 = Hotkey::new("Ctrl+Shift+F1").unwrap();
        let hk2 = Hotkey::new("Shift+Ctrl+F1").unwrap();
        let hk3 = Hotkey::new("F1+Shift+Ctrl").unwrap();

        assert_eq!(hk1, hk2);
        assert_eq!(hk2, hk3);
        assert!(hk1.conflict_with(&hk2));
    }

    #[test]
    fn case_insensitive() {
        let hk1 = Hotkey::new("ctrl+alt+a").unwrap();
        let hk2 = Hotkey::new("CTRL+ALT+A").unwrap();
        let hk3 = Hotkey::new("Ctrl+Alt+A").unwrap();

        assert_eq!(hk1, hk2);
        assert_eq!(hk2, hk3);
    }

    #[test]
    fn function_keys() {
        for i in 1..=12 {
            let hk = Hotkey::new(&format!("F{}", i)).unwrap();
            assert!(hk.normalized().starts_with("F"));
        }
    }

    #[test]
    fn reject_invalid_key() {
        assert!(Hotkey::new("Ctrl+InvalidKey").is_err());
    }

    #[test]
    fn reject_empty() {
        assert!(Hotkey::new("").is_err());
        assert!(Hotkey::new("   ").is_err());
    }

    #[test]
    fn reject_only_modifiers() {
        assert!(Hotkey::new("Ctrl+Alt+Shift").is_err());
    }

    #[test]
    fn whitespace_tolerance() {
        let hk1 = Hotkey::new("Ctrl + Alt + A").unwrap();
        let hk2 = Hotkey::new("Ctrl+Alt+A").unwrap();
        assert_eq!(hk1, hk2);
    }

    #[test]
    fn dedup_modifiers() {
        // Ctrl+Ctrl+Alt+A → Alt+Ctrl+A
        let hk = Hotkey::new("Ctrl+Ctrl+Alt+A").unwrap();
        assert_eq!(hk.normalized(), "Alt+Ctrl+A");
    }

    #[test]
    fn use_in_hashset() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let hk1 = Hotkey::new("Ctrl+Alt+A").unwrap();
        let hk2 = Hotkey::new("Alt+Ctrl+A").unwrap();

        set.insert(hk1);
        // hk2 é igual a hk1, então não será inserida
        assert_eq!(set.insert(hk2), false);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn display_original() {
        let hk = Hotkey::new("Ctrl+Alt+A").unwrap();
        assert_eq!(format!("{}", hk), "Ctrl+Alt+A");
    }
}
