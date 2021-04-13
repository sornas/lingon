use super::{SpriteRegion, prelude::*};

use sungod::Ra;

use crate::random::RandomProperty;

/// An actual particle system. Contains a lot of
/// knobs.
///
/// Particles are rendered only on the GPU and as such are 'almost' free.
#[derive(Default)]
pub struct ParticleSystem {
    pub time: f32,
    pub particles: Vec<Particle>,

    // TODO(ed): GRR!! I want this to be Vector2
    // and implement Transform
    pub position: [f32; 2],

    pub sprites: Vec<SpriteRegion>,

    /// Allowed x-coordinates to spawn on, relative to 'position'.
    pub x: RandomProperty,
    /// Allowed y-coordinates to spawn on, relative to 'position'.
    pub y: RandomProperty,

    /// How long, in seconds, the particle should live.
    pub lifetime: RandomProperty,

    // TODO(ed): Options for how this is selected
    /// The angle of the velocity in radians.
    pub v_angle: RandomProperty,
    /// How fast a particle should move when it spawns.
    pub v_magnitude: RandomProperty,

    /// What direction to accelerate in.
    pub acceleration_angle: RandomProperty,
    /// How strong the acceleration is in that direction.
    pub acceleration_magnitude: RandomProperty,

    /// A fake 'air-resistance'. Lower values mean less resistance.
    /// Negative values give energy over time.
    pub drag: RandomProperty,

    /// The rotation to spawn with.
    pub angle: RandomProperty,
    /// How fast the angle should change when the particle spawns.
    pub angle_velocity: RandomProperty,
    /// A fake 'energy-loss'. Lower values mean less resistance.
    /// Negative values give energy over time.
    pub angle_drag: RandomProperty,

    /// How large the particle should be in X when it starts.
    pub start_sx: RandomProperty,
    /// How large the particle should be in Y when it starts.
    pub start_sy: RandomProperty,

    /// How large the particle should be in X when it dies.
    pub end_sx: RandomProperty,
    /// How large the particle should be in Y when it dies.
    pub end_sy: RandomProperty,

    /// How red the particle should be when it spawns.
    pub start_red: RandomProperty,
    /// How green the particle should be when it spawns.
    pub start_green: RandomProperty,
    /// How blue the particle should be when it spawns.
    pub start_blue: RandomProperty,
    /// How transparent the particle should be when it spawns.
    pub start_alpha: RandomProperty,

    /// How red the particle should be when it dies.
    pub end_red: RandomProperty,
    /// How green the particle should be when it dies.
    pub end_green: RandomProperty,
    /// How blue the particle should be when it dies.
    pub end_blue: RandomProperty,
    /// How transparent the particle should be when it dies.
    pub end_alpha: RandomProperty,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            particles: Vec::new(),

            x: RandomProperty::new(-0.1, 0.1),
            y: RandomProperty::new(-0.1, 0.1),

            v_angle: RandomProperty::new(0.0, 2.0 * std::f32::consts::PI),

            acceleration_angle: RandomProperty::new(0.0, 2.0 * std::f32::consts::PI),

            start_sx: RandomProperty::new(1.0, 1.0),
            start_sy: RandomProperty::new(1.0, 1.0),

            ..Self::default()
        }
    }

    /// Steps the particle system some delta-time forward. Removes dead particles.
    pub fn update(&mut self, delta: f32) {
        self.time += delta;

        self.particles = std::mem::take(&mut self.particles)
            .into_iter()
            .filter(|x| *x.lifetime > (self.time - *x.spawn))
            .collect();
    }

    /// Spawns a new particle.
    pub fn spawn(&mut self) {
        let velocity_angle = self.v_angle.sample();
        let velocity_magnitude = self.v_magnitude.sample();

        let acceleration_angle = self.acceleration_angle.sample();
        let acceleration_magnitude = self.acceleration_magnitude.sample();

        let (sheet, uv) = if self.sprites.is_empty() {
            &(-1.0, [0.0, 0.0, 0.0, 0.0])
        } else {
            let i = Ra::ggen::<usize>();
            let i = i % self.sprites.len();
            &self.sprites[i]
        };

        self.particles.push(Particle {
            spawn: PSpawn::new(self.time),
            lifetime: PLifetime::new(self.lifetime.sample()),

            position: IPosition::new([
                self.x.sample() + self.position[0],
                self.y.sample() + self.position[1],
            ]),
            velocity: PVelocity::new([
                velocity_angle.cos() * velocity_magnitude,
                velocity_angle.sin() * velocity_magnitude,
            ]),
            acceleration: PAcceleration::new([
                acceleration_angle.cos() * acceleration_magnitude,
                acceleration_angle.sin() * acceleration_magnitude,
            ]),
            drag: PDrag::new(self.drag.sample()),

            angle_info: PAngleInfo::new([
                self.angle.sample(),
                self.angle_velocity.sample(),
                self.angle_drag.sample(),
            ]),

            scale_extrems: PScaleExtrems::new([
                self.start_sx.sample(),
                self.start_sy.sample(),
                self.end_sx.sample(),
                self.end_sy.sample(),
            ]),

            start_color: PStartColor::new([
                self.start_red.sample(),
                self.start_green.sample(),
                self.start_blue.sample(),
                self.start_alpha.sample(),
            ]),
            end_color: PEndColor::new([
                self.end_red.sample(),
                self.end_green.sample(),
                self.end_blue.sample(),
                self.end_alpha.sample(),
            ]),

            sheet: ISheet::new(*sheet),
            uv: IUV::new(*uv),
        });
    }

    /// Copies out the rendering information.
    pub fn freeze(&self) -> FrozenParticles {
        // TODO(ed): Can we get rid of this clone?
        FrozenParticles {
            position: self.position,
            time: self.time,
            particles: self.particles.clone(),
        }
    }
}

/// A particle system that can be rendered.
/// Used internally.
pub struct FrozenParticles {
    pub position: [f32; 2],
    pub time: f32,
    pub particles: Vec<Particle>,
}
