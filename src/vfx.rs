use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::rules::{BombColor, Rng};

#[derive(Clone, Copy, Debug)]
enum ParticleKind {
    Hex,
    Smoke,
    Flame { frame: u8, frame_t: f32 },
}

#[derive(Component)]
pub struct Particle {
    velocity: Vec2,
    life: f32,
    max_life: f32,
    tint: Color,
    spin: f32,
    grow: f32,
    drag: f32,
    kind: ParticleKind,
}

pub fn color_tint(color: BombColor) -> Color {
    match color {
        BombColor::Purple => Color::srgb(0.576, 0.396, 0.722),
        BombColor::Orange => Color::srgb(0.984, 0.627, 0.149),
        BombColor::Green => Color::srgb(0.380, 0.741, 0.427),
    }
}

pub fn spawn_burst(
    commands: &mut Commands,
    assets: &GameAssets,
    rng: &mut Rng,
    origin: Vec3,
    tint: Color,
) {
    let count = 10;
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU + rng.range(-0.25, 0.25);
        let speed = rng.range(80.0, 170.0);
        spawn_particle(
            commands,
            assets.hex.clone(),
            origin,
            Vec2::new(angle.cos(), angle.sin()) * speed,
            rng.range(12.0, 20.0),
            rng.range(0.32, 0.5),
            tint,
            rng.range(-4.0, 4.0),
            0.8,
            2.2,
            ParticleKind::Hex,
        );
    }
}

pub fn spawn_smoke(commands: &mut Commands, assets: &GameAssets, rng: &mut Rng, origin: Vec3) {
    spawn_particle(
        commands,
        assets.hex.clone(),
        origin,
        Vec2::new(rng.range(-18.0, 18.0), rng.range(28.0, 70.0)),
        rng.range(16.0, 28.0),
        rng.range(0.7, 1.1),
        Color::srgb(0.62, 0.64, 0.68),
        rng.range(-1.0, 1.0),
        0.6,
        0.4,
        ParticleKind::Smoke,
    );
}

pub fn spawn_flame(commands: &mut Commands, assets: &GameAssets, rng: &mut Rng, origin: Vec3) {
    let frame = (rng.next_u32() % 3) as u8;
    spawn_particle(
        commands,
        assets.flames[frame as usize].clone(),
        origin,
        Vec2::new(rng.range(-22.0, 22.0), rng.range(40.0, 110.0)),
        rng.range(22.0, 40.0),
        rng.range(0.35, 0.6),
        Color::WHITE,
        rng.range(-1.5, 1.5),
        0.15,
        0.35,
        ParticleKind::Flame {
            frame,
            frame_t: 0.0,
        },
    );
}

fn spawn_particle(
    commands: &mut Commands,
    image: Handle<Image>,
    origin: Vec3,
    velocity: Vec2,
    size: f32,
    life: f32,
    tint: Color,
    spin: f32,
    grow: f32,
    drag: f32,
    kind: ParticleKind,
) {
    commands.spawn((
        Sprite {
            image,
            custom_size: Some(Vec2::splat(size)),
            color: tint,
            ..default()
        },
        Transform::from_translation(origin.with_z(8.0)),
        Particle {
            velocity,
            life,
            max_life: life,
            tint,
            spin,
            grow,
            drag,
            kind,
        },
    ));
}

pub fn update_particles(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Particle, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut sprite, mut transform) in &mut particles {
        particle.life -= dt;
        if particle.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let keep = (1.0 - particle.drag * dt).max(0.0);
        particle.velocity *= keep;
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;
        let age = 1.0 - (particle.life / particle.max_life);
        let scale = 1.0 + age * particle.grow;
        transform.scale = Vec3::splat(scale);
        transform.rotation = Quat::from_rotation_z(particle.spin * age);
        sprite.color = particle.tint.with_alpha(particle.life / particle.max_life);

        if let ParticleKind::Flame { frame, frame_t } = &mut particle.kind {
            *frame_t += dt;
            if *frame_t >= 0.09 {
                *frame_t = 0.0;
                *frame = (*frame + 1) % 3;
                sprite.image = assets.flames[*frame as usize].clone();
            }
        }
    }
}
