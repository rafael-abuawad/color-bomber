use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct GameAssets {
    pub bombs: [Handle<Image>; 3],
    pub buildings: [Handle<Image>; 4],
    pub buttons: [Handle<Image>; 3],
    pub buttons_pressed: [Handle<Image>; 3],
    pub heart: Handle<Image>,
    pub hex: Handle<Image>,
    pub flames: [Handle<Image>; 3],
    pub font: Handle<Font>,
}

impl GameAssets {
    pub fn load(server: &AssetServer) -> Self {
        let bomb = |name: &str| server.load(format!("sprites/bomb_{name}.png"));
        let button = |name: &str| server.load(format!("sprites/button_{name}.png"));
        let button_down = |name: &str| server.load(format!("sprites/button_{name}_pressed.png"));
        Self {
            bombs: [bomb("purple"), bomb("orange"), bomb("green")],
            buildings: [
                server.load("sprites/building_green.png"),
                server.load("sprites/building_center.png"),
                server.load("sprites/building_purple.png"),
                server.load("sprites/building_orange.png"),
            ],
            buttons: [button("purple"), button("orange"), button("green")],
            buttons_pressed: [
                button_down("purple"),
                button_down("orange"),
                button_down("green"),
            ],
            heart: server.load("sprites/heart.png"),
            hex: server.load("sprites/hex.png"),
            flames: [
                server.load("sprites/flame_0.png"),
                server.load("sprites/flame_1.png"),
                server.load("sprites/flame_2.png"),
            ],
            font: server.load("fonts/PressStart2P-Regular.ttf"),
        }
    }
}
