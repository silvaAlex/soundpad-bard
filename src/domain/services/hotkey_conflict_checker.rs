use crate::domain::entities::Hotkey;
use std::collections::HashSet;

/// Registro único de hotkeys — soundpad e controles do Bard checam contra
/// o mesmo conjunto, então não há como colidir entre os dois modos.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct HotkeyConflictChecker {
    registered: HashSet<Hotkey>,
}

#[allow(dead_code)]
impl HotkeyConflictChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_available(&self, hotkey: &Hotkey) -> bool {
        !self.registered.contains(hotkey)
    }

    pub fn register(&mut self, hotkey: Hotkey) -> Result<(), String> {
        if !self.is_available(&hotkey) {
            return Err(format!("hotkey '{}' já está em uso", hotkey.as_str()));
        }
        self.registered.insert(hotkey);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate() {
        let mut checker = HotkeyConflictChecker::new();
        let hk = Hotkey::parse("CTRL+F1").unwrap();
        assert!(checker.register(hk.clone()).is_ok());
        assert!(checker.register(hk).is_err());
    }

    #[test]
    fn normalized_order_still_conflicts() {
        let mut checker = HotkeyConflictChecker::new();
        checker.register(Hotkey::parse("CTRL+SHIFT+F1").unwrap()).unwrap();
        let dup = Hotkey::parse("SHIFT+CTRL+F1").unwrap();
        assert!(!checker.is_available(&dup));
    }
}
