use sungod::Ra;

/// Takes a lower and upper bound and randomly selects values in-between.
pub struct RandomProperty {
    pub distribution: Box<dyn Distribute>,
    pub range: [f32; 2],
}

impl Default for RandomProperty {
    fn default() -> Self {
        Self {
            distribution: Box::new(Uniform),
            range: [0.0, 1.0],
        }
    }
}

impl RandomProperty {
    pub fn new(lo: f32, hi: f32, distribution: Box<dyn Distribute>) -> Self {
        Self {
            distribution,
            range: [lo, hi],
        }
    }

    /// Samples a random value in the given range.
    pub fn sample(&self) -> f32 {
        self.distribution.between(self.range[0], self.range[1])
    }
}

pub trait Distribute {
    /// Get a random value between 0.0 and 1.0.
    fn sample(&self) -> f32;

    /// Get a random value between two endpoints.
    fn between(&self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.sample()
    }
}

/// Always returns the lowest value.
pub struct NoDice;

impl Distribute for NoDice {
    fn sample(&self) -> f32 {
        0.0
    }
}

/// All values are equally likely with no bias.
pub struct Uniform;

impl Distribute for Uniform {
    fn sample(&self) -> f32 {
        Ra::ggen::<f32>()
    }
}

/// Biased towards the middle. Looks like a triangle.
pub struct TwoDice;

impl Distribute for TwoDice {
    fn sample(&self) -> f32 {
        (Ra::ggen::<f32>() + Ra::ggen::<f32>()) / 2.0
    }
}

/// Biased towards the middle. Looks like a bellcurve.
pub struct ThreeDice;

impl Distribute for ThreeDice {
    fn sample(&self) -> f32 {
        (Ra::ggen::<f32>() + Ra::ggen::<f32>() + Ra::ggen::<f32>()) / 3.0
    }
}

/// Biased towards the lowest value. Looks like 1/x.
pub struct Square;

impl Distribute for Square {
    fn sample(&self) -> f32 {
        Ra::ggen::<f32>() * Ra::ggen::<f32>()
    }
}
