use crate::domain::entities::Hotkey;
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct HotkeyConflictChecker {
    registered: HashSet<Hotkey>,
}

impl HotkeyConflictChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_available(&self, hotkey: &Hotkey) -> bool {
        !self.registered.contains(hotkey)
    }

    pub fn register(&mut self, hotkey: Hotkey) -> Result<(), String> {
        if !self.is_available(&hotkey) {
            return Err(format!(
                "Hotkey '{}' já está em uso ({} registradas)",
                hotkey.normalized(),
                self.registered.len(),
            ));
        }
        self.registered.insert(hotkey);
        Ok(())
    }

    pub fn unregister(&mut self, hotkey: &Hotkey) -> bool {
        self.registered.remove(hotkey)
    }

    pub fn clear(&mut self) {
        self.registered.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let checker = HotkeyConflictChecker::new();
        assert!(checker.is_available(&Hotkey::new("F1").unwrap()));
    }

    #[test]
    fn register_single() {
        let mut checker = HotkeyConflictChecker::new();
        let hk = Hotkey::new("Ctrl+Alt+A").unwrap();

        assert!(checker.is_available(&hk));
        assert!(checker.register(hk.clone()).is_ok());
        assert!(!checker.is_available(&hk));
    }

    #[test]
    fn reject_duplicate_exact() {
        let mut checker = HotkeyConflictChecker::new();
        let hk = Hotkey::new("Ctrl+Alt+A").unwrap();

        assert!(checker.register(hk.clone()).is_ok());
        assert!(checker.register(hk).is_err());
    }

    #[test]
    fn reject_duplicate_reordered() {
        let mut checker = HotkeyConflictChecker::new();
        let hk1 = Hotkey::new("Ctrl+Shift+F1").unwrap();
        let hk2 = Hotkey::new("Shift+Ctrl+F1").unwrap();

        assert!(checker.register(hk1).is_ok());
        assert!(checker.register(hk2).is_err());
    }

    #[test]
    fn unregister_makes_available() {
        let mut checker = HotkeyConflictChecker::new();
        let hk = Hotkey::new("Ctrl+Alt+A").unwrap();

        checker.register(hk.clone()).unwrap();
        assert!(!checker.is_available(&hk));

        assert!(checker.unregister(&hk));
        assert!(checker.is_available(&hk));
        assert!(!checker.unregister(&hk));
    }

    #[test]
    fn clear_removes_all() {
        let mut checker = HotkeyConflictChecker::new();
        checker
            .register(Hotkey::new("Ctrl+Alt+A").unwrap())
            .unwrap();
        checker.register(Hotkey::new("F1").unwrap()).unwrap();

        checker.clear();
        assert!(checker.is_available(&Hotkey::new("Ctrl+Alt+A").unwrap()));
    }

    #[test]
    fn error_message_informative() {
        let mut checker = HotkeyConflictChecker::new();
        let hk = Hotkey::new("Ctrl+Alt+A").unwrap();
        checker.register(hk).unwrap();

        let err = checker
            .register(Hotkey::new("Ctrl+Alt+A").unwrap())
            .unwrap_err();
        assert!(err.contains("já está em uso"));
        assert!(err.contains("Alt+Ctrl+A"));
    }
}
