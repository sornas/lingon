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
    pub fn new(lo: f32, hi: f32) -> Self {
        Self {
            distribution: Box::new(ThreeDice),
            range: [lo, hi],
        }
    }

    /// Samples a random value in the given range.
    pub fn sample(&self) -> f32 {
        self.range[0] + (self.range[1] - self.range[0]) * self.distribution.sample()
    }
}

pub trait Distribute {
    /// Get a random value.
    fn sample(&self) -> f32;
}

/// Always returns 0.
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

/// Biased towards 0.5. Looks like a triangle.
pub struct TwoDice;

impl Distribute for TwoDice {
    fn sample(&self) -> f32 {
        (Ra::ggen::<f32>() + Ra::ggen::<f32>()) / 2.0
    }
}

/// Biased towards 0.5. Looks like a bellcurve.
pub struct ThreeDice;

impl Distribute for ThreeDice {
    fn sample(&self) -> f32 {
        (Ra::ggen::<f32>() + Ra::ggen::<f32>() + Ra::ggen::<f32>()) / 3.0
    }
}

/// Biased towards 0. Looks like 1/x.
pub struct Square;

impl Distribute for Square {
    fn sample(&self) -> f32 {
        Ra::ggen::<f32>() * Ra::ggen::<f32>()
    }
}
