#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate serde_derive;

use std::env;
use std::ops::{Add, Div, Mul, Sub};
use std::path;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{quit, KeyCode, KeyMods};
use ggez::graphics::{Mesh, MeshBuilder};
use ggez::Context;
use ggez::*;
use graphics::Color;
use hecs::*;
use itertools::Itertools;
use nalgebra as na;

type Point2 = na::Point2<f32>;
type Vector2 = na::Vector2<f32>;

const WORLD_WIDTH: f32 = 10_000.0;
const WORLD_HEIGHT: f32 = 10_000.0;
const WIDTH: f32 = 1280.0;
const MIDDLE_X: f32 = WIDTH / 2.0;
const HEIGHT: f32 = 720.0;
const MIDDLE_Y: f32 = HEIGHT / 2.0;
const FLOOR: f32 = ((WORLD_HEIGHT / GRID_SIZE) as i32 as f32 * GRID_SIZE) - (GRID_SIZE * 2.0);
const FLOOR_THICKNESS: f32 = 5.0;
const GRID_SIZE: f32 = 32.0;
const GRID_THICKNESS: f32 = 1.0;
const DESIRED_FPS: u32 = 60;

trait SafeNormalization {
    fn normalize_safe(&self) -> Self;
}

impl SafeNormalization for Vector2 {
    fn normalize_safe(&self) -> Self {
        self.try_normalize(f32::EPSILON)
            .unwrap_or_else(|| Vector2::new(0.0, 0.0))
    }
}

trait PhysicsHelper {
    fn apply_force(&mut self, force: &Vector2, mass: f32) -> &mut Self;

    fn apply_gravity(&mut self, force: &Vector2) -> &mut Self;
}

fn apply_force(v: &mut Vector2, force: &Vector2, mass: f32) {
    let force = force / mass;
    *v += force;
}

fn apply_gravity(v: &mut Vector2, force: &Vector2) {
    *v += force;
}

#[derive(Deserialize, Debug)]
struct PlayerConfig {
    acceleration: f32,
    jump_acceleration: f32,
    mass: f32,
    size: f32,
    float_modifier: f32,
    allow_air_control: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        PlayerConfig {
            acceleration: 1.0,
            jump_acceleration: 70.0,
            mass: 10.0,
            size: 50.0,
            float_modifier: 0.5,
            allow_air_control: false,
        }
    }
}

#[derive(Deserialize, Debug)]
struct CameraConfig {
    deadzone: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        CameraConfig { deadzone: 16.0 }
    }
}

#[derive(Deserialize, Debug)]
struct PhysicsConfig {
    max_horizontal_velocity: f32,
    max_vertical_velocity: f32,
    friction: f32,
    normal_force: f32,
    gravity: f32,
    movement_deadzone: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        PhysicsConfig {
            max_horizontal_velocity: 10.0,
            max_vertical_velocity: 10.0,
            friction: 0.5,
            normal_force: 1.0,
            gravity: 0.2,
            movement_deadzone: 0.0001,
        }
    }
}

#[derive(Deserialize, Debug, Default)]
struct Config {
    player: PlayerConfig,
    camera: CameraConfig,
    physics: PhysicsConfig,
}

#[derive(Debug, PartialEq)]
enum CameraMode {
    Locked,
    Free,
}

struct Position(Point2);

impl Position {
    fn new(x: f32, y: f32) -> Self {
        Position(Point2::new(x, y))
    }

    fn is_grounded(&self) -> bool {
        (FLOOR - self.0.y).abs() <= f32::EPSILON
    }
}

struct Acceleration(Vector2);

impl Acceleration {
    fn new(x: f32, y: f32) -> Self {
        Acceleration(Vector2::new(x, y))
    }
}

struct Size(f32);

struct Velocity(Vector2);

impl Velocity {
    fn new(x: f32, y: f32) -> Self {
        Velocity(Vector2::new(x, y))
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Velocity(Vector2::new(0.0, 0.0))
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

struct Mass(f32);

struct Gravity(Vector2);

#[derive(Debug)]
struct Camera {
    center: Vector2,
    mode: CameraMode,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            center: Vector2::new(0.0, 0.0),
            mode: CameraMode::Free,
        }
    }
}

#[derive(Default)]
struct Player;

#[derive(Default)]
struct ZOrder(i32);

#[derive(Default)]
struct Controls {
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

struct GameState {
    tick: usize,
    config: Config,
    camera: Camera,
    world: World,
    controls: Controls,
}

impl GameState {
    fn new(ctx: &mut Context) -> ggez::GameResult<GameState> {
        let start_x = WIDTH / 2.0;
        let start_y = WORLD_HEIGHT - 300.0;

        let mut world = World::new();

        let config: Config = std::fs::read_to_string("config.toml")
            .map(|config_string| toml::from_str(&config_string).unwrap_or_default())
            .unwrap_or_default();

        world.spawn((
            Player,
            Position::new(start_x, start_y),
            Acceleration::new(0.0, 0.0),
            Velocity::new(0.0, 0.0),
            Size(config.player.size),
            Mass(config.player.mass),
            Gravity(Vector2::new(0.0, config.physics.gravity)),
            graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                Point2::new(0.0, -(config.player.size / 2.0)),
                config.player.size / 2.0,
                0.1,
                Color::from_rgb(0, 0, 255),
            )?,
            ZOrder(10),
        ));

        world.spawn((
            Position::new(start_x + 50.0, start_y - 100.0),
            Acceleration::new(0.0, 0.0),
            Velocity::new(0.0, 0.0),
            Size(config.player.size / 2.0),
            Mass(config.player.mass),
            Gravity(Vector2::new(0.0, 0.01)),
            graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                Point2::new(0.0, -(config.player.size / 4.0)),
                config.player.size / 4.0,
                0.1,
                Color::from_rgb(255, 0, 0),
            )?,
            ZOrder(20),
        ));

        let mut mb = MeshBuilder::new();
        mb.line(
            &[
                Point2::new(0.0, FLOOR + (FLOOR_THICKNESS / 2.0)),
                Point2::new(WORLD_WIDTH, FLOOR + (FLOOR_THICKNESS / 2.0)),
            ],
            FLOOR_THICKNESS,
            graphics::BLACK,
        )?;

        for i in 0..(WORLD_WIDTH / GRID_SIZE) as i32 + 1 {
            let start = i as f32 * GRID_SIZE;
            mb.line(
                &[Point2::new(start, 0.0), Point2::new(start, WORLD_HEIGHT)],
                GRID_THICKNESS,
                graphics::BLACK,
            )?;
        }
        for i in 0..(WORLD_HEIGHT / GRID_SIZE) as i32 + 1 {
            let start = i as f32 * GRID_SIZE;
            mb.line(
                &[Point2::new(0.0, start), Point2::new(WORLD_WIDTH, start)],
                GRID_THICKNESS,
                graphics::BLACK,
            )?;
        }
        world.spawn((Position::new(0.0, 0.0), mb.build(ctx)?, ZOrder(30)));

        Ok(GameState {
            config,
            world,
            camera: Camera::default(),
            controls: Controls::default(),
            tick: 0,
        })
    }

    #[inline(always)]
    fn do_movement(&mut self, _ctx: &mut Context) {
        for (_id, (acceleration, gravity, velocity, pos, mass, _)) in &mut self.world.query::<(
            &mut Acceleration,
            &mut Gravity,
            &Velocity,
            &Position,
            &Mass,
            &Player,
        )>() {
            if pos.is_grounded() || self.config.player.allow_air_control {
                if self.controls.left_held {
                    acceleration
                        .apply_force(&Vector2::new(-self.config.player.acceleration, 0.0), mass.0);
                }

                if self.controls.right_held {
                    acceleration
                        .apply_force(&Vector2::new(self.config.player.acceleration, 0.0), mass.0);
                }
            }

            if pos.is_grounded() {
                if self.controls.jump_pressed {
                    let mag = velocity.0.magnitude();
                    if mag <= f32::EPSILON {
                        acceleration.apply_force(
                            &Vector2::new(0.0, -self.config.player.jump_acceleration),
                            mass.0,
                        );
                    } else {
                        acceleration.apply_force(
                            &Vector2::new(
                                0.0,
                                -self.config.player.jump_acceleration * (1.0 + (mag / 30.0)),
                            ),
                            mass.0,
                        );
                    }
                }
            }
            if self.controls.jump_held {
                gravity.0.y = self.config.physics.gravity * self.config.player.float_modifier;
            } else {
                gravity.0.y = self.config.physics.gravity;
            }
        }
    }

    #[inline(always)]
    fn apply_physics(&mut self, _ctx: &mut Context) {
        for (_id, (acceleration, velocity, position, mass, gravity, size)) in
            &mut self.world.query::<(
                &mut Acceleration,
                &mut Velocity,
                &mut Position,
                &Mass,
                &Gravity,
                &Size,
            )>()
        {
            acceleration.apply_gravity(&gravity.0);

            if position.is_grounded() {
                let mut friction = velocity.0.clone();
                friction *= -1.0;
                friction = friction.normalize_safe();
                friction *= self.config.physics.friction * self.config.physics.normal_force;
                acceleration.apply_force(&friction, mass.0);
            }

            // apply acceleration
            velocity.0 += acceleration.0;

            acceleration.0 *= 0.0;
            // limit velocity
            velocity.0.x = limit(velocity.0.x, self.config.physics.max_horizontal_velocity);
            velocity.0.y = limit(velocity.0.y, self.config.physics.max_vertical_velocity);

            if velocity.0.x.abs() < self.config.physics.movement_deadzone {
                velocity.0.x = 0.0;
            }

            if velocity.0.y.abs() < self.config.physics.movement_deadzone {
                velocity.0.y = 0.0;
            }

            // apply velocity
            position.0 += velocity.0;

            let half_size = size.0 / 2.0;
            let max_x = WORLD_WIDTH - half_size;
            let min_x = half_size;

            // stop
            if position.0.x >= max_x {
                position.0.x = max_x;
                velocity.0.x = 0.0;
            } else if position.0.x <= min_x {
                position.0.x = min_x;
                velocity.0.x = 0.0;
            }

            let max_y = WORLD_HEIGHT - half_size;
            let min_y = half_size;

            if position.0.y >= FLOOR {
                position.0.y = FLOOR;
                velocity.0.y = 0.0;
            } else if position.0.y >= max_y {
                position.0.y = max_y;
                velocity.0.y = 0.0;
            } else if position.0.y <= min_y {
                position.0.y = min_y;
                velocity.0.y = 0.0;
            }
        }
    }

    #[inline(always)]
    fn move_camera(&mut self, _ctx: &mut Context) {
        for (_id, (position, _)) in &mut self.world.query::<(&Position, &Player)>() {
            let difference = self.camera.center.x - position.0.x;
            if difference.abs() > self.config.camera.deadzone {
                self.camera.center.x =
                    position.0.x + (self.config.camera.deadzone * difference.signum())
            }
            if self.camera.mode == CameraMode::Free {
                self.camera.center.y = position.0.y;
            }

            if self.camera.center.x < WIDTH / 2.0 {
                self.camera.center.x = WIDTH / 2.0;
            } else if self.camera.center.x > WORLD_WIDTH - (WIDTH / 2.0) {
                self.camera.center.x = WORLD_WIDTH - (WIDTH / 2.0)
            }
            if self.camera.center.y < HEIGHT / 2.0 {
                self.camera.center.y = HEIGHT / 2.0;
            } else if self.camera.center.y > WORLD_HEIGHT - (HEIGHT / 2.0) {
                self.camera.center.y = WORLD_HEIGHT - (HEIGHT / 2.0);
            }
        }
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

/// Camera
///
/// relative_x = (x - camera_center.x) + (SCREE_WIDTH / 2)
///
/// relative_y = (y - camera_center.y) + (SCREE_HEIGHT / 2)
#[inline(always)]
fn relative_point(camera: Vector2, point: Point2) -> Point2 {
    let moved = point - camera;
    let scaled = moved;
    Point2::new(scaled.x + MIDDLE_X, scaled.y + MIDDLE_Y)
}

impl ggez::event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.tick += 1;

            self.do_movement(ctx);
            self.apply_physics(ctx);
            self.move_camera(ctx);

            self.controls.left_pressed = false;
            self.controls.right_pressed = false;
            self.controls.up_pressed = false;
            self.controls.down_pressed = false;
            self.controls.jump_pressed = false;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [1.0, 1.0, 1.0, 1.0].into());

        for (_id, (pos, mesh, _z_order)) in &mut self
            .world
            .query::<(&Position, &Mesh, &ZOrder)>()
            .iter()
            .sorted_by_key(|(_id, (_pos, _mesh, z_order))| -z_order.0)
        // sort by z-order, descending
        {
            graphics::draw(ctx, &*mesh, (relative_point(self.camera.center, pos.0),))?;
        }

        if self.tick % 50 == 0 {
            graphics::set_window_title(ctx, &format!("{:.0} FPS", timer::fps(ctx)));
        }

        graphics::present(ctx)?;
        Ok(())
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
                self.controls.left_pressed = !repeat;
                self.controls.left_held = true;
            }
            KeyCode::D | KeyCode::Right => {
                self.controls.right_pressed = !repeat;
                self.controls.right_held = true;
            }
            KeyCode::W /*| KeyCode::Up*/ => {
                self.controls.up_pressed = !repeat;
                self.controls.up_held = true;
            }
            KeyCode::S | KeyCode::Down => {
                self.controls.down_pressed = !repeat;
                self.controls.down_held = true;
            }
            KeyCode::Space | KeyCode::Up => {
                self.controls.jump_pressed = !repeat;
                self.controls.jump_held = true;
            }
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::A | KeyCode::Left => {
                self.controls.left_pressed = false;
                self.controls.left_held = false;
            }
            KeyCode::D | KeyCode::Right => {
                self.controls.right_pressed = false;
                self.controls.right_held = false;
            }
            KeyCode::W | KeyCode::Up => {
                self.controls.up_pressed = false;
                self.controls.up_held = false;
            }
            KeyCode::S | KeyCode::Down => {
                self.controls.down_pressed = false;
                self.controls.down_held = false;
            }
            KeyCode::Space => {
                self.controls.jump_pressed = false;
                self.controls.jump_held = false;
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
    ggez::event::run(ctx, event_loop, state)
}
