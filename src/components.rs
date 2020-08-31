use ggez::graphics::Rect;

use crate::common::*;
use crate::physics::*;

pub struct Velocity(pub Vector2);

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Velocity(Vector2::new(x, y))
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Velocity(Vector2::new(0.0, 0.0))
    }
}

pub struct Acceleration(pub Vector2);

impl Acceleration {
    pub fn new(x: f32, y: f32) -> Self {
        Acceleration(Vector2::new(x, y))
    }
}

impl PhysicsHelper for Acceleration {
    fn apply_force(&mut self, force: &Vector2, mass: f32) -> &mut Self {
        apply_force(&mut self.0, force, mass);
        self
    }

    fn apply_gravity(&mut self, force: &Vector2) -> &mut Self {
        apply_gravity(&mut self.0, force);
        self
    }
}

pub struct Mass(pub f32);

pub struct Gravity(pub Vector2);

pub struct Position(pub Point2);

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Position(Point2::new(x, y))
    }
}

#[derive(Default)]
pub struct Player;

#[derive(Default)]
pub struct ZOrder(pub i32);

pub struct BoundingBox(pub Rect);

pub struct Grounded(pub i32);

pub struct Death;

pub struct LevelId(pub DefaultAtom);

pub struct TeleportTo(pub String);

pub struct TextContainer {
    pub value: String,
    pub size: f32,
}
