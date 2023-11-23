use std::collections::HashMap;
use std::fs::read_dir;
use std::ops::RangeInclusive;
use std::path::PathBuf;

use bevy::{prelude::*, sprite::collide_aabb::collide};

use rand::{thread_rng, Rng};

mod attack;
mod entity;
mod item;
mod player;

use attack::*;
use item::*;
use player::*;

// プレイヤーの初期位置
const INITIAL_PLAYER_POSITION: Vec2 = Vec2::new(0.0, -300.0);

// プレイヤーが移動できる限界
pub const PLAYER_MOVE_LIMIT_LEFT: f32 = -375.0;
pub const PLAYER_MOVE_LIMIT_RIGHT: f32 = 375.0;
// プレイヤーの速度
pub const PLAYER_SPEED: f32 = 7.5;

// entityの初期位置のY座標
const ENTITY_START_POSITION_Y: f32 = 400.0;
// Entityが作られる範囲(プレイヤーが移動できる範囲)
const ENTITY_CREATE_RANGE: RangeInclusive<f32> = PLAYER_MOVE_LIMIT_LEFT..=PLAYER_MOVE_LIMIT_RIGHT;
// 攻撃が作られる時間の間隔
const ATTACK_CREATE_INTERVAR_TIME_RANGE: RangeInclusive<f32> = 0.1..=0.3;
// アイテムが作られる間隔
const _ITEM_CREATE_INTERVAR_TIME_RANGE: RangeInclusive<f32> = 5.0..=7.0;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 720.0;

const _: () = assert!(PLAYER_MOVE_LIMIT_LEFT > -WINDOW_WIDTH / 2.);
const _: () = assert!(PLAYER_MOVE_LIMIT_RIGHT < WINDOW_WIDTH / 2.);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "避けゲー".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, (move_entity, check_for_collision))
        .add_plugins((ItemPlugin, AttackPlugin, PlayerPlugin))
        .run();
}

type AssetMap<T> = HashMap<String, Handle<T>>;

#[derive(Resource)]
struct Sounds {
    damage: AssetMap<AudioSource>,
    item: AssetMap<AudioSource>,
}

#[derive(Resource)]
pub struct Images {
    item: AssetMap<Image>,
    attack: AssetMap<Image>,
}

#[derive(Component)]
struct Collider;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut damage_sound: AssetMap<AudioSource> = HashMap::new();
    let mut item_sound: AssetMap<AudioSource> = HashMap::new();
    let mut item_image: AssetMap<Image> = HashMap::new();
    let mut attack_image: AssetMap<Image> = HashMap::new();

    get_folder("assets/audio/damage", "ogg")
        .iter()
        .for_each(|path| {
            let name = PathBuf::from(path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            damage_sound.insert(name, asset_server.load(path));
        });

    get_folder("assets/audio/item", "ogg")
        .iter()
        .for_each(|path| {
            let name = PathBuf::from(path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            item_sound.insert(name, asset_server.load(path));
        });

    get_folder("assets/image/item", "png")
        .iter()
        .for_each(|path| {
            let name = PathBuf::from(path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            item_image.insert(name, asset_server.load(path));
        });

    get_folder("assets/image/attack", "png")
        .iter()
        .for_each(|path| {
            let name = PathBuf::from(path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            attack_image.insert(name, asset_server.load(path));
        });
    // サウンドをリソースに追加
    commands.insert_resource(Sounds {
        damage: damage_sound,
        item: item_sound,
    });
    commands.insert_resource(Images {
        item: item_image,
        attack: attack_image,
    });

    // プレイヤーを作成
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: INITIAL_PLAYER_POSITION.extend(0.0),
                scale: Vec3::new(40., 40., 0.),
                ..default()
            },
            ..default()
        },
        Player,
    ));
}

// entity(攻撃、アイテム)を動かす
fn move_entity(mut query: Query<(&mut Transform, Option<&Attack>, Option<&Item>)>) {
    for (mut transform, attack, item) in &mut query {
        // 攻撃の種類によって速度を変える
        if let Some(attack) = attack {
            transform.translation.y -= attack.0.speed();
        }
        if let Some(item) = item {
            transform.translation.y -= item.0.speed();
        }
    }
}

// 衝突の判定
fn check_for_collision(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    entity_query: Query<(Entity, &Transform, Option<&Attack>, Option<&Item>), With<Collider>>,
    sound: Res<Sounds>,
    player: Res<PlayerDate>,
    mut attack_state: ResMut<NextState<AttackType>>,
    mut item_state: ResMut<NextState<ItemType>>,
) {
    let player_transform = player_query.single();

    for (entity, transform, attack_type, item_type) in &entity_query {
        let collision = collide(
            player_transform.translation,
            player_transform.scale.truncate(),
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(_) = collision {
            if let Some(attack) = attack_type {
                let key = attack.0.sound_key();
                if let Some(audio) = sound.damage.get(&key) {
                    commands.spawn(AudioBundle {
                        source: audio.clone(),
                        settings: PlaybackSettings::DESPAWN,
                        ..default()
                    });
                }

                attack_state.set(attack.0);
            }
            if let Some(item_type) = item_type {
                let key = item_type.0.sound_key();
                if let Some(audio) = sound.item.get(&key) {
                    commands.spawn(AudioBundle {
                        source: audio.clone(),
                        settings: PlaybackSettings::DESPAWN,
                        ..default()
                    });
                }

                item_state.set(item_type.0);
            }

            println!("{:#?}", player.as_ref());

            commands.entity(entity).despawn();
        }
    }
}

// entityのランダムな位置
fn random_translation() -> Vec2 {
    let mut rng = thread_rng();
    Vec2::new(rng.gen_range(ENTITY_CREATE_RANGE), ENTITY_START_POSITION_Y)
}

// 指定された拡張子のファイルのみ抽出する
fn get_folder(target_path: &str, exten: &str) -> Vec<String> {
    let mut folder: Vec<String> = Vec::new();

    if let Ok(folder_path) = read_dir(target_path) {
        for dir_entry in folder_path {
            // 拡張子があったら
            if let Some(extension) = &dir_entry.as_ref().unwrap().path().extension() {
                // 指定した拡張子だったら
                if *extension == exten {
                    let path = dir_entry.unwrap().path().to_string_lossy().into_owned();
                    folder.push(path[7..].to_string());
                }
            }
        }
    } else {
        panic!("ディレクトリが見つかりません{}", target_path);
    }

    folder
}
