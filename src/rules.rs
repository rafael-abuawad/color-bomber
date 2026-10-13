//! Pure play rules. No Bevy types, so the awkward cases stay unit-testable.

/// Buttons and bombs. Order matches sprite arrays and the A / S / D keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BombColor {
    Purple = 0,
    Orange = 1,
    Green = 2,
}

impl BombColor {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_roll(n: u32) -> Self {
        match n % 3 {
            0 => Self::Purple,
            1 => Self::Orange,
            _ => Self::Green,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AirBomb {
    pub id: u64,
    pub color: BombColor,
    pub y: f32,
}

/// Lowest bomb of `color` (smallest y). Equal heights break toward the smaller id.
pub fn lowest_bomb(bombs: &[AirBomb], color: BombColor) -> Option<u64> {
    bombs
        .iter()
        .filter(|bomb| bomb.color == color)
        .min_by(|a, b| a.y.total_cmp(&b.y).then(a.id.cmp(&b.id)))
        .map(|bomb| bomb.id)
}

#[derive(Clone, Copy, Debug)]
pub struct BuildingSlot {
    pub x: f32,
    pub width: f32,
    pub phase: f32,
    pub bob_speed: f32,
}

/// Left to right: green, white center tower, purple, orange.
pub const BUILDINGS: [BuildingSlot; 4] = [
    BuildingSlot {
        x: -255.0,
        width: 155.0,
        phase: 0.4,
        bob_speed: 1.05,
    },
    BuildingSlot {
        x: -70.0,
        width: 198.0,
        phase: 1.8,
        bob_speed: 0.82,
    },
    BuildingSlot {
        x: 115.0,
        width: 155.0,
        phase: 3.2,
        bob_speed: 1.16,
    },
    BuildingSlot {
        x: 270.0,
        width: 160.0,
        phase: 4.6,
        bob_speed: 0.94,
    },
];

/// Cropped sprite height / width, same order as [`BUILDINGS`].
pub const BUILDING_ASPECT: [f32; 4] = [
    1526.0 / 1059.0,
    1944.0 / 1091.0,
    1457.0 / 958.0,
    1502.0 / 989.0,
];

pub const BUILDING_BASELINE_Y: f32 = -455.0;

/// A miss lands when the bomb's bottom crosses this line, just above the tallest roof.
pub const SKYLINE_Y: f32 = -80.0;

pub const VIEW_W: f32 = 720.0;
pub const VIEW_H: f32 = 1080.0;
pub const BOMB_WIDTH: f32 = 68.0;
pub const BOMB_HEIGHT: f32 = 68.0 * (673.0 / 402.0);
pub const SPAWN_Y: f32 = VIEW_H * 0.5 + BOMB_HEIGHT;
pub const SPAWN_X_MIN: f32 = -300.0;
pub const SPAWN_X_MAX: f32 = 300.0;
pub const MAX_BOMBS: usize = 7;
pub const START_LIVES: u32 = 5;
pub const BOB_AMPLITUDE: f32 = 7.0;

pub const BUTTON_X: [f32; 3] = [-220.0, 8.0, 224.0];
pub const BUTTON_Y: f32 = -438.0;
pub const BUTTON_W: f32 = 168.0;
pub const BUTTON_H: f32 = BUTTON_W * (681.0 / 675.0);

/// Building a falling bomb at `x` will damage. Ties go to the left-hand building.
pub fn building_hit_by(x: f32) -> usize {
    let mut best = 0;
    let mut best_dist = f32::MAX;
    for (index, slot) in BUILDINGS.iter().enumerate() {
        let dist = (x - slot.x).abs();
        if dist < best_dist {
            best = index;
            best_dist = dist;
        }
    }
    best
}

pub fn building_height(index: usize) -> f32 {
    BUILDINGS[index].width * BUILDING_ASPECT[index]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageStage {
    Intact,
    Scorched,
    OnFire,
    Ruined,
}

impl DamageStage {
    pub fn hit(self) -> Self {
        match self {
            Self::Intact => Self::Scorched,
            Self::Scorched => Self::OnFire,
            Self::OnFire | Self::Ruined => Self::Ruined,
        }
    }

    /// Ruined towers barely drift.
    pub fn bob_scale(self) -> f32 {
        match self {
            Self::Ruined => 0.2,
            _ => 1.0,
        }
    }

    pub fn sink(self) -> f32 {
        match self {
            Self::Ruined => 46.0,
            _ => 0.0,
        }
    }

    pub fn emits_smoke(self) -> bool {
        matches!(self, Self::Scorched)
    }

    pub fn emits_flame(self) -> bool {
        matches!(self, Self::OnFire | Self::Ruined)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Difficulty {
    pub fall_speed: f32,
    pub spawn_gap: f32,
}

/// Speed rises and the gap shrinks until score 48, then both hold.
pub fn difficulty_for_score(score: u32) -> Difficulty {
    let t = (score as f32 / 48.0).clamp(0.0, 1.0);
    Difficulty {
        fall_speed: 130.0 + t * 280.0,
        spawn_gap: 1.45 - t * 0.95,
    }
}

#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x as u32
    }

    pub fn f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / ((1u32 << 24) as f32)
    }

    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.f32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowest_bomb_picks_the_closest_of_that_color() {
        let bombs = [
            AirBomb {
                id: 1,
                color: BombColor::Purple,
                y: 100.0,
            },
            AirBomb {
                id: 2,
                color: BombColor::Orange,
                y: 10.0,
            },
            AirBomb {
                id: 3,
                color: BombColor::Purple,
                y: 40.0,
            },
            AirBomb {
                id: 4,
                color: BombColor::Purple,
                y: 40.0,
            },
        ];
        assert_eq!(lowest_bomb(&bombs, BombColor::Purple), Some(3));
        assert_eq!(lowest_bomb(&bombs, BombColor::Orange), Some(2));
        assert_eq!(lowest_bomb(&bombs, BombColor::Green), None);
        assert_eq!(lowest_bomb(&[], BombColor::Green), None);
    }

    #[test]
    fn impact_hits_the_nearest_building() {
        assert_eq!(building_hit_by(-400.0), 0);
        assert_eq!(building_hit_by(BUILDINGS[2].x), 2);
        assert_eq!(building_hit_by(900.0), 3);
        let between_center_and_purple = (BUILDINGS[1].x + BUILDINGS[2].x) * 0.5;
        assert_eq!(building_hit_by(between_center_and_purple), 1);
    }

    #[test]
    fn buildings_sit_left_to_right() {
        for pair in BUILDINGS.windows(2) {
            assert!(pair[0].x < pair[1].x);
        }
    }

    #[test]
    fn skyline_is_above_the_tallest_roof() {
        let roof = BUILDING_BASELINE_Y + building_height(1);
        assert!(
            SKYLINE_Y > roof,
            "skyline {SKYLINE_Y} should clear the center roof {roof}"
        );
    }

    #[test]
    fn damage_advances_then_sticks_on_ruined() {
        let mut stage = DamageStage::Intact;
        for expected in [
            DamageStage::Scorched,
            DamageStage::OnFire,
            DamageStage::Ruined,
            DamageStage::Ruined,
        ] {
            stage = stage.hit();
            assert_eq!(stage, expected);
        }
        assert!(DamageStage::Intact.bob_scale() > DamageStage::Ruined.bob_scale());
        assert_eq!(DamageStage::Intact.sink(), 0.0);
        assert!(DamageStage::Ruined.sink() > 0.0);
        assert!(DamageStage::Scorched.emits_smoke());
        assert!(DamageStage::OnFire.emits_flame());
        assert!(DamageStage::Ruined.emits_flame());
        assert!(!DamageStage::Intact.emits_flame());
    }

    #[test]
    fn difficulty_ramps_then_holds() {
        let start = difficulty_for_score(0);
        let mid = difficulty_for_score(24);
        let cap = difficulty_for_score(48);
        let beyond = difficulty_for_score(500);
        assert!(start.fall_speed < mid.fall_speed && mid.fall_speed < cap.fall_speed);
        assert!(start.spawn_gap > mid.spawn_gap && mid.spawn_gap > cap.spawn_gap);
        assert_eq!(cap, beyond);
        assert!(cap.spawn_gap >= 0.42);
        assert!(cap.fall_speed <= 410.0);
        assert!(start.fall_speed >= 130.0);
    }

    #[test]
    fn rng_range_stays_inside_the_interval() {
        let mut rng = Rng::new(0xC0_10_B0);
        for _ in 0..64 {
            let n = rng.range(-300.0, 300.0);
            assert!((-300.0..300.0).contains(&n), "{n}");
        }
        let color = BombColor::from_roll(rng.next_u32());
        assert!(matches!(
            color,
            BombColor::Purple | BombColor::Orange | BombColor::Green
        ));
    }
}
