//! Functions and types relating to audio playback.

use std::fmt::{self, Debug, Formatter};
use std::path::Path;
use std::sync::Arc;

use crate::error::Result;
use crate::fs;
use crate::platform::AudioControls;
use crate::Context;

/// Sound data that can be played back.
///
/// All of the playback methods on this type return a [`SoundInstance`](./struct.SoundInstance.html) that
/// can be used to control the sound after it has started. If you just want
/// to 'fire and forget' a sound, you can discard it - the sound will
/// continue playing regardless.
///
/// # Supported Formats
///
/// Various file formats are supported, and can be enabled or disabled via Cargo features:
///
/// | Format | Cargo feature | Enabled by default? |
/// |-|-|-|
/// | WAV | `audio_wav` | Yes |
/// | OGG Vorbis | `audio_vorbis | Yes |
/// | FLAC | `audio_flac` | Yes |
/// | MP3 | `audio_mp3` | Yes |
///
/// For convenience, there is also an `audio_all_formats` feature (which enables all of the above formats)
/// and an `audio_default_formats` feature (which just enables the default formats).
///
/// # Performance
///
/// Creating a `Sound` is a fairly cheap operation, as the data is not decoded until playback begins.
///
/// Cloning a `Sound` is a very cheap operation, as the underlying data is shared between the
/// original instance and the clone via [reference-counting](https://doc.rust-lang.org/std/rc/struct.Rc.html).
///
/// # Examples
///
/// The [`audio`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/audio.rs)
/// example demonstrates how to play several different kinds of sound.
#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    pub(crate) data: Arc<[u8]>,
}

impl Sound {
    /// Creates a new sound from the given file.
    ///
    /// Note that the data is not decoded until playback begins, so this function will not
    /// validate that the data being read is formatted correctly.
    ///
    /// # Errors
    ///
    /// * `TetraError::FailedToLoadAsset` will be returned if the file could not be loaded.
    pub fn new<P>(path: P) -> Result<Sound>
    where
        P: AsRef<Path>,
    {
        Ok(Sound {
            data: fs::read(path)?.into(),
        })
    }

    /// Creates a new sound from a slice of binary data, encoded in one of Tetra's supported
    /// file formats.
    ///
    /// This is useful in combination with `include_bytes`, as it allows you to include
    /// your audio data directly in the binary.
    ///
    /// Note that the data is not decoded until playback begins, so this function will not
    /// validate that the data being read is formatted correctly.
    pub fn from_file_data(data: &[u8]) -> Sound {
        Sound { data: data.into() }
    }

    /// Plays the sound.
    ///
    /// # Errors
    ///
    /// * `TetraError::NoAudioDevice` will be returned if no audio device is active.
    /// * `TetraError::InvalidSound` will be returned if the sound data could not be decoded.
    pub fn play(&self, ctx: &Context) -> Result<SoundInstance> {
        ctx.audio
            .play_sound(Arc::clone(&self.data), true, false, 1.0, 1.0)
            .map(|controls| SoundInstance { controls })
    }

    /// Plays the sound repeatedly.
    ///
    /// # Errors
    ///
    /// * `TetraError::NoAudioDevice` will be returned if no audio device is active.
    /// * `TetraError::InvalidSound` will be returned if the sound data could not be decoded.
    pub fn repeat(&self, ctx: &Context) -> Result<SoundInstance> {
        ctx.audio
            .play_sound(Arc::clone(&self.data), true, true, 1.0, 1.0)
            .map(|controls| SoundInstance { controls })
    }

    /// Spawns a new instance of the sound that is not playing yet.
    ///
    /// # Errors
    ///
    /// * `TetraError::NoAudioDevice` will be returned if no audio device is active.
    /// * `TetraError::InvalidSound` will be returned if the sound data could not be decoded.
    pub fn spawn(&self, ctx: &Context) -> Result<SoundInstance> {
        ctx.audio
            .play_sound(Arc::clone(&self.data), false, false, 1.0, 1.0)
            .map(|controls| SoundInstance { controls })
    }

    /// Plays the sound, with the provided settings.
    ///
    /// # Errors
    ///
    /// * `TetraError::NoAudioDevice` will be returned if no audio device is active.
    /// * `TetraError::InvalidSound` will be returned if the sound data could not be decoded.
    pub fn play_with(&self, ctx: &Context, volume: f32, speed: f32) -> Result<SoundInstance> {
        ctx.audio
            .play_sound(Arc::clone(&self.data), true, false, volume, speed)
            .map(|controls| SoundInstance { controls })
    }

    /// Plays the sound repeatedly, with the provided settings.
    ///
    /// # Errors
    ///
    /// * `TetraError::NoAudioDevice` will be returned if no audio device is active.
    /// * `TetraError::InvalidSound` will be returned if the sound data could not be decoded.
    pub fn repeat_with(&self, ctx: &Context, volume: f32, speed: f32) -> Result<SoundInstance> {
        ctx.audio
            .play_sound(Arc::clone(&self.data), true, true, volume, speed)
            .map(|controls| SoundInstance { controls })
    }

    /// Spawns a new instance of the sound that is not playing yet, with the provided settings.
    ///
    /// # Errors
    ///
    /// * `TetraError::NoAudioDevice` will be returned if no audio device is active.
    /// * `TetraError::InvalidSound` will be returned if the sound data could not be decoded.
    pub fn spawn_with(&self, ctx: &Context, volume: f32, speed: f32) -> Result<SoundInstance> {
        ctx.audio
            .play_sound(Arc::clone(&self.data), false, false, volume, speed)
            .map(|controls| SoundInstance { controls })
    }
}

/// A handle to a single instance of a [`Sound`](./struct.Sound.html).
///
/// The audio thread will poll this for updates every 220 samples (roughly
/// every 5ms at a 44100hz sample rate).
///
/// Cloning a `SoundInstance` will create a new handle to the same instance,
/// rather than creating a new instance.
///
/// Note that dropping a `SoundInstance` does not stop playback.
#[derive(Clone)]
pub struct SoundInstance {
    pub(crate) controls: Arc<AudioControls>,
}

impl SoundInstance {
    /// Plays the sound if it is stopped, or resumes the sound if it is paused.
    pub fn play(&self) {
        self.set_state(SoundState::Playing)
    }

    /// Stops the sound. If playback is resumed, it will start over from the
    /// beginning.
    pub fn stop(&self) {
        self.set_state(SoundState::Stopped);
    }

    /// Pauses the sound. If playback is resumed, it will continue
    /// from the point where it was paused.
    pub fn pause(&self) {
        self.set_state(SoundState::Paused);
    }

    /// Returns the current state of playback.
    pub fn state(&self) -> SoundState {
        self.controls.state()
    }

    /// Sets the current state of playback.
    ///
    /// In most cases, using the `play`, `stop` and `pause` methods is easier than explicitly
    /// setting a state, but this may be useful when, for example, defining transitions from
    /// one state to another.
    pub fn set_state(&self, state: SoundState) {
        self.controls.set_state(state)
    }

    /// Sets the volume of the sound.
    ///
    /// The parameter is used as a multiplier - for example, `1.0` would result in the
    /// sound being played back at its original volume.
    pub fn set_volume(&self, volume: f32) {
        self.controls.set_volume(volume);
    }

    /// Sets the speed (and by extension, the pitch) of the sound.
    ///
    /// The parameter is used as a multiplier - for example, `1.0` would result in the
    /// sound being played back at its original speed.
    pub fn set_speed(&self, speed: f32) {
        self.controls.set_speed(speed);
    }

    /// Sets whether the sound should repeat or not.
    pub fn set_repeating(&self, repeating: bool) {
        self.controls.set_repeating(repeating);
    }

    /// Toggles whether the sound should repeat or not.
    pub fn toggle_repeating(&self) {
        self.controls.set_repeating(!self.controls.repeating());
    }
}

// TODO: Remove or make more useful in 0.4.
impl Debug for SoundInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoundInstance")
            .field("controls", &"<platform internals>")
            .finish()
    }
}

/// The states that playback of a `SoundInstance` can be in.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SoundState {
    /// The sound is currently playing.
    ///
    /// If a `SoundInstance` is created via `Sound::play`, `Sound::play_with`,
    /// `Sound::repeat` or `Sound::repeat_with`, it will be in this state
    /// initially.
    Playing,

    /// The sound is paused. If playback is resumed, it will continue
    /// from the point where it was paused.
    ///
    /// If a `SoundInstance` is created via `Sound::spawn` or `Sound::spawn_with`,
    /// it will be in this state initially.
    Paused,

    /// The sound has stopped, either manually or as a result of it reaching
    /// the end of the audio data. If playback is resumed, it will start
    /// over from the beginning of the sound.
    ///
    /// This state will never occur while a `SoundInstance` is set
    /// to be `repeating`.
    Stopped,
}

/// Sets the master volume for the game.
///
/// The parameter is used as a multiplier - for example, `1.0` would result in
/// sounds being played back at their original volume.
pub fn set_master_volume(ctx: &mut Context, volume: f32) {
    ctx.audio.set_master_volume(volume);
}

/// Gets the master volume for the game.
pub fn get_master_volume(ctx: &mut Context) -> f32 {
    ctx.audio.master_volume()
}
