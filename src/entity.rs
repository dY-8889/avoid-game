use bevy::{
    ecs::{change_detection::Res, system::Commands},
    math::Vec2,
};

use crate::Images;

pub trait EntityBundle {}
pub trait EntityType {
    fn speed(&self) -> f32;
    fn scale(&self) -> Vec2;
    fn random() -> Self;
    fn sound_key(&self) -> String;
    fn image_key(&self) -> String;
    fn create(commands: Commands, image: Res<Images>);
}
