use luminance_sdl2::sdl2;
use luminance_sdl2::GL33Surface;
use sdl2::audio::AudioDevice;
use sdl2::Sdl;
use std::hash::Hash;
use std::time::Instant;

pub mod audio;
pub mod asset;
pub mod input;
pub mod random;
pub mod renderer;
pub mod performance;

/// Everything you need to create a game.
pub struct Game<T> {
    pub audio: AudioDevice<audio::Audio>,
    pub assets: asset::AssetSystem,
    pub renderer: renderer::Renderer,
    pub input: input::InputManager<T>,

    surface: GL33Surface,
    start_t: Instant,
    prev_t: f32,
}

impl<T: Eq + Hash + Clone> Game<T> {
    pub fn new(title: &str, window_width: u32, window_height: u32) -> Self {
        let mut surface = GL33Surface::build_with(|video| video.window(title,
                                                                       window_width,
                                                                       window_height))
            .expect("Failed to create surface");

        let mut sampler = luminance::texture::Sampler::default();
        sampler.mag_filter = luminance::texture::MagFilter::Nearest;
        let renderer = renderer::Renderer::new(&mut surface, sampler);

        let audio = audio::Audio::init(surface.sdl());
        audio.resume();
        let assets = asset::AssetSystem::new();

        let input = input::InputManager::new(surface.sdl());

        Self {
            audio,
            assets,
            renderer,
            input,

            surface,
            start_t: Instant::now(),
            prev_t: 0.0,
        }
    }

    pub fn update(&mut self, _delta: f32) { 
        performance::frame();
        self.assets.reload();
        self.renderer.reload();
        self.input.poll(self.surface.sdl());
    }

    pub fn draw(&mut self) -> Result<(), ()> {
        self.renderer.render(&mut self.surface)
    }
    
    pub fn sdl(&self) -> &Sdl {
        self.surface.sdl()
    }

    pub fn time_tick(&mut self) -> f32 {
        let t = self.start_t.elapsed().as_millis() as f32 * 1e-3;
        let delta = t - self.prev_t;
        self.prev_t = t;
        delta
    }

    pub fn total_time(&self) -> f32 {
        self.prev_t
    }
}
