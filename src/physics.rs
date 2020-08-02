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
                let mut friction = velocity.0.clone();
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
    pub fn collision_detection(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut grounded_entities = vec![];
        for (id, (velocity, position, &BoundingBox(bbox))) in
            &mut self
                .world
                .query::<(&mut Velocity, &mut Position, &BoundingBox)>()
        {
            let mut bbox = bbox.clone();
            bbox.translate(Vector2::new(position.0.x, position.0.y));
            for (_other, BoundingBox(other)) in self
                .world
                .query::<&BoundingBox>()
                .iter()
                .filter(|(other, _)| id != *other)
                .filter(|(_, BoundingBox(other))| other.overlaps(&bbox))
            {
                if bbox.bottom() >= other.top() {
                    velocity.0.y = 0.0;
                    position.0.y = other.top();
                    grounded_entities.push(id);
                }
            }

            let half_size = bbox.w / 2.0;
            let max_x = self.level_size.width - half_size;
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
            let max_y = self.level_size.height - half_size;
            let min_y = half_size;

            if position.0.y >= max_y {
                position.0.y = max_y;
                velocity.0.y = 0.0;
                ggez::event::quit(ctx);
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
