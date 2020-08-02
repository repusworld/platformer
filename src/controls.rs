use ggez::event::{quit, KeyCode, KeyMods};

use crate::common::*;
use crate::components::*;
use crate::physics::*;

impl GameState {
    #[inline(always)]
    pub fn do_movement(&mut self, _ctx: &mut Context) -> GameResult<()> {
        for (id, (acceleration, gravity, velocity, mass, _)) in
            &mut self
                .world
                .query::<(&mut Acceleration, &mut Gravity, &Velocity, &Mass, &Player)>()
        {
            if self.controls.debug_pressed {
                self.config.debug.draw_bounds = !self.config.debug.draw_bounds;
                self.config.debug.draw_grid = !self.config.debug.draw_grid;
            }

            let grounded = self
                .world
                .query::<&Grounded>()
                .iter()
                .filter(|(other, _)| id == *other)
                .any(|(_, grounded)| grounded.0);

            if grounded || self.config.player.allow_air_control {
                if self.controls.left_held {
                    acceleration
                        .apply_force(&Vector2::new(-self.config.player.acceleration, 0.0), mass.0);
                }

                if self.controls.right_held {
                    acceleration
                        .apply_force(&Vector2::new(self.config.player.acceleration, 0.0), mass.0);
                }
            }

            if grounded {
                if self.controls.jump_pressed {
                    let mag = velocity.0.magnitude();
                    acceleration.apply_force(
                        &Vector2::new(
                            0.0,
                            -self.config.player.jump_acceleration * (1.0 + (mag / 30.0)),
                        ),
                        mass.0,
                    );
                }
            }

            if self.controls.jump_held {
                gravity.0.y = self.config.physics.gravity * self.config.player.float_modifier;
            } else {
                gravity.0.y = self.config.physics.gravity;
            }
        }
        Ok(())
    }

    #[inline(always)]
    pub fn map_key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        repeat: bool,
    ) {
        match keycode {
            KeyCode::Escape => quit(ctx),
            KeyCode::A | KeyCode::Left => {
                self.controls.left_pressed = !repeat;
                self.controls.left_held = true;
            }
            KeyCode::D | KeyCode::Right => {
                self.controls.right_pressed = !repeat;
                self.controls.right_held = true;
            }
            KeyCode::W /*| KeyCode::Up*/ => {
                self.controls.up_pressed = !repeat;
                self.controls.up_held = true;
            }
            KeyCode::S | KeyCode::Down => {
                self.controls.down_pressed = !repeat;
                self.controls.down_held = true;
            }
            KeyCode::Space | KeyCode::Up => {
                self.controls.jump_pressed = !repeat;
                self.controls.jump_held = true;
            }
            KeyCode::F12 => {
                self.controls.debug_pressed = !repeat;
                self.controls.debug_held = true;
            }
            _ => (),
        }
    }

    #[inline(always)]
    pub fn map_key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::A | KeyCode::Left => {
                self.controls.left_pressed = false;
                self.controls.left_held = false;
            }
            KeyCode::D | KeyCode::Right => {
                self.controls.right_pressed = false;
                self.controls.right_held = false;
            }
            KeyCode::W | KeyCode::Up => {
                self.controls.up_pressed = false;
                self.controls.up_held = false;
            }
            KeyCode::S | KeyCode::Down => {
                self.controls.down_pressed = false;
                self.controls.down_held = false;
            }
            KeyCode::Space => {
                self.controls.jump_pressed = false;
                self.controls.jump_held = false;
            }
            KeyCode::F12 => {
                self.controls.debug_pressed = false;
                self.controls.debug_held = false;
            }
            _ => (),
        }
    }

    #[inline(always)]
    pub fn reset_pressed_state(&mut self) {
        self.controls.left_pressed = false;
        self.controls.right_pressed = false;
        self.controls.up_pressed = false;
        self.controls.down_pressed = false;
        self.controls.jump_pressed = false;
        self.controls.debug_pressed = false;
    }
}
