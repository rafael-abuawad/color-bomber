use std::time::Duration;
// std::time::SystemTime::now panics on wasm32-unknown-unknown. web-time uses
// Date.now() in the browser and the normal clock everywhere else.
use web_time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::buttons::{ColorButton, ColorPressed};
use crate::city::{stage_tint, Building};
use crate::rules::{
    building_hit_by, difficulty_for_score, lowest_bomb, AirBomb, BombColor, Rng,
    BOMB_HEIGHT, BOMB_WIDTH, MAX_BOMBS, SKYLINE_Y, SPAWN_X_MAX, SPAWN_X_MIN, SPAWN_Y, START_LIVES,
};
use crate::vfx::{self, Particle};
use crate::GameState;

#[derive(Component)]
pub struct Bomb {
    id: u64,
    color: BombColor,
}

#[derive(Resource)]
pub struct Session {
    pub score: u32,
    pub lives: u32,
    pub spawn: Timer,
    pub next_id: u64,
    pub rng: Rng,
}

impl Session {
    pub fn fresh(seed: u64) -> Self {
        let gap = difficulty_for_score(0).spawn_gap;
        let mut spawn = Timer::from_seconds(gap, TimerMode::Repeating);
        // First bomb shows up shortly after the run starts, not after a full gap.
        let warmup = (gap - 0.45).max(0.0);
        spawn.tick(Duration::from_secs_f32(warmup));
        Self {
            score: 0,
            lives: START_LIVES,
            spawn,
            next_id: 1,
            rng: Rng::new(seed),
        }
    }
}

pub fn spawn_bombs(
    time: Res<Time>,
    state: Res<State<GameState>>,
    assets: Res<GameAssets>,
    mut commands: Commands,
    mut session: ResMut<Session>,
    bombs: Query<(), With<Bomb>>,
) {
    if *state.get() != GameState::Playing {
        return;
    }
    session.spawn.tick(time.delta());
    if !session.spawn.just_finished() || bombs.iter().count() >= MAX_BOMBS {
        return;
    }
    let gap = difficulty_for_score(session.score).spawn_gap;
    session.spawn.set_duration(Duration::from_secs_f32(gap));
    spawn_bomb(&mut commands, &assets, &mut session);
}

fn spawn_bomb(commands: &mut Commands, assets: &GameAssets, session: &mut Session) {
    let color = BombColor::from_roll(session.rng.next_u32());
    let x = session.rng.range(SPAWN_X_MIN, SPAWN_X_MAX);
    let id = session.next_id;
    session.next_id += 1;
    commands.spawn((
        Sprite {
            image: assets.bombs[color.index()].clone(),
            custom_size: Some(Vec2::new(BOMB_WIDTH, BOMB_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(x, SPAWN_Y, 6.0),
        Bomb { id, color },
    ));
}

pub fn fall_bombs(
    time: Res<Time>,
    state: Res<State<GameState>>,
    assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut session: ResMut<Session>,
    mut bombs: Query<(Entity, &Bomb, &mut Transform)>,
    mut buildings: Query<(&mut Building, &mut Sprite)>,
) {
    if *state.get() != GameState::Playing {
        return;
    }
    let dt = time.delta_secs();
    let speed = difficulty_for_score(session.score).fall_speed;
    for (entity, _, mut transform) in &mut bombs {
        transform.translation.y -= speed * dt;
        if transform.translation.y - BOMB_HEIGHT * 0.5 > SKYLINE_Y {
            continue;
        }
        let x = transform.translation.x;
        let y = transform.translation.y;
        commands.entity(entity).despawn();
        miss(
            &mut commands,
            &assets,
            &mut session,
            &mut buildings,
            &mut next_state,
            x,
            y,
        );
    }
}

fn miss(
    commands: &mut Commands,
    assets: &GameAssets,
    session: &mut Session,
    buildings: &mut Query<(&mut Building, &mut Sprite)>,
    next_state: &mut NextState<GameState>,
    x: f32,
    y: f32,
) {
    vfx::spawn_burst(
        commands,
        assets,
        &mut session.rng,
        Vec3::new(x, y, 8.0),
        Color::srgb(0.75, 0.75, 0.78),
    );
    if session.lives == 0 {
        return;
    }
    let index = building_hit_by(x);
    for (mut building, mut sprite) in buildings.iter_mut() {
        if building.index != index {
            continue;
        }
        building.stage = building.stage.hit();
        sprite.color = stage_tint(building.stage);
        info!(
            "building {} hit, now {:?}",
            index, building.stage
        );
    }
    session.lives = session.lives.saturating_sub(1);
    if session.lives == 0 {
        next_state.set(GameState::GameOver);
    }
}

pub fn resolve_presses(
    mut presses: MessageReader<ColorPressed>,
    state: Res<State<GameState>>,
    assets: Res<GameAssets>,
    mut commands: Commands,
    mut session: ResMut<Session>,
    mut buttons: Query<&mut ColorButton>,
    bombs: Query<(Entity, &Bomb, &Transform)>,
) {
    let playing = *state.get() == GameState::Playing;
    let mut gone = Vec::new();
    for press in presses.read() {
        for mut button in &mut buttons {
            if button.color == press.0 {
                button.pressed = 0.14;
            }
        }
        if !playing {
            continue;
        }
        let air: Vec<AirBomb> = bombs
            .iter()
            .filter(|(_, bomb, _)| !gone.contains(&bomb.id))
            .map(|(_, bomb, transform)| AirBomb {
                id: bomb.id,
                color: bomb.color,
                y: transform.translation.y,
            })
            .collect();
        let Some(id) = lowest_bomb(&air, press.0) else {
            continue;
        };
        let Some((entity, _, transform)) = bombs.iter().find(|(_, bomb, _)| bomb.id == id) else {
            continue;
        };
        let origin = transform.translation;
        let tint = vfx::color_tint(press.0);
        gone.push(id);
        commands.entity(entity).despawn();
        session.score += 1;
        vfx::spawn_burst(&mut commands, &assets, &mut session.rng, origin, tint);
    }
}

pub fn begin_run(
    mut commands: Commands,
    mut session: ResMut<Session>,
    mut buildings: Query<(&mut Building, &mut Sprite)>,
    bombs: Query<Entity, With<Bomb>>,
    particles: Query<Entity, With<Particle>>,
) {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    *session = Session::fresh(seed);
    for (mut building, mut sprite) in &mut buildings {
        building.restore();
        sprite.color = Color::WHITE;
    }
    for entity in &bombs {
        commands.entity(entity).despawn();
    }
    for entity in &particles {
        commands.entity(entity).despawn();
    }
    info!("run started lives={}", session.lives);
}

pub fn end_run(session: Res<Session>, mut commands: Commands, bombs: Query<Entity, With<Bomb>>) {
    info!("game over score={}", session.score);
    for entity in &bombs {
        commands.entity(entity).despawn();
    }
}
