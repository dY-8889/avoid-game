use bevy::prelude::*;

use crate::{PLAYER_MOVE_LIMIT_LEFT, PLAYER_MOVE_LIMIT_RIGHT, PLAYER_SPEED};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerDate>()
            .add_systems(Update, move_player);
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Resource, Debug)]
pub struct PlayerDate {
    pub hp: i32,
    pub condition: Vec<Condition>,
}

impl Default for PlayerDate {
    fn default() -> Self {
        PlayerDate {
            hp: 100,
            condition: Vec::new(),
        }
    }
}

// プレイヤーの状態
// 使わないかも
#[derive(Default, Debug)]
pub enum Condition {
    #[default]
    Normal,
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut player_transform = query.single_mut();

    let x = player_transform.translation.x;

    if keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) && x < PLAYER_MOVE_LIMIT_RIGHT {
        player_transform.translation.x += PLAYER_SPEED;
    }
    if keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) && x > PLAYER_MOVE_LIMIT_LEFT {
        player_transform.translation.x += -PLAYER_SPEED;
    }
}
