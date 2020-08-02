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
            size: 32.0,
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
    pub draw_bounds: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        DebugConfig {
            draw_grid: false,
            draw_bounds: false,
        }
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
