use bevy::{prelude::*, sprite::collide_aabb::collide};
use std::{ops::RangeInclusive, time::Duration};

use rand::{thread_rng, Rng};

// プレイヤーの初期位置
const INITIAL_PLAYER_POSITION: Vec2 = Vec2::new(0.0, -300.0);
// プレイヤーが移動できる限界
const PLAYER_MOVE_LIMIT_LEFT: f32 = -475.0;
const PLAYER_MOVE_LIMIT_RIGHT: f32 = 475.0;
// プレイヤーの速度
const PLAYER_SPEED: f32 = 7.5;

// 攻撃の初期位置のY座標
const ATTACK_START_POSITION_Y: f32 = 350.0;
// 攻撃が作られる範囲(プレイヤーが移動できる範囲)
const ATTACK_CREATE_RANGE: RangeInclusive<f32> = PLAYER_MOVE_LIMIT_LEFT..=PLAYER_MOVE_LIMIT_RIGHT;
// 攻撃が作られる時間の間隔
const ATTACK_CREATE_INTERVAR_TIME_RANGE: RangeInclusive<f32> = 0.2..=0.5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "避けゲー".into(),
                resolution: (1000., 720.).into(),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_seconds(0.2))
        .add_event::<DamageEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(
            Update,
            (move_player, move_attack, check_for_collision, damage_event),
        )
        .add_systems(FixedUpdate, create_attack)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Attack;

#[derive(Bundle)]
struct AttackBundle {
    sprite_bundle: SpriteBundle,
    attack: Attack,
    attack_type: AttackType,
}

#[derive(Event)]
struct DamageEvent {
    attack_type: AttackType,
}

#[derive(Clone, Copy, Component)]
enum AttackType {
    Normal,
}

impl AttackBundle {
    // 攻撃を新しく作る
    fn new(attack_type: AttackType) -> AttackBundle {
        AttackBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: Self::random_translation().extend(0.0),
                    scale: attack_type.scale().extend(0.0),
                    ..default()
                },
                ..default()
            },
            attack: Attack,
            attack_type,
        }
    }
    // ランダムな位置に攻撃を出現させる
    fn random_translation() -> Vec2 {
        let mut rng = thread_rng();

        Vec2::new(rng.gen_range(ATTACK_CREATE_RANGE), ATTACK_START_POSITION_Y)
    }
}

impl AttackType {
    const NORMAL: Vec2 = Vec2::new(25., 25.);

    // 攻撃の速度
    fn speed(&self) -> f32 {
        match self {
            AttackType::Normal => 10.0,
        }
    }
    // 攻撃の大きさ
    fn scale(&self) -> Vec2 {
        match self {
            AttackType::Normal => Vec2::new(25., 25.),
        }
    }
    fn create_intervar(&self) -> f32 {
        match self {
            AttackType::Normal => 0.3,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

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

// 衝突の判定
fn check_for_collision(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    attack_query: Query<(Entity, &Transform, &AttackType), With<Attack>>,
    mut damage_event: EventWriter<DamageEvent>,
) {
    let player_transform = player_query.single();

    for (attack_entity, attack_transform, attack_type) in &attack_query {
        let collision = collide(
            player_transform.translation,
            player_transform.scale.truncate(),
            attack_transform.translation,
            attack_transform.scale.truncate(),
        );
        if let Some(_) = collision {
            commands.entity(attack_entity).despawn();

            damage_event.send(DamageEvent {
                attack_type: *attack_type,
            })
        }
    }
}

// 攻撃を作る
fn create_attack(mut commands: Commands, mut time: ResMut<Time<Fixed>>) {
    commands.spawn(AttackBundle::new(AttackType::Normal));

    // 攻撃を作る感覚を変更する
    let random_time = thread_rng().gen_range(ATTACK_CREATE_INTERVAR_TIME_RANGE);
    time.set_timestep(Duration::from_secs_f32(random_time));
}

// 攻撃を動かす
fn move_attack(mut query: Query<(&mut Transform, &AttackType), With<Attack>>) {
    for (mut transform, attack_type) in &mut query {
        transform.translation.y -= attack_type.speed();
    }
}
// プレイヤーが攻撃と衝突した時に発生するイベントの処理
fn damage_event(
    mut damage_event: EventReader<DamageEvent>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    for event in damage_event.read() {
        match event.attack_type {
            AttackType::Normal => query.single_mut().scale += 2.,
        }
    }
}
