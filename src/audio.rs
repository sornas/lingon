use luminance_sdl2::sdl2::Sdl;
use luminance_sdl2::sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

pub struct Audio {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl Audio {
    pub fn init(sdl: &Sdl) -> AudioDevice<Self> {
        let audio_subsystem = sdl.audio().unwrap();
        let desired = AudioSpecDesired {
            freq: Some(48000),
            channels: Some(1),
            samples: None,
        };

        audio_subsystem.open_playback(None, &desired, |spec| {
            Self {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        }).unwrap()
    }
}

impl AudioCallback for Audio {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
