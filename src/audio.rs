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
/// Supports WAV, Ogg Vorbis, MP3 and FLAC (in other words, everything that
/// [Rodio](https://github.com/tomaka/rodio) provides support for).
///
/// All of the playback methods on this type return a [`SoundInstance`](./struct.SoundInstance.html) that
/// can be used to control the sound after it has started. If you just want
/// to 'fire and forget' a sound, you can discard it - the sound will
/// continue playing regardless.
///
/// This type acts as a lightweight handle to the associated audio data,
/// and so can be cloned with little overhead.
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
        self.controls.play();
    }

    /// Stops the sound, and rewinds it to the beginning.
    pub fn stop(&self) {
        self.controls.stop();
    }

    /// Pauses the sound.
    pub fn pause(&self) {
        self.controls.pause();
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
