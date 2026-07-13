pub mod activate_bard;
pub mod pause_bard;
pub mod play_sound_effect;
pub mod register_hotkey;
pub mod skip_bard_track;

#[allow(unused_imports)]
pub use activate_bard::ActivateBard;
#[allow(unused_imports)]
pub use pause_bard::PauseBard;
#[allow(unused_imports)]
pub use play_sound_effect::PlaySoundEffect;
#[allow(unused_imports)]
pub use register_hotkey::RegisterHotkey;
#[allow(unused_imports)]
pub use skip_bard_track::SkipBardTrack;

use crate::application::ports::{ConfigRepository, ObsConnector};

#[allow(dead_code)]
pub async fn play_sound<O: ObsConnector, C: ConfigRepository>(
    obs: &mut O,
    config: &C,
    clip_id: &str,
) -> anyhow::Result<()> {
    PlaySoundEffect { obs, config }.execute(clip_id).await
}

#[allow(dead_code)]
pub async fn activate_bard<O: ObsConnector, C: ConfigRepository>(
    obs: &mut O,
    config: &C,
    music_file: &str,
) -> anyhow::Result<()> {
    ActivateBard { obs, config }
        .execute(music_file)
        .await
}

#[allow(dead_code)]
pub async fn pause_bard<O: ObsConnector, C: ConfigRepository>(
    obs: &mut O,
    config: &C,
) -> anyhow::Result<()> {
    PauseBard { obs, config }.execute().await
}

#[allow(dead_code)]
pub async fn skip_bard<O: ObsConnector, C: ConfigRepository>(
    obs: &mut O,
    config: &C,
    next_file: &str,
) -> anyhow::Result<()> {
    SkipBardTrack { obs, config }
        .execute(next_file)
        .await
}
