use ggez::GameResult;

use crate::common::*;
use crate::components::*;

pub trait SafeNormalization {
    fn normalize_safe(&self) -> Self;
}

impl SafeNormalization for Vector2 {
    fn normalize_safe(&self) -> Self {
        self.try_normalize(f32::EPSILON)
            .unwrap_or_else(|| Vector2::new(0.0, 0.0))
    }
}

#[inline(always)]
fn limit(v: f32, max: f32) -> f32 {
    if v.abs() > max {
        max * v.signum()
    } else {
        v
    }
}

pub trait PhysicsHelper {
    fn apply_force(&mut self, force: &Vector2, mass: f32) -> &mut Self;

    fn apply_gravity(&mut self, force: &Vector2) -> &mut Self;
}

pub fn apply_force(v: &mut Vector2, force: &Vector2, mass: f32) {
    let force = force / mass;
    *v += force;
}

pub fn apply_gravity(v: &mut Vector2, force: &Vector2) {
    *v += force;
}

impl GameState {
    #[inline(always)]
    pub fn apply_physics(&mut self, _ctx: &mut Context) -> GameResult<()> {
        for (_id, (acceleration, velocity, mass, grounded)) in
            &mut self
                .world
                .query::<(&mut Acceleration, &mut Velocity, &Mass, &Grounded)>()
        {
            if grounded.0 {
                let mut friction = velocity.0;
                friction *= -1.0;
                friction = friction.normalize_safe();
                friction *= self.config.physics.friction * self.config.physics.normal_force;
                acceleration.apply_force(&friction, mass.0);
            }
        }

        for (_id, (acceleration, velocity, position, gravity)) in
            &mut self
                .world
                .query::<(&mut Acceleration, &mut Velocity, &mut Position, &Gravity)>()
        {
            acceleration.apply_gravity(&gravity.0);

            // apply acceleration
            velocity.0 += acceleration.0;

            acceleration.0 *= 0.0;

            // limit velocity
            velocity.0.x = limit(velocity.0.x, self.config.physics.max_horizontal_velocity);
            velocity.0.y = limit(velocity.0.y, self.config.physics.max_vertical_velocity);

            if velocity.0.x.abs() < self.config.physics.movement_deadzone {
                velocity.0.x = 0.0;
            }

            if velocity.0.y.abs() < self.config.physics.movement_deadzone {
                velocity.0.y = 0.0;
            }

            // apply velocity
            position.0 += velocity.0;
        }

        Ok(())
    }

    #[inline(always)]
    pub fn collision_detection(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let mut grounded_entities = vec![];
        for (id, (velocity, position, &BoundingBox(bbox))) in
            &mut self
                .world
                .query::<(&mut Velocity, &mut Position, &BoundingBox)>()
        {
            let mut bbox = bbox;
            bbox.translate(Vector2::new(position.0.x, position.0.y));

            for (other, BoundingBox(other_bbox)) in self
                .world
                .query::<&BoundingBox>()
                .iter()
                .filter(|(other, _)| id != *other)
                .filter(|(_, BoundingBox(other))| other.overlaps(&bbox))
            {
                if let Ok(mut q) = self.world.query_one::<&Death>(other) {
                    if q.get().is_some() {
                        self.restart_level = true;
                    }
                }

                if let Ok(mut q) = self.world.query_one::<&TeleportTo>(other) {
                    self.change_level = q.get().map(|tele| tele.0.clone());
                }

                let bbox_left = bbox.left();
                let bbox_right = bbox.right();
                let bbox_top = bbox.top();
                let bbox_bottom = bbox.bottom();

                let top_left = Point2::new(bbox_left, bbox_top);
                let top_right = Point2::new(bbox_right, bbox_top);
                let bottom_left = Point2::new(bbox_left, bbox_bottom);
                let bottom_right = Point2::new(bbox_right, bbox_bottom);

                let bl = other_bbox.contains(bottom_left);
                let br = other_bbox.contains(bottom_right);

                let tl = other_bbox.contains(top_left);
                let tr = other_bbox.contains(top_right);

                if bl && br {
                    velocity.0.y = 0.0;
                    position.0.y = other_bbox.top();
                    grounded_entities.push(id);
                } else if tl && tr {
                    velocity.0.y /= 2.0;
                    position.0.y = other_bbox.bottom() + self.config.player.size;
                } else if tl && bl {
                    velocity.0.x = 0.0;
                    position.0.x = other_bbox.right() + (self.config.player.size / 2.0);
                } else if tr && br {
                    velocity.0.x = 0.0;
                    position.0.x = other_bbox.left() - (self.config.player.size / 2.0);
                } else {
                    // TODO: simplify this mess
                    if bl {
                        let distance_y = bbox_bottom - other_bbox.top();
                        let distance_x = bbox_left - other_bbox.right();
                        if distance_y.abs() <= distance_x.abs() {
                            velocity.0.y = 0.0;
                            position.0.y = other_bbox.top();
                            grounded_entities.push(id);
                        } else {
                            velocity.0.x = 0.0;
                            position.0.x = other_bbox.right() + (self.config.player.size / 2.0);
                        }
                    } else if br {
                        let distance_y = bbox_bottom - other_bbox.top();
                        let distance_x = bbox_right - other_bbox.left();
                        if distance_y.abs() <= distance_x.abs() {
                            velocity.0.y = 0.0;
                            position.0.y = other_bbox.top();
                            grounded_entities.push(id);
                        } else {
                            velocity.0.x = 0.0;
                            position.0.x = other_bbox.left() - (self.config.player.size / 2.0);
                        }
                    } else if tl {
                        let distance_y = bbox_top - other_bbox.bottom();
                        let distance_x = bbox_left - other_bbox.right();
                        if distance_y.abs() <= distance_x.abs() {
                            velocity.0.y /= 2.0;
                            position.0.y = other_bbox.bottom() + self.config.player.size;
                        } else {
                            velocity.0.x = 0.0;
                            position.0.x = other_bbox.right() + (self.config.player.size / 2.0);
                        }
                    } else if tr {
                        let distance_y = bbox_top - other_bbox.bottom();
                        let distance_x = bbox_right - other_bbox.left();
                        if distance_y.abs() <= distance_x.abs() {
                            velocity.0.y /= 2.0;
                            position.0.y = other_bbox.bottom() + self.config.player.size;
                        } else {
                            velocity.0.x = 0.0;
                            position.0.x = other_bbox.left() - (self.config.player.size / 2.0);
                        }
                    }
                }
            }

            let half_size = bbox.w / 2.0;
            let max_x = self.levels[&self.current_level].size.width - half_size;
            let min_x = half_size;

            // stop
            if position.0.x >= max_x {
                position.0.x = max_x;
                velocity.0.x = 0.0;
            } else if position.0.x <= min_x {
                position.0.x = min_x;
                velocity.0.x = 0.0;
            }

            let half_size = bbox.h / 2.0;
            let max_y = self.levels[&self.current_level].size.height - half_size;
            let min_y = half_size;

            if position.0.y >= max_y {
                position.0.y = max_y;
                velocity.0.y = 0.0;
                self.restart_level = true;
            } else if position.0.y <= min_y {
                position.0.y = min_y;
                velocity.0.y = 0.0;
            }
        }

        for (id, grounded) in &mut self.world.query::<&mut Grounded>() {
            grounded.0 = grounded_entities.contains(&id);
        }

        for id in grounded_entities {
            let _ = self.world.insert_one(id, Grounded(true));
        }

        Ok(())
    }
}
