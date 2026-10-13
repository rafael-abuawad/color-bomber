use bevy::audio::Volume;
use bevy::prelude::*;

/// Play a one-shot effect and drop the entity when it finishes.
pub fn play(commands: &mut Commands, source: Handle<AudioSource>) {
    commands.spawn((
        AudioPlayer::new(source),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.7)),
    ));
}
