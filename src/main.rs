use bevy::time::common_conditions::on_timer;
use bevy::{prelude::*, sprite::collide_aabb::collide};
use std::collections::HashMap;
use std::fs::read_dir;
use std::path::PathBuf;
use std::{ops::RangeInclusive, time::Duration};

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

// プレイヤーの初期位置
const INITIAL_PLAYER_POSITION: Vec2 = Vec2::new(0.0, -300.0);
// プレイヤーが移動できる限界
const PLAYER_MOVE_LIMIT_LEFT: f32 = -375.0;
const PLAYER_MOVE_LIMIT_RIGHT: f32 = 375.0;
// プレイヤーの速度
const PLAYER_SPEED: f32 = 7.5;

// entityの初期位置のY座標
const ENTITY_START_POSITION_Y: f32 = 350.0;
// Entityが作られる範囲(プレイヤーが移動できる範囲)
const ENTITY_CREATE_RANGE: RangeInclusive<f32> = PLAYER_MOVE_LIMIT_LEFT..=PLAYER_MOVE_LIMIT_RIGHT;
// 攻撃が作られる時間の間隔
const ATTACK_CREATE_INTERVAR_TIME_RANGE: RangeInclusive<f32> = 0.1..=0.3;
// アイテムが作られる間隔
const _ITEM_CREATE_INTERVAR_TIME_RANGE: RangeInclusive<f32> = 5.0..=7.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "避けゲー".into(),
                resolution: (800., 720.).into(),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_seconds(0.5))
        .add_event::<DamageEvent>()
        .add_event::<ItemEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(
            Update,
            (
                move_player,
                move_entity,
                check_for_collision,
                damage_event,
                item_event,
            ),
        )
        .add_systems(FixedUpdate, create_attack)
        .add_systems(
            Update,
            create_item.run_if(on_timer(Duration::from_secs_f32(4.0))),
        )
        .run();
}

#[derive(Resource)]
struct Sounds {
    damage: Vec<Handle<AudioSource>>,
    item: HashMap<String, Handle<AudioSource>>,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Attack(AttackType);

#[derive(Component)]
struct Item(ItemType);

#[derive(Component)]
struct Collider;

#[derive(Bundle)]
struct AttackBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    attack: Attack,
}

#[derive(Bundle)]
struct ItemBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    item: Item,
}

#[derive(Event)]
struct DamageEvent {
    attack_type: AttackType,
}

#[derive(Event)]
struct ItemEvent {
    item_type: ItemType,
}

#[derive(Clone, Copy)]
enum AttackType {
    Normal,
    First,
}

#[derive(Clone, Copy)]
enum ItemType {
    Portion,
    SpeedUp,
}

impl AttackBundle {
    // 攻撃を新しく作る
    fn new(attack_type: AttackType) -> AttackBundle {
        AttackBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: random_translation().extend(0.0),
                    scale: attack_type.scale().extend(0.0),
                    ..default()
                },
                ..default()
            },
            collider: Collider,
            attack: Attack(attack_type),
        }
    }
}

impl ItemBundle {
    fn new(item_type: ItemType) -> ItemBundle {
        ItemBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: random_translation().extend(0.0),
                    scale: item_type.scale().extend(0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::GREEN,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
            item: Item(item_type),
        }
    }
}
fn random_translation() -> Vec2 {
    let mut rng = thread_rng();

    Vec2::new(rng.gen_range(ENTITY_CREATE_RANGE), ENTITY_START_POSITION_Y)
}

impl AttackType {
    // 攻撃の速度
    fn speed(&self) -> f32 {
        match self {
            AttackType::Normal => 7.0,
            AttackType::First => 10.0,
        }
    }
    // 攻撃の大きさ
    fn scale(&self) -> Vec2 {
        match self {
            AttackType::Normal => Vec2::new(25., 25.),
            AttackType::First => Vec2::new(30., 30.),
        }
    }

    fn random_item() -> AttackType {
        match thread_rng().gen_range(0..1) {
            0 => AttackType::Normal,
            _ => AttackType::First,
        }
    }
}
impl ItemType {
    fn speed(&self) -> f32 {
        match self {
            ItemType::Portion => 7.0,
            ItemType::SpeedUp => 10.0,
        }
    }
    // 攻撃の大きさ
    fn scale(&self) -> Vec2 {
        match self {
            ItemType::Portion => Vec2::new(25., 25.),
            ItemType::SpeedUp => Vec2::new(30., 30.),
        }
    }

    fn random_item() -> ItemType {
        match thread_rng().gen_range(0..=1) {
            0 => ItemType::Portion,
            _ => ItemType::SpeedUp,
        }
    }
    fn sound_key(&self) -> String {
        let key = match self {
            ItemType::Portion => "recovery",
            ItemType::SpeedUp => "powerup",
        };
        key.to_string()
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut damage_sound: Vec<Handle<AudioSource>> = Vec::new();
    get_folder("assets/audio/damage", "ogg")
        .iter()
        .for_each(|path| damage_sound.push(asset_server.load(path)));

    let mut item_sound: HashMap<String, Handle<AudioSource>> = HashMap::new();
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

    // サウンドをリソースに追加
    commands.insert_resource(Sounds {
        damage: damage_sound,
        item: item_sound,
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
    mut damage_event: EventWriter<DamageEvent>,
    mut item_event: EventWriter<ItemEvent>,
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
                damage_event.send(DamageEvent {
                    attack_type: attack.0,
                })
            }
            if let Some(item_type) = item_type {
                item_event.send(ItemEvent {
                    item_type: item_type.0,
                })
            }

            commands.entity(entity).despawn();
        }
    }
}

// 攻撃を作る
fn create_attack(mut commands: Commands, mut time: ResMut<Time<Fixed>>) {
    commands.spawn(AttackBundle::new(AttackType::random_item()));

    // 攻撃を作る間隔を変更する
    let random_time = thread_rng().gen_range(ATTACK_CREATE_INTERVAR_TIME_RANGE);
    time.set_timestep(Duration::from_secs_f32(random_time));
}

// アイテムを作る
fn create_item(mut commands: Commands) {
    commands.spawn(ItemBundle::new(ItemType::random_item()));
}

// 攻撃を受けたら
fn damage_event(
    mut commands: Commands,
    mut damage_event: EventReader<DamageEvent>,
    // mut query: Query<&mut Transform, With<Player>>,
    sound: Res<Sounds>,
) {
    for event in damage_event.read() {
        // ランダムな音を鳴らす
        let r_sound = sound.damage.choose(&mut thread_rng()).unwrap();
        commands.spawn(AudioBundle {
            source: r_sound.clone(),
            settings: PlaybackSettings::DESPAWN,
            ..default()
        });

        // 攻撃のタイプによって処理
        match event.attack_type {
            AttackType::Normal => {}
            AttackType::First => {}
        }
    }
}

// アイテムを回収したら
fn item_event(mut commands: Commands, mut item_event: EventReader<ItemEvent>, sound: Res<Sounds>) {
    for event in item_event.read() {
        let key = event.item_type.sound_key();
        if let Some(audio) = sound.item.get(&key) {
            commands.spawn(AudioBundle {
                source: audio.clone(),
                settings: PlaybackSettings::DESPAWN,
                ..default()
            });
        } else {
            error!("キーが存在しません: {}", key);
        }

        match event.item_type {
            ItemType::Portion => {}
            ItemType::SpeedUp => {}
        }
    }
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
