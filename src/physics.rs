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
    pub fn apply_physics(&mut self, _ctx: &mut Context) {
        for (_id, (acceleration, velocity, position, mass, gravity, size)) in
            &mut self.world.query::<(
                &mut Acceleration,
                &mut Velocity,
                &mut Position,
                &Mass,
                &Gravity,
                &Size,
            )>()
        {
            acceleration.apply_gravity(&gravity.0);

            for BoundingBox(bbox) in self
                .world
                .query::<&BoundingBox>()
                .iter()
                .filter(|BoundingBox(bbox)| bbox.contains(position.0))
            {}

            if position.is_grounded() {
                let mut friction = velocity.0.clone();
                friction *= -1.0;
                friction = friction.normalize_safe();
                friction *= self.config.physics.friction * self.config.physics.normal_force;
                acceleration.apply_force(&friction, mass.0);
            }

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

            let half_size = size.0 / 2.0;
            let max_x = WORLD_WIDTH - half_size;
            let min_x = half_size;

            // stop
            if position.0.x >= max_x {
                position.0.x = max_x;
                velocity.0.x = 0.0;
            } else if position.0.x <= min_x {
                position.0.x = min_x;
                velocity.0.x = 0.0;
            }

            let max_y = WORLD_HEIGHT - half_size;
            let min_y = half_size;

            if position.0.y >= FLOOR {
                position.0.y = FLOOR;
                velocity.0.y = 0.0;
            } else if position.0.y >= max_y {
                position.0.y = max_y;
                velocity.0.y = 0.0;
            } else if position.0.y <= min_y {
                position.0.y = min_y;
                velocity.0.y = 0.0;
            }
        }
    }
}
