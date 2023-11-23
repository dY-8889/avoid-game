use std::time::Duration;

use bevy::prelude::*;

use rand::{thread_rng, Rng};

use self::AttackType::*;
use crate::{
    player::PlayerDate, random_translation, Collider, Images, ATTACK_CREATE_INTERVAR_TIME_RANGE,
};

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_seconds(0.5))
            .add_state::<AttackType>()
            .add_systems(FixedUpdate, create_attack)
            .add_systems(OnEnter(Bom), bom);
    }
}

#[derive(Component)]
pub struct Attack(pub AttackType);

#[derive(Bundle)]
struct AttackBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    attack: Attack,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, States)]
pub enum AttackType {
    Bom,
    #[default]
    Null,
}

impl AttackBundle {
    // 攻撃を新しく作る
    fn new(attack_type: AttackType, texture: Handle<Image>) -> AttackBundle {
        AttackBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: random_translation().extend(0.0),
                    scale: attack_type.scale().extend(0.0),
                    ..default()
                },
                texture,
                sprite: Sprite {
                    custom_size: Some(Vec2::new(3., 3.)),
                    ..default()
                },
                ..default()
            },
            collider: Collider,
            attack: Attack(attack_type),
        }
    }
}

impl AttackType {
    // 攻撃の速度
    pub fn speed(&self) -> f32 {
        match self {
            Bom => 7.0,
            Null => panic!(),
        }
    }
    // 攻撃の大きさ
    fn scale(&self) -> Vec2 {
        match self {
            Bom => Vec2::new(40., 40.),
            Null => panic!(),
        }
    }

    fn random_item() -> AttackType {
        match thread_rng().gen_range(0..1) {
            0 => Bom,
            _ => panic!(),
        }
    }

    pub fn sound_key(&self) -> String {
        let key = match self {
            Bom => "bom",
            _ => panic!(),
        };
        key.to_string()
    }
    fn image_key(&self) -> String {
        let key = match self {
            Bom => "bom",
            Null => panic!(),
        };
        key.to_string()
    }
}

// 攻撃を作る
fn create_attack(mut commands: Commands, image: Res<Images>, mut time: ResMut<Time<Fixed>>) {
    let attack = AttackType::random_item();
    let texture = image.attack.get(&attack.image_key());

    if let Some(texture) = texture {
        commands.spawn(AttackBundle::new(attack, texture.clone()));
    }

    // 攻撃を作る間隔を変更する
    let random_time = thread_rng().gen_range(ATTACK_CREATE_INTERVAR_TIME_RANGE);
    time.set_timestep(Duration::from_secs_f32(random_time));
}

fn bom(mut player_data: ResMut<PlayerDate>, mut state: ResMut<NextState<AttackType>>) {
    player_data.hp -= 10;

    state.set(Null);
}
