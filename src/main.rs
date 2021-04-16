use luminance_sdl2::GL33Surface;
use std::path::Path;
use std::time::Instant;

use crate::random::{Distribute, RandomProperty};

mod asset;
mod input;
mod random;
mod renderer;

fn main() {
    let surface = GL33Surface::build_with(|video| video.window("game", 800, 600))
        .expect("Failed to create surface");

    main_loop(surface);
}

fn main_loop(mut surface: GL33Surface) {
    let mut coin_i = 0;
    const COINS: &[&str] = &["res/coin-gold.png", "res/coin-red.png"];

    use renderer::*;
    let mut sampler = luminance::texture::Sampler::default();
    sampler.mag_filter = luminance::texture::MagFilter::Nearest;
    let mut renderer = Renderer::new(&mut surface, sampler);

    let image = asset::Image::load(Path::new("res/coin-gold.png").to_path_buf());
    let mut sheet = renderer.add_sprite_sheet(image, (16, 16));

    let mut particle_systems = ParticleSystem::new();
    particle_systems.lifetime = RandomProperty::new(1.0, 2.0);
    particle_systems.start_sx = RandomProperty::new(0.01, 0.015);
    particle_systems.start_sy = RandomProperty::new(0.01, 0.015);
    particle_systems.end_sx = RandomProperty::new(0.0, 0.0);
    particle_systems.end_sy = RandomProperty::new(0.0, 0.0);
    particle_systems.v_angle = RandomProperty::new(-std::f32::consts::PI, std::f32::consts::PI);
    particle_systems.v_magnitude = RandomProperty::new(-2.0, 2.0);
    particle_systems.acceleration_angle =
        RandomProperty::new(-std::f32::consts::PI, std::f32::consts::PI);
    particle_systems.acceleration_magnitude = RandomProperty::new(0.2, 0.8);
    particle_systems.angle = RandomProperty::new(-2.0, 2.0);
    particle_systems.angle_velocity = RandomProperty::new(-2.0, 2.0);
    particle_systems.angle_drag = RandomProperty::new(0.0, 2.0);

    let start_t = Instant::now();

    let mut input = input::InputManager::new(surface.sdl());
    input.bind(input::Device::Key(input::Keycode::A), input::Name::Left);
    input.bind(input::Device::Key(input::Keycode::D), input::Name::Right);
    input.bind(input::Device::Key(input::Keycode::W), input::Name::Up);
    input.bind(input::Device::Key(input::Keycode::S), input::Name::Down);
    input.bind(
        input::Device::Key(input::Keycode::Escape),
        input::Name::Quit,
    );
    input.bind(input::Device::Quit, input::Name::Quit);
    input.bind(input::Device::Axis(0, input::Axis::LeftX), input::Name::Right);
    input.bind(input::Device::Axis(0, input::Axis::RightY), input::Name::Up);

    input.bind(input::Device::Key(input::Keycode::R), input::Name::NextCoin);

    let mut old_t = start_t.elapsed().as_millis() as f32 * 1e-3;
    'app: loop {
        let t = start_t.elapsed().as_millis() as f32 * 1e-3;
        let delta = t - old_t;
        old_t = t;

        input.poll(surface.sdl());
        if input.pressed(input::Name::Quit) {
            break 'app;
        }

        if input.pressed(input::Name::NextCoin) {
            coin_i = (coin_i + 1) % COINS.len();

            let image = asset::Image::load(Path::new(COINS[coin_i]).to_path_buf());
            sheet = renderer.add_sprite_sheet(image, (16, 16));
        }

        particle_systems.position[0] = t.cos() * 0.5;
        particle_systems.position[1] = t.sin() * 0.5;
        for _ in 0..5 {
            particle_systems.spawn();
        }
        particle_systems.update(delta);

        let region = renderer.sprite_sheets[sheet].grid([0, 1, 2, 3, 2, 1][((t * 10.0) as usize) % 6], 0);
        for x in -5..5 {
            for y in -5..5 {
                renderer.push(
                    Sprite::new(region)
                        .at(x as f32, y as f32)
                        .scale(0.3, 0.3)
                        .angle(t),
                );
            }
        }

        const NUM_BUCKETS: usize = 100;
        let mut buckets = [0; NUM_BUCKETS];
        for _ in 0..10000 {
            let sample = random::Square.sample();
            buckets[(sample * (NUM_BUCKETS as f32)) as usize] += 1;
        }

        for (i, v) in buckets.iter().enumerate() {
            let w = 1.0 / (NUM_BUCKETS as f32);
            let h = (*v as f32) * w * 0.1;
            renderer.push(Rect::new().scale(w, h).at((i as f32) * w, h / 2.0));
        }

        renderer.push_particle_system(&particle_systems);
        //input.rumble(0, input.value(input::Name::Right), input.value(input::Name::Up), 1.0).unwrap();
        renderer.camera.move_by(
            (input.value(input::Name::Right) - input.value(input::Name::Left)) * delta,
            (input.value(input::Name::Up) - input.value(input::Name::Down)) * delta,
        );

        if renderer.render(&mut surface).is_err() {
            break 'app;
        }
    }
}
