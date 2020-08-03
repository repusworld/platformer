#[derive(Serialize, Deserialize, Clone)]
pub struct Platform {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Start {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LevelSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Level {
    pub size: LevelSize,
    pub start: Start,
    #[serde(rename = "platform")]
    pub platforms: Vec<Platform>,
    #[serde(rename = "trap")]
    pub traps: Vec<Platform>,
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
                y: 3.0,
                width: 4.0,
                height: 1.0,
            }],
        }
    }
}
