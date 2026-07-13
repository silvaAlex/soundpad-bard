/// Estado do Bard modelado explicitamente — ducking é uma transição de
/// estado, não uma flag de volume solta. Isso evita o tipo de bug de
/// transição implícita (ex: um "stop" que deveria só desducar mas
/// acaba levando pra Idle).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BardState {
    Idle,
    Playing,
    /// Tocando mas com volume reduzido porque há SFX ativo(s) no outro canal.
    Ducked,
    Paused,
}

impl BardState {
    #[allow(dead_code)]
    pub fn can_transition_to(&self, next: BardState) -> bool {
        use BardState::*;
        matches!(
            (self, next),
            (Idle, Playing)
                | (Playing, Ducked)
                | (Playing, Paused)
                | (Playing, Idle)
                | (Ducked, Playing)
                | (Ducked, Paused)
                | (Ducked, Idle)
                | (Paused, Playing)
                | (Paused, Idle)
        )
    }
}
