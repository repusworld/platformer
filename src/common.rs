pub use ggez::nalgebra;
pub use ggez::Context;
pub use hecs::*;

pub type Point2 = nalgebra::Point2<f32>;
pub type Vector2 = nalgebra::Vector2<f32>;

use std::ops::{Add, Div, Mul, Sub};

pub use crate::game_state::*;

pub const WORLD_WIDTH: f32 = 10_000.0;
pub const WORLD_HEIGHT: f32 = 10_000.0;
pub const WIDTH: f32 = 1280.0;
pub const MIDDLE_X: f32 = WIDTH / 2.0;
pub const HEIGHT: f32 = 720.0;
pub const MIDDLE_Y: f32 = HEIGHT / 2.0;
pub const FLOOR: f32 = ((WORLD_HEIGHT / GRID_SIZE) as i32 as f32 * GRID_SIZE) - (GRID_SIZE * 2.0);
pub const GRID_SIZE: f32 = 32.0;
pub const GRID_THICKNESS: f32 = 1.0;
pub const DESIRED_FPS: u32 = 60;

#[inline(always)]
#[allow(unused)]
pub fn map_range<
    T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Div<Output = T> + Copy,
>(
    val: T,
    start1: T,
    end1: T,
    start2: T,
    end2: T,
) -> T {
    (val - start1) / (end1 - start1) * (end2 - start2) + start2
}

/// Camera
///
/// relative_x = (x - camera_center.x) + (SCREE_WIDTH / 2)
///
/// relative_y = (y - camera_center.y) + (SCREE_HEIGHT / 2)
#[inline(always)]
pub fn relative_point(camera: Vector2, point: Point2) -> Point2 {
    let moved = point - camera;
    let scaled = moved /* * zoom */;
    Point2::new(scaled.x + MIDDLE_X, scaled.y + MIDDLE_Y)
}
