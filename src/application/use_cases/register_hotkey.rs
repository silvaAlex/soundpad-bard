use crate::application::ports::HotkeyListener;
use crate::domain::entities::Hotkey;
use crate::domain::services::HotkeyConflictChecker;
use anyhow::{Context, Result};

pub struct RegisterHotkey<'a> {
    pub listener: &'a mut dyn HotkeyListener,
    pub checker: &'a mut HotkeyConflictChecker,
}

impl<'a> RegisterHotkey<'a> {
    pub fn execute(&mut self, hotkey_str: &str) -> Result<u32> {
        let hotkey = Hotkey::new(hotkey_str).context("falha ao normalizar hotkey")?;

        self.checker
            .register(hotkey.clone())
            .map_err(|e| anyhow::anyhow!(e))?;

        let id = self
            .listener
            .register(&hotkey)
            .context("falha ao registrar hotkey no SO")?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::HotkeyListener;

    struct MockListener;

    impl HotkeyListener for MockListener {
        fn register(&mut self, _hotkey: &Hotkey) -> Result<u32> {
            Ok(1)
        }
        fn unregister(&mut self, _id: u32) -> Result<()> {
            Ok(())
        }
        fn poll_events(&mut self) -> Vec<u32> {
            Vec::new()
        }
    }

    #[test]
    fn register_valid_hotkey() {
        let mut listener = MockListener;
        let mut checker = HotkeyConflictChecker::new();
        let mut uc = RegisterHotkey {
            listener: &mut listener,
            checker: &mut checker,
        };
        let id = uc.execute("Ctrl+Alt+A").unwrap();
        assert_eq!(id, 1);
    }

    #[test]
    fn reject_invalid_hotkey() {
        let mut listener = MockListener;
        let mut checker = HotkeyConflictChecker::new();
        let mut uc = RegisterHotkey {
            listener: &mut listener,
            checker: &mut checker,
        };
        assert!(uc.execute("INVALID").is_err());
    }

    #[test]
    fn reject_duplicate_hotkey() {
        let mut listener = MockListener;
        let mut checker = HotkeyConflictChecker::new();
        let mut uc = RegisterHotkey {
            listener: &mut listener,
            checker: &mut checker,
        };
        uc.execute("Ctrl+Alt+A").unwrap();
        assert!(uc.execute("Ctrl+Alt+A").is_err());
    }
}
