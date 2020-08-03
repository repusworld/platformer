use ggez::graphics;
use ggez::graphics::{DrawMode, Rect};
use graphics::Color;

use crate::common::*;
use crate::components::*;
use crate::config::*;
use crate::level::*;

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
    pub debug_pressed: bool,
    pub debug_held: bool,
}

pub struct GameState {
    pub tick: usize,
    pub config: Config,
    pub camera: Camera,
    pub world: World,
    pub controls: Controls,
    pub level_size: LevelSize,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> ggez::GameResult<GameState> {
        let config: Config = std::fs::read_to_string("config.toml")
            .map(|config_string| toml::from_str(&config_string).unwrap_or_default())
            .unwrap_or_default();

        let level: Level =
            toml::from_str(&std::fs::read_to_string("level.toml").unwrap_or_default())
                .unwrap_or_default();

        let mut world = World::new();

        let start_x = level.start.x * config.player.size;
        let start_y = level.size.height - (level.start.y * config.player.size);
        world.spawn((
            Player,
            Position::new(start_x, start_y),
            Acceleration::new(0.0, 0.0),
            Velocity::new(0.0, 0.0),
            Mass(config.player.mass),
            Gravity(Vector2::new(0.0, config.physics.gravity)),
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(
                    -(config.player.size / 2.0),
                    -config.player.size,
                    config.player.size,
                    config.player.size,
                ),
                Color::from_rgb(0, 0, 255),
            )?,
            ZOrder(0),
            BoundingBox(Rect::new(
                -(config.player.size / 2.0),
                -config.player.size,
                config.player.size,
                config.player.size,
            )),
        ));

        for platform in level.platforms {
            let x = platform.x * config.player.size;
            let y = level.size.height - (platform.y * config.player.size);
            let width = platform.width * config.player.size;
            let height = platform.height * config.player.size;
            world.spawn((
                graphics::Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect::new(0.0, 0.0, width, height),
                    graphics::BLACK,
                )?,
                ZOrder(20),
                BoundingBox(Rect::new(x, y, width, height)),
            ));
        }

        if config.debug.draw_grid {
            let mut mb = graphics::MeshBuilder::new();
            for i in 0..(level.size.width / config.player.size) as i32 + 1 {
                let start = i as f32 * config.player.size;
                mb.line(
                    &[
                        Point2::new(start, 0.0),
                        Point2::new(start, level.size.height),
                    ],
                    GRID_THICKNESS,
                    graphics::BLACK,
                )?;
            }
            for i in 0..(level.size.height / config.player.size) as i32 + 1 {
                let start = i as f32 * config.player.size;
                mb.line(
                    &[
                        Point2::new(0.0, start),
                        Point2::new(level.size.width, start),
                    ],
                    GRID_THICKNESS,
                    graphics::BLACK,
                )?;
            }
            world.spawn((Position::new(0.0, 0.0), mb.build(ctx)?, ZOrder(20)));
        }

        Ok(GameState {
            config,
            world,
            level_size: level.size.clone(),
            camera: Camera::default(),
            controls: Controls::default(),
            tick: 0,
        })
    }
}
