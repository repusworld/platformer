use ggez::graphics;
use ggez::graphics::{Color, DrawMode, Rect};

use crate::common::*;
use crate::components::*;

impl GameState {
    #[inline(always)]
    pub fn restart_level(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.change_level(ctx, self.current_level.clone())
    }

    #[inline(always)]
    pub fn change_level(&mut self, ctx: &mut Context, new_level: String) -> GameResult<()> {
        let current_level_atom = DefaultAtom::from(self.current_level.clone());
        let ids = self
            .world
            .query::<&LevelId>()
            .iter()
            .filter(|(_, LevelId(level))| *level == current_level_atom)
            .map(|(id, _)| id)
            .collect::<Vec<_>>();

        for id in ids {
            let _ = self.world.despawn(id);
        }

        self.current_level = new_level;

        let current_level_atom = DefaultAtom::from(self.current_level.clone());
        let start_x = self.levels[&self.current_level].start.x * self.config.player.size;
        let start_y = self.levels[&self.current_level].size.height
            - (self.levels[&self.current_level].start.y * self.config.player.size)
            - 1.0;
        self.world.spawn((
            Player,
            Position::new(start_x, start_y),
            Acceleration::new(0.0, 0.0),
            Velocity::new(0.0, 0.0),
            Mass(self.config.player.mass),
            Gravity(Vector2::new(0.0, self.config.physics.gravity)),
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(
                    -(self.config.player.size / 2.0),
                    -self.config.player.size,
                    self.config.player.size,
                    self.config.player.size,
                ),
                Color::from_rgb(0, 0, 255),
            )?,
            ZOrder(0),
            BoundingBox(Rect::new(
                -(self.config.player.size / 2.0),
                -self.config.player.size,
                self.config.player.size,
                self.config.player.size,
            )),
            LevelId(current_level_atom.clone()),
        ));

        for platform in &self.levels[&self.current_level].platforms {
            let x = platform.x * self.config.player.size;
            let y = self.levels[&self.current_level].size.height
                - (platform.y * self.config.player.size);
            let width = (platform.width * self.config.player.size) + 1.0;
            let height = (platform.height * self.config.player.size) + 1.0;
            self.world.spawn((
                graphics::Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect::new(0.0, 0.0, width, height),
                    graphics::BLACK,
                )?,
                ZOrder(20),
                BoundingBox(Rect::new(x, y, width, height)),
                LevelId(current_level_atom.clone()),
            ));
        }

        for trap in &self.levels[&self.current_level].traps {
            let x = trap.x * self.config.player.size;
            let y =
                self.levels[&self.current_level].size.height - (trap.y * self.config.player.size);
            let width = (trap.width * self.config.player.size) + 1.0;
            let height = (trap.height * self.config.player.size) + 1.0;
            self.world.spawn((
                graphics::Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect::new(0.0, 0.0, width, height),
                    Color::from_rgb(255, 0, 0),
                )?,
                ZOrder(20),
                BoundingBox(Rect::new(x, y, width, height)),
                Death,
                LevelId(current_level_atom.clone()),
            ));
        }

        for teleporter in &self.levels[&self.current_level].teleporters {
            let x = teleporter.x * self.config.player.size;
            let y = self.levels[&self.current_level].size.height
                - (teleporter.y * self.config.player.size);
            let width = (teleporter.width * self.config.player.size) + 1.0;
            let height = (teleporter.height * self.config.player.size) + 1.0;

            self.world.spawn((
                graphics::Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect::new(0.0, 0.0, width, height),
                    Color::from_rgb(0, 255, 0),
                )?,
                ZOrder(20),
                BoundingBox(Rect::new(x, y, width, height)),
                LevelId(current_level_atom.clone()),
                TeleportTo(teleporter.target.clone()),
            ));
        }

        for text in &self.levels[&self.current_level].texts {
            let x = text.x * self.config.player.size;
            let y =
                self.levels[&self.current_level].size.height - (text.y * self.config.player.size);
            // let width = (text.width * self.config.player.size) + 1.0;
            // let height = (text.height * self.config.player.size) + 1.0;

            self.world.spawn((
                Position::new(x, y),
                TextContainer {
                    value: text.value.clone(),
                    size: text.size,
                },
                Color::from_rgb(text.color.red, text.color.green, text.color.blue),
                LevelId(current_level_atom.clone()),
            ));
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Platform {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Default for TextColor {
    fn default() -> Self {
        TextColor {
            red: 0,
            green: 0,
            blue: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelText {
    pub x: f32,
    pub y: f32,
    pub value: String,
    pub size: f32,
    #[serde(default)]
    pub color: TextColor,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Teleporter {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub target: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Start {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Level {
    pub size: LevelSize,
    pub start: Start,
    #[serde(rename = "platform")]
    pub platforms: Vec<Platform>,
    #[serde(rename = "trap")]
    #[serde(default)]
    pub traps: Vec<Platform>,
    #[serde(rename = "teleporter")]
    #[serde(default)]
    pub teleporters: Vec<Teleporter>,
    #[serde(rename = "text")]
    #[serde(default)]
    pub texts: Vec<LevelText>,
}

impl Default for Level {
    fn default() -> Self {
        Level {
            size: LevelSize {
                width: 10_000.0,
                height: 10_000.0,
            },
            start: Start { x: 2.0, y: 1.0 },
            platforms: vec![
                Platform {
                    x: 0.0,
                    y: 2.0,
                    width: 4.0,
                    height: 1.0,
                },
                Platform {
                    x: 6.0,
                    y: 8.0,
                    width: 4.0,
                    height: 1.0,
                },
                Platform {
                    x: 12.0,
                    y: 14.0,
                    width: 4.0,
                    height: 1.0,
                },
            ],
            traps: vec![Platform {
                x: 4.0,
                y: 6.0,
                width: 4.0,
                height: 1.0,
            }],
            teleporters: vec![],
            texts: vec![],
        }
    }
}
