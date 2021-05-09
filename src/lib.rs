use luminance_sdl2::sdl2::{self, IntegerOrSdlError, surface::Surface, video::WindowPos};
use luminance_sdl2::GL33Surface;
use sdl2::audio::AudioDevice;
use sdl2::Sdl;
use std::{ffi::NulError, hash::Hash, path::Path};
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
    delta: f32,
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
            delta: 0.0,
            prev_t: 0.0,
        }
    }

    pub fn update(&mut self) {
        let t = self.start_t.elapsed().as_millis() as f32 * 1e-3;
        self.delta = t - self.prev_t;
        self.prev_t = t;

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

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn total_time(&self) -> f32 {
        self.prev_t
    }

    pub fn window_size(&self) -> (u32, u32) {
        self.surface.window().size()
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) -> Result<(), IntegerOrSdlError> {
        self.surface.window_mut().set_size(width, height)
    }

    pub fn window_position(&self) -> (i32, i32) {
        self.surface.window().position()
    }

    pub fn set_forced_window_position(&mut self, x: i32, y: i32) {
        self.surface.window_mut().set_position(
            WindowPos::Positioned(x),
            WindowPos::Positioned(y),
        );
    }

    pub fn center_window(&mut self) {
        self.surface.window_mut().set_position(
            WindowPos::Centered,
            WindowPos::Centered,
        );
    }

    pub fn window_title(&self) -> &str {
        self.surface.window().title()
    }

    pub fn set_window_title(&mut self, title: &str) -> Result<(), NulError> {
        self.surface.window_mut().set_title(title)
    }

    pub fn set_window_icon<P: AsRef<Path>>(&mut self, path: P) {
        let mut icon = asset::Image::new(path.as_ref().to_path_buf());
        let icon_surface = Surface::from_data(
            &mut icon.texture_data,
            icon.width as u32,
            icon.height as u32,
            icon.width as u32 * 4,
            sdl2::pixels::PixelFormatEnum::RGBA8888,
        ).unwrap();
        self.surface.window_mut().set_icon(icon_surface);
    }
}
