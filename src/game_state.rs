use ggez::graphics;
use graphics::Color;

use crate::common::*;
use crate::components::*;
use crate::config::*;
use ggez::graphics::Rect;

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
