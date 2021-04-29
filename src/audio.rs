use crate::asset::{self, audio::Samples};

use luminance_sdl2::sdl2::Sdl;
use luminance_sdl2::sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

pub const SAMPLE_RATE: i32 = 48000;

macro_rules! impl_builder {
    ( $( $field:ident : $type:ty ),* $(,)? ) => {
        $(
            pub fn $field(mut self, $field: $type) -> Self {
                self.$field = $field;
                self
            }
        )*
    }
}

/// A sound that is playing or can be played.
pub struct AudioSource {
    /// Which specific sample we're currently on.
    position: usize,
    /// Whether we should loop when the sample is done.
    looping: bool,
    /// The actual samples.
    samples: Samples,

    gain: f32,
    //TODO Add this when we have `position: f32`.
    // pitch: f32,

    /// If we should remove this source when we get the opportunity.
    ///
    /// This gets set if
    /// a) the audio is done playing and it doesn't loop,
    /// b) it is requested by the user.
    remove: bool,
}

impl AudioSource {
    pub fn new(audio: &asset::Audio) -> Self {
        Self {
            position: 0,
            looping: false,
            samples: audio.samples(),
            gain: 1.0,
            remove: false,
        }
    }

    impl_builder!(
        looping: bool,
        gain: f32,
    );
}

/// The audio subsystem.
pub struct Audio {
    sources: Vec<AudioSource>, 
}

impl Audio {
    pub fn init(sdl: &Sdl) -> AudioDevice<Self> {
        let audio_subsystem = sdl.audio().unwrap();
        let desired = AudioSpecDesired {
            freq: Some(SAMPLE_RATE),
            channels: Some(2),
            samples: None,
        };

        audio_subsystem.open_playback(None, &desired, |spec| {
            assert_eq!(spec.freq, SAMPLE_RATE); //TODO handle differing sample rates gracefully
            Self {
                sources: Vec::new(),
            }
        }).unwrap()
    }

    /// Start playing a new sound once.
    ///
    /// This uses the "default" settings for the [AudioSource]. If you want to modify it, use
    /// [Audio::play_source].
    pub fn play_asset(&mut self, audio: &asset::Audio) {
        self.play_source(AudioSource::new(audio));
    }

    /// Start playing a new source.
    ///
    /// The source can be created via [AudioSource::new] and modified by builders on [AudioSource]
    /// (like [AudioSource::looping]).
    pub fn play_source(&mut self, source: AudioSource) {
        self.sources.push(source);
    }
}

impl AudioCallback for Audio {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        // Clear the buffer.
        for x in out.iter_mut() {
            *x = 0.0;
        }

        'sources: for source in self.sources.iter_mut() {
            let samples = source.samples.read().unwrap();
            for x in out.iter_mut() {
                // Move forward
                source.position += 1;
                if source.position >= samples.len() {
                    if source.looping {
                        source.position %= samples.len();
                    } else {
                        source.remove = true;
                        continue 'sources;
                    }
                }

                // Write data
                *x += samples[source.position] * source.gain;
            }
        }

        // Remove sources that have finished.
        let mut i = 0;
        while i != self.sources.len() {
            if self.sources[i].remove {
                self.sources.remove(i);
            } else {
                i += 1;
            }
        }
    }
}
