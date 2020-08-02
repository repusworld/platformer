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
    pub platforms: Vec<Platform>,
}
