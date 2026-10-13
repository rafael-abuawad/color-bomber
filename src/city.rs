use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::assets::GameAssets;
use crate::rules::{
    building_height, DamageStage, BUILDING_BASELINE_Y, BUILDINGS, BOB_AMPLITUDE,
};
use crate::vfx::{self, Particle};

#[derive(Component)]
pub struct Building {
    pub index: usize,
    pub stage: DamageStage,
    pub height: f32,
    phase: f32,
    bob_speed: f32,
    amplitude: f32,
    fx_wait: f32,
}

impl Building {
    pub fn restore(&mut self) {
        self.stage = DamageStage::Intact;
        self.fx_wait = self.phase;
    }
}

pub fn stage_tint(stage: DamageStage) -> Color {
    match stage {
        DamageStage::Intact => Color::WHITE,
        DamageStage::Scorched => Color::srgb(0.72, 0.68, 0.64),
        DamageStage::OnFire => Color::srgb(0.95, 0.78, 0.62),
        DamageStage::Ruined => Color::srgb(0.40, 0.36, 0.34),
    }
}

pub fn spawn_city(commands: &mut Commands, assets: &GameAssets) {
    commands.spawn((
        Sprite::from_color(
            Color::srgb(55.0 / 255.0, 60.0 / 255.0, 68.0 / 255.0),
            Vec2::new(1600.0, 200.0),
        ),
        Transform::from_xyz(0.0, -470.0, 0.0),
    ));

    // Specks taken from the star marks in background.svg.
    let stars = [
        (-237.0, 207.0, 3.0),
        (163.0, 207.0, 2.5),
        (-279.0, 349.0, 3.5),
        (24.0, 429.0, 2.5),
        (237.0, 174.0, 3.0),
        (321.0, 431.0, 2.0),
        (-120.0, 310.0, 2.5),
        (90.0, 260.0, 3.0),
        (-40.0, 380.0, 2.0),
        (200.0, 340.0, 2.5),
    ];
    for (x, y, size) in stars {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.95, 0.95, 0.97), Vec2::splat(size)),
            Transform::from_xyz(x, y, 0.2),
        ));
    }

    for (index, slot) in BUILDINGS.iter().enumerate() {
        let height = building_height(index);
        // The center tower is building1-red: white facade, red trim. building1.svg
        // is the whole skyline pasted together, so it is not a runtime sprite.
        let z = if index == 1 { 3.2 } else { 2.0 + index as f32 * 0.1 };
        commands.spawn((
            Sprite {
                image: assets.buildings[index].clone(),
                custom_size: Some(Vec2::new(slot.width, height)),
                ..default()
            },
            Anchor::BOTTOM_CENTER,
            Transform::from_xyz(slot.x, BUILDING_BASELINE_Y, z),
            Building {
                index,
                stage: DamageStage::Intact,
                height,
                phase: slot.phase,
                bob_speed: slot.bob_speed,
                amplitude: BOB_AMPLITUDE,
                fx_wait: slot.phase,
            },
        ));
    }
}

pub fn float_buildings(time: Res<Time>, mut buildings: Query<(&Building, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (building, mut transform) in &mut buildings {
        let bob = (t * building.bob_speed + building.phase).sin()
            * building.amplitude
            * building.stage.bob_scale();
        transform.translation.y = BUILDING_BASELINE_Y - building.stage.sink() + bob;
    }
}

pub fn emit_damage_fx(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut session: ResMut<crate::bombs::Session>,
    mut buildings: Query<(&mut Building, &Transform)>,
    particles: Query<Entity, With<Particle>>,
) {
    if particles.iter().count() > 90 {
        return;
    }
    let dt = time.delta_secs();
    for (mut building, transform) in &mut buildings {
        if building.stage == DamageStage::Intact {
            continue;
        }
        building.fx_wait -= dt;
        if building.fx_wait > 0.0 {
            continue;
        }
        let roof = transform.translation.y + building.height * 0.62;
        let origin = Vec3::new(
            transform.translation.x + session.rng.range(-building.height * 0.08, building.height * 0.08),
            roof,
            8.0,
        );
        if building.stage.emits_smoke() {
            vfx::spawn_smoke(&mut commands, &assets, &mut session.rng, origin);
            building.fx_wait = 0.42;
        }
        if building.stage.emits_flame() {
            vfx::spawn_flame(&mut commands, &assets, &mut session.rng, origin);
            building.fx_wait = if building.stage == DamageStage::Ruined {
                0.07
            } else {
                0.13
            };
        }
    }
}
