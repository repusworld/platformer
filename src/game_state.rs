use std::collections::HashMap;

use ggez::graphics;
use maplit::hashmap;

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
    pub current_level: String,
    pub levels: HashMap<String, Level>,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> ggez::GameResult<GameState> {
        let config: Config = std::fs::read_to_string("config.toml")
            .map(|config_string| toml::from_str(&config_string).unwrap_or_default())
            .unwrap_or_default();

        let current_level = "start".to_string();
        let mut levels = match std::path::Path::new("levels").read_dir() {
            Ok(d) => d
                .flatten()
                .flat_map(|f| {
                    if f.metadata().unwrap().is_file() {
                        Some(f.path())
                    } else {
                        None
                    }
                })
                .map(|mut f| {
                    let level = std::fs::read_to_string(&f)
                        .map(|data| toml::from_str::<Level>(&data).ok())
                        .ok()
                        .flatten();
                    f.set_extension("");
                    (
                        f.file_name()
                            .expect("File name is not valid utf-8!")
                            .to_string_lossy()
                            .to_string(),
                        level,
                    )
                })
                .filter(|(_, l)| l.is_some())
                .map(|(f, l)| (f, l.unwrap()))
                .collect::<HashMap<_, _>>(),
            _ => hashmap! {"start".to_string() => Level::default()},
        };

        if levels.is_empty() {
            levels.insert("start".to_string(), Level::default());
        }

        let mut world = World::new();

        if config.debug.draw_grid {
            let mut mb = graphics::MeshBuilder::new();
            for i in 0..(levels[&current_level].size.width / config.player.size) as i32 + 1 {
                let start = i as f32 * config.player.size;
                mb.line(
                    &[
                        Point2::new(start, 0.0),
                        Point2::new(start, levels[&current_level].size.height),
                    ],
                    GRID_THICKNESS,
                    graphics::BLACK,
                )?;
            }
            for i in 0..(levels[&current_level].size.height / config.player.size) as i32 + 1 {
                let start = i as f32 * config.player.size;
                mb.line(
                    &[
                        Point2::new(0.0, start),
                        Point2::new(levels[&current_level].size.width, start),
                    ],
                    GRID_THICKNESS,
                    graphics::BLACK,
                )?;
            }
            world.spawn((Position::new(0.0, 0.0), mb.build(ctx)?, ZOrder(20)));
        }

        let mut game_state = GameState {
            config,
            world,
            current_level: current_level.clone(),
            levels,
            camera: Camera::default(),
            controls: Controls::default(),
            tick: 0,
        };

        game_state.change_level(ctx, current_level)?;

        Ok(game_state)
    }
}
