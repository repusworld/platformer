use crate::common::*;
use crate::components::*;

impl GameState {
    #[inline(always)]
    pub fn move_camera(&mut self, _ctx: &mut Context) -> GameResult<()> {
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
            } else if self.camera.center.x > self.level_size.width - (WIDTH / 2.0) {
                self.camera.center.x = self.level_size.width - (WIDTH / 2.0)
            }
            if self.camera.center.y < HEIGHT / 2.0 {
                self.camera.center.y = HEIGHT / 2.0;
            } else if self.camera.center.y > self.level_size.height - (HEIGHT / 2.0) {
                self.camera.center.y = self.level_size.height - (HEIGHT / 2.0);
            }
        }
        Ok(())
    }
}
