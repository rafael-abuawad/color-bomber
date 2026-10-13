use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::FontSmoothing;

use crate::assets::GameAssets;
use crate::bombs::Session;
use crate::rules::START_LIVES;
use crate::GameState;

#[derive(Component)]
pub struct LivesText;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct TitleText;

pub fn spawn_hud(commands: &mut Commands, assets: &GameAssets) {
    let font = TextFont {
        font: assets.font.clone().into(),
        font_size: FontSize::Px(64.0),
        ..default()
    }
    .with_font_smoothing(FontSmoothing::None);

    commands.spawn((
        Sprite {
            image: assets.heart.clone(),
            custom_size: Some(Vec2::new(78.0, 78.0 * (622.0 / 798.0))),
            ..default()
        },
        Transform::from_xyz(-310.0, 482.0, 20.0),
    ));

    commands.spawn((
        Text2d::new(START_LIVES.to_string()),
        font.clone(),
        TextColor(Color::srgb(0.90, 0.16, 0.18)),
        Anchor::CENTER_LEFT,
        Transform::from_xyz(-258.0, 478.0, 20.0),
        LivesText,
    ));

    commands.spawn((
        Text2d::new("0"),
        font,
        TextColor(Color::WHITE),
        Anchor::CENTER_RIGHT,
        Transform::from_xyz(332.0, 480.0, 20.0),
        ScoreText,
    ));

    let title_font = TextFont {
        font: assets.font.clone().into(),
        font_size: FontSize::Px(28.0),
        ..default()
    }
    .with_font_smoothing(FontSmoothing::None);
    commands.spawn((
        Text2d::new("TAP TO START"),
        title_font,
        TextColor(Color::WHITE),
        Anchor::CENTER,
        Transform::from_xyz(0.0, 220.0, 21.0),
        TitleText,
    ));
}

pub fn update_hud(
    session: Res<Session>,
    mut lives: Query<&mut Text2d, (With<LivesText>, Without<ScoreText>)>,
    mut score: Query<&mut Text2d, (With<ScoreText>, Without<LivesText>)>,
) {
    for mut text in &mut lives {
        text.0 = session.lives.to_string();
    }
    for mut text in &mut score {
        text.0 = session.score.to_string();
    }
}

pub fn sync_title(
    state: Res<State<GameState>>,
    mut title: Query<&mut Visibility, With<TitleText>>,
) {
    let visible = *state.get() != GameState::Playing;
    let visibility = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut title in &mut title {
        *title = visibility;
    }
}
