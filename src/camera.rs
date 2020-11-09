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

            let half_width = WIDTH / 2.0;
            let half_height = WIDTH / 2.0;

            if self.camera.center.x < half_width {
                self.camera.center.x = half_width;
            } else if self.camera.center.x
                > self.levels[&self.current_level].size.width - half_width
            {
                self.camera.center.x = self.levels[&self.current_level].size.width - half_width
            }
            if self.camera.center.y < half_height {
                self.camera.center.y = half_height;
            } else if self.camera.center.y
                > self.levels[&self.current_level].size.height - half_height
            {
                self.camera.center.y =
                    self.levels[&self.current_level].size.height - half_height;
            }
        }
        Ok(())
    }
}
