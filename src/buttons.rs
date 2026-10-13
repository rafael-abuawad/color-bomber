use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::rules::{BombColor, BUTTON_H, BUTTON_W, BUTTON_X, BUTTON_Y};

#[derive(Message, Debug, Clone, Copy)]
pub struct ColorPressed(pub BombColor);

#[derive(Component)]
pub struct ColorButton {
    pub color: BombColor,
    pub pressed: f32,
}

pub fn spawn_buttons(commands: &mut Commands, assets: &GameAssets) {
    for (index, color) in [BombColor::Purple, BombColor::Orange, BombColor::Green]
        .into_iter()
        .enumerate()
    {
        commands
            .spawn((
                Sprite {
                    image: assets.buttons[index].clone(),
                    custom_size: Some(Vec2::new(BUTTON_W, BUTTON_H)),
                    ..default()
                },
                Transform::from_xyz(BUTTON_X[index], BUTTON_Y, 5.0),
                Pickable::default(),
                ColorButton {
                    color,
                    pressed: 0.0,
                },
            ))
            .observe(on_button_press);
    }
}

fn on_button_press(
    press: On<PointerPress>,
    buttons: Query<&ColorButton>,
    mut writer: MessageWriter<ColorPressed>,
) {
    let Ok(button) = buttons.get(press.entity) else {
        return;
    };
    writer.write(ColorPressed(button.color));
}

pub fn keyboard_input(keys: Res<ButtonInput<KeyCode>>, mut writer: MessageWriter<ColorPressed>) {
    if keys.just_pressed(KeyCode::KeyA) {
        writer.write(ColorPressed(BombColor::Purple));
    }
    if keys.just_pressed(KeyCode::KeyS) {
        writer.write(ColorPressed(BombColor::Orange));
    }
    if keys.just_pressed(KeyCode::KeyD) {
        writer.write(ColorPressed(BombColor::Green));
    }
}

pub fn animate_buttons(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut buttons: Query<(&mut ColorButton, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (mut button, mut sprite, mut transform) in &mut buttons {
        if button.pressed > 0.0 {
            button.pressed = (button.pressed - dt).max(0.0);
            sprite.image = assets.buttons_pressed[button.color.index()].clone();
            let k = (button.pressed / 0.14).clamp(0.0, 1.0);
            transform.scale = Vec3::new(1.0 - 0.05 * k, 1.0 - 0.14 * k, 1.0);
        } else {
            sprite.image = assets.buttons[button.color.index()].clone();
            transform.scale = Vec3::ONE;
        }
    }
}
