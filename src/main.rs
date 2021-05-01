use std::f32::consts::PI;
use std::path::Path;

use lingon::audio::AudioSource;
use lingon::input;
use lingon::random::{self, Distribute, RandomProperty};
use lingon::renderer::{ParticleSystem, Rect, Sprite, Transform};

/// A list of all valid inputs.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Name {
    Left,
    Right,
    Up,
    Down,
    PlaySound,
    Quit,
}

fn bind_inputs(game: &mut lingon::Game<Name>) {
    game.input.bind(input::Device::Key(input::Keycode::A), Name::Left);
    game.input.bind(input::Device::Key(input::Keycode::D), Name::Right);
    game.input.bind(input::Device::Key(input::Keycode::W), Name::Up);
    game.input.bind(input::Device::Key(input::Keycode::S), Name::Down);
    game.input.bind(input::Device::Key(input::Keycode::Escape), Name::Quit);
    game.input.bind(input::Device::Key(input::Keycode::F), Name::PlaySound);
    game.input.bind(input::Device::Quit, Name::Quit);
    game.input.bind(input::Device::Axis(0, input::Axis::LeftX), Name::Right);
    game.input.bind(input::Device::Axis(0, input::Axis::RightY), Name::Up);
}

fn main() {
    // Create the initial game state and input manager.
    let mut game = lingon::Game::new("game", 800, 600);
    bind_inputs(&mut game);
    *game.audio.lock().gain_mut() = 0.5;
    game.set_window_icon("res/transparent.png");

    // Load an image and a sound.
    let transparent = game.assets.load_image(Path::new("res/transparent.png").to_path_buf());
    let bloop = game.assets.load_audio(Path::new("res/bloop.wav").to_path_buf());
    let bloop = AudioSource::new(&game.assets[bloop])
        .gain(0.3)
        .gain_variance(0.2)
        .pitch(1.5)
        .pitch_variance(0.2);

    // Add our image as a sprite sheet.
    let transparent_sheet = game.renderer.add_sprite_sheet(game.assets[transparent].clone(), (32, 32));

    // Create a particle system.
    let mut particle_system = lingon::particle_system!(
        lifetime       = [1.0, 2.0]    random::TwoDice,
        start_sx       = [0.01, 0.015] random::TwoDice,
        start_sy       = [0.01, 0.015] random::TwoDice,
        end_sx         = [0.0, 0.0]    random::TwoDice,
        end_sy         = [0.0, 0.0]    random::TwoDice,
        vel_angle      = [-PI, PI]     random::TwoDice,
        vel_magnitude  = [-2.0, 2.0]   random::TwoDice,
        acc_angle      = [-PI, PI]     random::TwoDice,
        acc_magnitude  = [0.2, 0.8]    random::TwoDice,
        angle          = [-2.0, 2.0]   random::TwoDice,
        angle_velocity = [-2.0, 2.0]   random::TwoDice,
        angle_drag     = [0.0, 2.0]    random::TwoDice,
    );

    let mut i = 0;

    'main: loop {
        // Go a step forward in time.
        let delta = game.time_tick();
        game.update(delta);

        // Poll input and time it.
        let timer = lingon::counter!("input");
        drop(timer);

        if game.input.pressed(Name::Quit) {
            break 'main;
        }

        if game.input.pressed(Name::PlaySound) {
            // Play an audio asset.
            game.audio.lock().play(bloop.clone());
            game.center_window();
            game.set_window_title(&format!("{} {}", i, i*2)).unwrap();
            i += 1;
        }

        if game.input.pressed(Name::Left) {
            let (sx, sy) = game.window_size();
            game.set_window_size(sx - 10, sy).unwrap();
        }

        if game.input.pressed(Name::Right) {
            let(x, y) = game.window_position();
            game.set_forced_window_position(x, y + 10)
        }

        // Move the particle system in a circle. One revolution takes 2*PI seconds.
        particle_system.position[0] = game.total_time().cos() * 0.5;
        particle_system.position[1] = game.total_time().sin() * 0.5;

        // Spawn five particles each frame.
        for _ in 0..5 {
            particle_system.spawn();
        }

        // Simulate the particle system.
        particle_system.update(delta);

        // Get a region of the previously added sprite sheet.
        // The time-dependence effectively creates an animation.
        let region = game.renderer.sprite_sheets[transparent_sheet].grid(
                0,
                0
        );

        // Draw the selected coin sprite in a table layout.
        for x in -5..5 {
            for y in -5..5 {
                game.renderer.push(
                    Sprite::new(region)
                        .at(x as f32, y as f32)
                        .scale(1.0, 1.0)
                        // Also, spin them around! :D
                        .angle(game.total_time()),
                );
            }
        }

        // Simulate a Square distribution...
        const NUM_BUCKETS: usize = 100;
        let mut buckets = [0; NUM_BUCKETS];
        for _ in 0..10000 {
            let sample = random::Square.sample();
            buckets[(sample * (NUM_BUCKETS as f32)) as usize] += 1;
        }

        // ... by drawing rectangles that are scaled according to how likely the value was.
        for (i, v) in buckets.iter().enumerate() {
            let w = 1.0 / (NUM_BUCKETS as f32);
            let h = (*v as f32) * w * 0.1;
            game.renderer.push(Rect::new().scale(w, h).at((i as f32) * w, h / 2.0));
        }

        // Tell the renderer to draw the particle system.
        game.renderer.push_particle_system(&particle_system);

        // Tell the renderer to move the camera.
        game.renderer.camera.move_by(
            (game.input.value(Name::Right) - game.input.value(Name::Left)) * delta,
            (game.input.value(Name::Up) - game.input.value(Name::Down)) * delta,
        );

        // Draw this frame.
        if game.draw().is_err() {
            break 'main;
        }
    }
}
