#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::ops::{Add, Div, Mul, Sub};
use std::path;

use conf::{WindowMode, WindowSetup};
use event::{quit, KeyCode, KeyMods};
use ggez::graphics::MeshBuilder;
use ggez::Context;
use ggez::*;
use graphics::Color;
use nalgebra as na;

type Point2 = na::Point2<f32>;
type Vector2 = na::Vector2<f32>;

const WORLD_WIDTH: f32 = 10_000.0;
const WORLD_HEIGHT: f32 = 10_000.0;

const SIZE: f32 = 50.0;
const HALF_SIZE: f32 = SIZE / 2.0;
const WIDTH: f32 = 1280.0;
const MIDDLE_X: f32 = WIDTH / 2.0;
const MIN_X: f32 = 0.0 + HALF_SIZE;
const MAX_X: f32 = WORLD_WIDTH - HALF_SIZE;
const HEIGHT: f32 = 720.0;
const MIDDLE_Y: f32 = HEIGHT / 2.0;
const MIN_Y: f32 = 0.0 + HALF_SIZE;
const MAX_Y: f32 = WORLD_HEIGHT - HALF_SIZE;
const FLOOR: f32 = MAX_Y - 30.0 - HALF_SIZE;
const FLOOR_THICKNESS: f32 = 5.0;
const ACCELERATION: f32 = 1.0;
const JUMP_ACCELERATION: f32 = 70.0;
const DESIRED_FPS: u32 = 60;
const MAX_VELOCITY: f32 = 10.0;
const MAX_JUMP_VELOCITY: f32 = 10.0;
const FRICTION: f32 = 0.5;
const NORMAL_FORCE: f32 = 1.0;
const CAMERA_DEADZONE: f32 = 16.0;
const BOTTOM: f32 = FLOOR + (FLOOR_THICKNESS / 2.0) + HALF_SIZE;

trait SafeNormalization {
    fn normalize_safe(&self) -> Self;
}

impl SafeNormalization for Vector2 {
    fn normalize_safe(&self) -> Self {
        self.try_normalize(f32::EPSILON)
            .unwrap_or_else(|| Vector2::new(0.0, 0.0))
    }
}

#[derive(Debug, PartialEq)]
enum CameraMode {
    Locked,
    Free,
}

#[derive(Debug)]
struct Walker {
    position: Point2,
    velocity: Vector2,
    acceleration: Vector2,
    color: Color,
    mass: f32,
}

impl Default for Walker {
    fn default() -> Self {
        Walker {
            position: Point2::new(0.0, 0.0),
            velocity: Vector2::new(0.0, 0.0),
            acceleration: Vector2::new(0.0, 0.0),
            color: graphics::BLACK,
            mass: Default::default(),
        }
    }
}

impl Walker {
    fn new(position: Point2, color: Color) -> Walker {
        Walker {
            position,
            color,
            acceleration: Vector2::new(0.0, 0.0),
            velocity: Vector2::new(0.0, 0.0),
            mass: 10.0,
        }
    }

    fn is_grounded(&self) -> bool {
        (FLOOR - self.position.y).abs() <= f32::EPSILON
    }

    fn apply_force(&mut self, force: Vector2) -> &mut Self {
        let force = force / self.mass;
        self.acceleration += force;
        self
    }

    fn apply_gravity(&mut self, force: &Vector2) -> &mut Self {
        self.acceleration += force;
        self
    }
}

#[derive(Debug)]
struct World {
    gravity: Vector2,
    float_gravity: Vector2,
    camera_center: Vector2,
    camera_mode: CameraMode,

    zoom: f32,
}

impl Default for World {
    fn default() -> Self {
        World {
            gravity: Vector2::new(0.0, 0.0),
            float_gravity: Vector2::new(0.0, 0.0),
            camera_center: Vector2::new(0.0, 0.0),
            camera_mode: CameraMode::Locked,
            zoom: 1.0,
        }
    }
}

#[derive(Debug, Default)]
struct GameState {
    tick: usize,
    highest_jump: f32,
    draw_highest_jump: bool,
    walker: Walker,
    world: World,
    left_pressed: bool,
    left_held: bool,
    right_pressed: bool,
    right_held: bool,
    up_pressed: bool,
    up_held: bool,
    down_pressed: bool,
    down_held: bool,
    jump_pressed: bool,
    jump_held: bool,
}

impl GameState {
    fn new(_ctx: &mut Context) -> ggez::GameResult<GameState> {
        let start_x = WIDTH / 2.0;
        let start_y = MAX_Y;

        Ok(GameState {
            walker: Walker::new(Point2::new(start_x, start_y), Color::from_rgb(0, 0, 255)),
            world: World {
                gravity: Vector2::new(0.0, 0.2),
                float_gravity: Vector2::new(0.0, 0.1),
                camera_center: Vector2::new(start_x, start_y),
                camera_mode: CameraMode::Locked,
                ..Default::default()
            },
            ..Default::default()
        })
    }
    fn relative_point(&mut self, point: Point2) -> Point2 {
        relative_point(self.world.camera_center, self.world.zoom, point)
    }

    fn draw_line(
        &mut self,
        ctx: &mut Context,
        thickness: f32,
        start: Point2,
        end: Point2,
    ) -> GameResult {
        let line = graphics::Mesh::new_line(
            ctx,
            &[self.relative_point(start), self.relative_point(end)],
            thickness * self.world.zoom,
            graphics::BLACK,
        )?;
        graphics::draw(ctx, &line, (Point2::new(0.0, 0.0),))
    }
}

#[inline(always)]
#[allow(unused)]
fn map_range<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Div<Output = T> + Copy>(
    val: T,
    start1: T,
    end1: T,
    start2: T,
    end2: T,
) -> T {
    (val - start1) / (end1 - start1) * (end2 - start2) + start2
}

#[inline(always)]
fn limit(v: f32, max: f32) -> f32 {
    if v.abs() > max {
        max * v.signum()
    } else {
        v
    }
}

/// Camera with zoom
///
/// relative_x = (x - camera_center.x) * zoom + (SCREE_WIDTH / 2)
///
/// relative_y = (y - camera_center.y) * zoom + (SCREE_HEIGHT / 2)
#[inline(always)]
fn relative_point(camera: Vector2, zoom: f32, point: Point2) -> Point2 {
    let moved = point - camera;
    let scaled = moved * zoom;
    Point2::new(scaled.x + MIDDLE_X, scaled.y + MIDDLE_Y)
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.tick += 1;
            if self.walker.position.y < self.highest_jump {
                self.highest_jump = self.walker.position.y;
            }

            if self.left_held {
                self.walker.apply_force(Vector2::new(-ACCELERATION, 0.0));
            }

            if self.right_held {
                self.walker.apply_force(Vector2::new(ACCELERATION, 0.0));
            }

            if self.walker.is_grounded() && self.jump_pressed {
                self.highest_jump = self.walker.position.y;
                let mag = self.walker.velocity.magnitude();
                if mag <= f32::EPSILON {
                    self.walker
                        .apply_force(Vector2::new(0.0, -JUMP_ACCELERATION));
                } else {
                    self.walker
                        .apply_force(Vector2::new(0.0, -JUMP_ACCELERATION * (1.0 + (mag / 30.0))));
                }
            }

            if self.walker.is_grounded() {
                let mut friction = self.walker.velocity.clone();
                friction *= -1.0;
                friction = friction.normalize_safe();
                friction *= FRICTION * NORMAL_FORCE;
                self.walker.apply_force(friction);
            }

            if self.jump_held {
                self.walker.apply_gravity(&self.world.float_gravity);
            } else {
                self.walker.apply_gravity(&self.world.gravity);
            }

            // apply acceleration
            self.walker.velocity += self.walker.acceleration;

            // reset acceleration
            self.walker.acceleration *= 0.0;

            // limit velocity
            self.walker.velocity.x = limit(self.walker.velocity.x, MAX_VELOCITY);
            self.walker.velocity.y = limit(self.walker.velocity.y, MAX_JUMP_VELOCITY);

            if self.walker.velocity.x.abs() < 0.06 {
                self.walker.velocity.x = 0.0;
            }

            if self.walker.velocity.y.abs() < 0.06 {
                self.walker.velocity.y = 0.0;
            }

            // apply velocity
            self.walker.position += self.walker.velocity;

            let difference = self.world.camera_center.x - self.walker.position.x;
            if difference.abs() > CAMERA_DEADZONE {
                self.world.camera_center.x =
                    self.walker.position.x + (CAMERA_DEADZONE * difference.signum())
            }
            if self.world.camera_mode == CameraMode::Free {
                self.world.camera_center.y = self.walker.position.y;
            }

            // // screen wrap
            // self.walker.position.x = self.walker.position.x.rem_euclid(WIDTH);
            // self.walker.position.y = self.walker.position.y.rem_euclid(HEIGHT);

            // bounce
            // if self.walker.position.x > MAX_X {
            //     self.walker.position.x = MAX_X;
            //     self.walker.velocity.x = self.walker.velocity.x * -1.0;
            // } else if self.walker.position.x < MIN_X {
            //     self.walker.position.x = MIN_X;
            //     self.walker.velocity.x = self.walker.velocity.x * -1.0;
            // }
            //
            // if self.walker.position.y > MAX_Y {
            //     self.walker.position.y = MAX_Y;
            //     self.walker.velocity.y = self.walker.velocity.y * -1.0;
            // } else if self.walker.position.y < MIN_Y {
            //     self.walker.position.y = MIN_Y;
            //     self.walker.velocity.y = self.walker.velocity.y * -1.0;
            // }

            // stop
            if self.walker.position.x >= MAX_X {
                self.walker.position.x = MAX_X;
                self.walker.velocity.x = 0.0;
            } else if self.walker.position.x <= MIN_X {
                self.walker.position.x = MIN_X;
                self.walker.velocity.x = 0.0;
            }

            if self.walker.position.y >= FLOOR {
                self.walker.position.y = FLOOR;
                self.walker.velocity.y = 0.0;
            } else if self.walker.position.y >= MAX_Y {
                self.walker.position.y = MAX_Y;
                self.walker.velocity.y = 0.0;
            } else if self.walker.position.y <= MIN_Y {
                self.walker.position.y = MIN_Y;
                self.walker.velocity.y = 0.0;
            }

            if self.world.camera_center.x < WIDTH / 2.0 {
                self.world.camera_center.x = WIDTH / 2.0;
            } else if self.world.camera_center.x > WORLD_WIDTH - (WIDTH / 2.0) {
                self.world.camera_center.x = WORLD_WIDTH - (WIDTH / 2.0)
            }
            if self.world.camera_center.y < HEIGHT / 2.0 {
                self.world.camera_center.y = HEIGHT / 2.0;
            } else if self.world.camera_center.y > WORLD_HEIGHT - (HEIGHT / 2.0) {
                self.world.camera_center.y = WORLD_HEIGHT - (HEIGHT / 2.0);
            }

            self.left_pressed = false;
            self.right_pressed = false;
            self.up_pressed = false;
            self.down_pressed = false;
            self.jump_pressed = false;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [1.0, 1.0, 1.0, 1.0].into());
        if self.draw_highest_jump {
            let half_thickness = self.highest_jump + (FLOOR_THICKNESS / 2.0) - HALF_SIZE;
            self.draw_line(
                ctx,
                FLOOR_THICKNESS,
                Point2::new(0.0, half_thickness),
                Point2::new(WIDTH, half_thickness),
            )?;
        }

        self.draw_line(
            ctx,
            FLOOR_THICKNESS,
            Point2::new(0.0, BOTTOM),
            Point2::new(WORLD_WIDTH, BOTTOM),
        )?;

        let size = HALF_SIZE;
        let mut mb = MeshBuilder::new();
        for i in 0..(WORLD_WIDTH / size) as i32 {
            let half_thickness = BOTTOM + 10.0;
            let start = i as f32 * size;
            mb.line(
                &[
                    self.relative_point(Point2::new(start, half_thickness)),
                    self.relative_point(Point2::new(start + 2.0, half_thickness)),
                ],
                FLOOR_THICKNESS,
                graphics::BLACK,
            )?;
        }
        let mesh = mb.build(ctx)?;
        graphics::draw(ctx, &mesh, (na::Point2::new(0.0, 0.0),))?;

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            self.relative_point(self.walker.position),
            HALF_SIZE * self.world.zoom,
            0.1,
            self.walker.color,
        )?;
        graphics::draw(ctx, &circle, (Point2::new(0.0, 0.0),))?;

        let difference_left = MAX_X - self.walker.position.x;
        let difference_right = self.walker.position.x - MIN_X;
        let new_x = if difference_left < 0.0 {
            Some(-HALF_SIZE + difference_left.abs())
        } else if difference_right < 0.0 {
            Some(WIDTH + HALF_SIZE - difference_right.abs())
        } else {
            None
        };
        if let Some(x) = new_x {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                self.relative_point(Point2::new(x, self.walker.position.y)),
                HALF_SIZE * self.world.zoom,
                0.1,
                self.walker.color,
            )?;
            graphics::draw(ctx, &circle, (Point2::new(0.0, 0.0),))?;
        }

        if self.tick % 50 == 0 {
            graphics::set_window_title(ctx, &format!("{:.0} FPS", timer::fps(ctx)));
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        if y > 0.0 {
            self.world.zoom = self.world.zoom - 0.1;
        } else if y < 0.0 {
            self.world.zoom = self.world.zoom + 0.1;
        }
        if self.world.zoom < 0.1 {
            self.world.zoom = 0.1;
        }
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        repeat: bool,
    ) {
        match keycode {
            KeyCode::Escape => quit(ctx),
            KeyCode::A | KeyCode::Left => {
                self.left_pressed = !repeat;
                self.left_held = true;
            }
            KeyCode::D | KeyCode::Right => {
                self.right_pressed = !repeat;
                self.right_held = true;
            }
            KeyCode::W /*| KeyCode::Up*/ => {
                self.up_pressed = !repeat;
                self.up_held = true;
            }
            KeyCode::S | KeyCode::Down => {
                self.down_pressed = !repeat;
                self.down_held = true;
            }
            KeyCode::Space | KeyCode::Up => {
                self.jump_pressed = !repeat;
                self.jump_held = true;
            }
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::A | KeyCode::Left => {
                self.left_pressed = false;
                self.left_held = false;
            }
            KeyCode::D | KeyCode::Right => {
                self.right_pressed = false;
                self.right_held = false;
            }
            KeyCode::W | KeyCode::Up => {
                self.up_pressed = false;
                self.up_held = false;
            }
            KeyCode::S | KeyCode::Down => {
                self.down_pressed = false;
                self.down_held = false;
            }
            KeyCode::Space => {
                self.jump_pressed = false;
                self.jump_held = false;
            }
            _ => (),
        }
    }
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("platformer", "ggez")
        .add_resource_path(resource_dir)
        .window_mode(WindowMode {
            width: WIDTH,
            height: HEIGHT,
            ..Default::default()
        })
        .window_setup(WindowSetup {
            vsync: false,
            ..Default::default()
        });
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut GameState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
