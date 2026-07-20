/// Regra de ducking: calcula o volume efetivo do Bgm dado seu volume base
/// e quantos SFX estão tocando agora. Pura, sem I/O — fácil de testar isolada.
#[derive(Debug, Clone)]
pub struct DuckingPolicy {
    /// Proporção do volume base mantida durante ducking (ex: 0.25 = cai pra 25%).
    /// Proporcional, não valor fixo, para acompanhar mudanças no volume base ao vivo.
    pub duck_ratio: f32,
    pub attack_ms: u32,
    pub release_ms: u32,
}

impl Default for DuckingPolicy {
    fn default() -> Self {
        Self {
            duck_ratio: 0.25,
            attack_ms: 100,
            release_ms: 900,
        }
    }
}

impl DuckingPolicy {
    pub fn target_volume(&self, bgm_base_volume: f32, active_sfx_count: u32) -> f32 {
        if active_sfx_count > 0 {
            bgm_base_volume * self.duck_ratio
        } else {
            bgm_base_volume
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duck_reduces_proportionally() {
        let policy = DuckingPolicy::default();
        assert_eq!(policy.target_volume(0.30, 1), 0.075);
    }

    #[test]
    fn no_sfx_keeps_base_volume() {
        let policy = DuckingPolicy::default();
        assert_eq!(policy.target_volume(0.30, 0), 0.30);
    }

    #[test]
    fn multiple_sfx_still_ducked_once() {
        let policy = DuckingPolicy::default();
        assert_eq!(policy.target_volume(0.30, 3), policy.target_volume(0.30, 1));
    }
}
