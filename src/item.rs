use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use rand::{thread_rng, Rng};

use self::ItemType::*;
use crate::{player::PlayerDate, random_translation, Collider, Images};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<ItemType>()
            .add_systems(
                Update,
                create_item.run_if(on_timer(Duration::from_secs_f32(3.0))),
            )
            .add_systems(OnEnter(Portion), portion);
    }
}

#[derive(Component)]
pub struct Item(pub ItemType);

#[derive(Bundle)]
struct ItemBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    item: Item,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, States)]
pub enum ItemType {
    Portion,
    SpeedUp,
    Big,
    #[default]
    Null,
}

impl ItemBundle {
    fn new(item_type: ItemType, texture: Handle<Image>) -> ItemBundle {
        ItemBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: random_translation().extend(0.0),
                    scale: item_type.scale().extend(0.0),
                    ..default()
                },
                texture,
                sprite: Sprite {
                    custom_size: Some(Vec2::new(2.5, 2.5)),
                    ..default()
                },
                ..default()
            },
            collider: Collider,
            item: Item(item_type),
        }
    }
}

impl ItemType {
    pub fn speed(&self) -> f32 {
        match self {
            Portion => 7.0,
            SpeedUp => 10.0,
            Big => 5.0,
            Null => panic!(),
        }
    }
    // 攻撃の大きさ
    fn scale(&self) -> Vec2 {
        match self {
            Portion => Vec2::new(25., 25.),
            SpeedUp => Vec2::new(30., 30.),
            Big => Vec2::new(45.0, 45.0),
            Null => panic!(),
        }
    }

    fn random_item() -> Self {
        match thread_rng().gen_range(0..=1) {
            0 => Portion,
            1 => Big,
            _ => SpeedUp,
        }
    }

    pub fn sound_key(&self) -> String {
        let key = match self {
            Portion => "recovery",
            SpeedUp => "powerup",
            Big => "big",
            Null => panic!(),
        };
        key.to_string()
    }
    fn image_key(&self) -> String {
        let key = match self {
            Portion => "portion",
            SpeedUp => "powerup",
            Big => "",
            Null => panic!(),
        };
        key.to_string()
    }
}

// アイテムを作る
fn create_item(mut commands: Commands, image: Res<Images>) {
    let item = ItemType::random_item();
    let texture = image.item.get(&item.image_key());

    if let Some(texture) = texture {
        commands.spawn(ItemBundle::new(item, texture.clone()));
    } else {
        error!("{:?}のimage_keyが存在しません", item);
    }
}

fn portion(mut player_date: ResMut<PlayerDate>, mut state: ResMut<NextState<ItemType>>) {
    player_date.hp += 10;

    if player_date.hp > 100 {
        player_date.hp = 100
    }

    state.set(Null);
}
