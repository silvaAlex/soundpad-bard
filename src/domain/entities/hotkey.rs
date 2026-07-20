/// Value object que representa uma combinação de teclas.
/// Formato normalizado tipo "CTRL+SHIFT+F1" — a normalização (ordem dos
/// modificadores, uppercase) é o que permite comparar duas hotkeys por
/// igualdade simples no HotkeyConflictChecker, sem parsing repetido.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Hotkey {
    normalized: String,
}

impl Hotkey {
    pub fn new(raw: &str) -> Self {
        Self { normalized: raw.to_string() }
    }

    pub fn parse(raw: &str) -> Result<Self, String> {
        let mut parts: Vec<String> = raw
            .split('+')
            .map(|p| p.trim().to_uppercase())
            .filter(|p| !p.is_empty())
            .collect();

        if parts.is_empty() {
            return Err("hotkey vazia".into());
        }

        // última parte é a tecla principal, o resto são modificadores — ordena
        // os modificadores para que "CTRL+SHIFT+F1" == "SHIFT+CTRL+F1"
        let key = parts.pop().unwrap();
        parts.sort();
        parts.push(key);

        Ok(Self {
            normalized: parts.join("+"),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.normalized
    }
}
