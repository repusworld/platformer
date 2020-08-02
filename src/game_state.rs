use ggez::graphics;
use graphics::Color;

use crate::common::*;
use crate::components::*;
use ggez::graphics::Rect;

#[derive(Deserialize, Debug)]
pub struct PlayerConfig {
    pub acceleration: f32,
    pub jump_acceleration: f32,
    pub mass: f32,
    pub size: f32,
    pub float_modifier: f32,
    pub allow_air_control: bool,
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
pub struct CameraConfig {
    pub deadzone: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        CameraConfig { deadzone: 16.0 }
    }
}

#[derive(Deserialize, Debug)]
pub struct DebugConfig {
    pub draw_grid: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        DebugConfig { draw_grid: false }
    }
}

#[derive(Deserialize, Debug)]
pub struct PhysicsConfig {
    pub max_horizontal_velocity: f32,
    pub max_vertical_velocity: f32,
    pub friction: f32,
    pub normal_force: f32,
    pub gravity: f32,
    pub movement_deadzone: f32,
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
pub struct Config {
    pub player: PlayerConfig,
    pub camera: CameraConfig,
    pub physics: PhysicsConfig,
    pub debug: DebugConfig,
}

#[derive(Debug, PartialEq)]
pub enum CameraMode {
    Locked,
    Free,
}

#[derive(Debug)]
pub struct Camera {
    pub center: Vector2,
    pub mode: CameraMode,
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
pub struct Controls {
    pub left_pressed: bool,
    pub left_held: bool,
    pub right_pressed: bool,
    pub right_held: bool,
    pub up_pressed: bool,
    pub up_held: bool,
    pub down_pressed: bool,
    pub down_held: bool,
    pub jump_pressed: bool,
    pub jump_held: bool,
}

pub struct GameState {
    pub tick: usize,
    pub config: Config,
    pub camera: Camera,
    pub world: World,
    pub controls: Controls,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> ggez::GameResult<GameState> {
        let config: Config = std::fs::read_to_string("config.toml")
            .map(|config_string| toml::from_str(&config_string).unwrap_or_default())
            .unwrap_or_default();

        let mut world = World::new();

        let start_x = WIDTH / 2.0;
        let start_y = WORLD_HEIGHT - 300.0;
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
            ZOrder(0),
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
            ZOrder(10),
        ));

        world.spawn((
            Position::new(0.0, 0.0),
            graphics::Mesh::new_line(
                ctx,
                &[
                    Point2::new(0.0, FLOOR + (GRID_SIZE / 2.0)),
                    Point2::new(WORLD_WIDTH, FLOOR + (GRID_SIZE / 2.0)),
                ],
                GRID_SIZE,
                graphics::BLACK,
            )?,
            ZOrder(20),
            BoundingBox(Rect::new(0.0, FLOOR + GRID_SIZE, WORLD_WIDTH, GRID_SIZE)),
        ));

        let mut mb = graphics::MeshBuilder::new();
        if config.debug.draw_grid {
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
        }
        world.spawn((Position::new(0.0, 0.0), mb.build(ctx)?, ZOrder(20)));

        Ok(GameState {
            config,
            world,
            camera: Camera::default(),
            controls: Controls::default(),
            tick: 0,
        })
    }
}
