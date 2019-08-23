//! Functions and types relating to audio playback.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::error::Result;
use crate::platform;
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
    /// # Errors
    ///
    /// If the file path is invalid, a `TetraError::Io` will be returned. Note that the data
    /// is not decoded until playback begins, so this function will not validate
    /// that the data being read is formatted correctly.
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
    /// If there is no active audio device, a `TetraError::NoAudioDevice` will be returned.
    ///
    /// If the sound data could not be decoded, a `TetraError::FailedToDecodeAudio` will be returned.
    pub fn play(&self, ctx: &Context) -> Result<SoundInstance> {
        platform::play_sound(ctx, self, true, false, 1.0, 1.0)
    }

    /// Plays the sound repeatedly.
    ///
    /// # Errors
    ///
    /// If there is no active audio device, a `TetraError::NoAudioDevice` will be returned.
    ///
    /// If the sound data could not be decoded, a `TetraError::FailedToDecodeAudio` will be returned.
    pub fn repeat(&self, ctx: &Context) -> Result<SoundInstance> {
        platform::play_sound(ctx, self, true, true, 1.0, 1.0)
    }

    /// Spawns a new instance of the sound that is not playing yet.
    ///
    /// # Errors
    ///
    /// If there is no active audio device, a `TetraError::NoAudioDevice` will be returned.
    ///
    /// If the sound data could not be decoded, a `TetraError::FailedToDecodeAudio` will be returned.
    pub fn spawn(&self, ctx: &Context) -> Result<SoundInstance> {
        platform::play_sound(ctx, self, false, false, 1.0, 1.0)
    }

    /// Plays the sound, with the provided settings.
    ///
    /// # Errors
    ///
    /// If there is no active audio device, a `TetraError::NoAudioDevice` will be returned.
    ///
    /// If the sound data could not be decoded, a `TetraError::FailedToDecodeAudio` will be returned.
    pub fn play_with(&self, ctx: &Context, volume: f32, speed: f32) -> Result<SoundInstance> {
        platform::play_sound(ctx, self, true, false, volume, speed)
    }

    /// Plays the sound repeatedly, with the provided settings.
    ///
    /// # Errors
    ///
    /// If there is no active audio device, a `TetraError::NoAudioDevice` will be returned.
    ///
    /// If the sound data could not be decoded, a `TetraError::FailedToDecodeAudio` will be returned.
    pub fn repeat_with(&self, ctx: &Context, volume: f32, speed: f32) -> Result<SoundInstance> {
        platform::play_sound(ctx, self, true, true, volume, speed)
    }

    /// Spawns a new instance of the sound that is not playing yet, with the provided settings.
    ///
    /// # Errors
    ///
    /// If there is no active audio device, a `TetraError::NoAudioDevice` will be returned.
    ///
    /// If the sound data could not be decoded, a `TetraError::FailedToDecodeAudio` will be returned.
    pub fn spawn_with(&self, ctx: &Context, volume: f32, speed: f32) -> Result<SoundInstance> {
        platform::play_sound(ctx, self, false, false, volume, speed)
    }
}

#[derive(Debug)]
pub(crate) struct RemoteControls {
    pub(crate) playing: AtomicBool,
    pub(crate) repeating: AtomicBool,
    pub(crate) rewind: AtomicBool,
    pub(crate) volume: Mutex<f32>,
    pub(crate) speed: Mutex<f32>,
}

/// A handle to a single instance of a [`Sound`](./struct.Sound.html).
///
/// The audio thread will poll this for updates every 220 samples (roughly
/// every 5ms at a 44100hz sample rate).
///
/// Note that dropping a `SoundInstance` does not stop playback.
#[derive(Debug, Clone)]
pub struct SoundInstance {
    pub(crate) controls: Arc<RemoteControls>,
}

impl SoundInstance {
    /// Plays the sound if it is stopped, or resumes the sound if it is paused.
    pub fn play(&self) {
        self.controls.playing.store(true, Ordering::SeqCst);
    }

    /// Stops the sound, and rewinds it to the beginning.
    pub fn stop(&self) {
        self.controls.playing.store(false, Ordering::SeqCst);
        self.controls.rewind.store(true, Ordering::SeqCst);
    }

    /// Pauses the sound.
    pub fn pause(&self) {
        self.controls.playing.store(false, Ordering::SeqCst);
    }

    /// Sets the volume of the sound.
    ///
    /// The parameter is used as a multiplier - for example, `1.0` would result in the
    /// sound being played back at its original volume.
    pub fn set_volume(&self, volume: f32) {
        *self.controls.volume.lock().unwrap() = volume;
    }

    /// Sets the speed (and by extension, the pitch) of the sound.
    ///
    /// The parameter is used as a multiplier - for example, `1.0` would result in the
    /// sound being played back at its original speed.
    pub fn set_speed(&self, speed: f32) {
        *self.controls.speed.lock().unwrap() = speed;
    }

    /// Sets whether the sound should repeat or not.
    pub fn set_repeating(&self, repeating: bool) {
        self.controls.repeating.store(repeating, Ordering::SeqCst);
    }

    /// Toggles whether the sound should repeat or not.
    pub fn toggle_repeating(&self) {
        if self.controls.repeating.load(Ordering::SeqCst) {
            self.controls.repeating.store(false, Ordering::SeqCst);
        } else {
            self.controls.repeating.store(true, Ordering::SeqCst);
        }
    }
}

/// Sets the master volume for the game.
///
/// The parameter is used as a multiplier - for example, `1.0` would result in
/// sounds being played back at their original volume.
pub fn set_master_volume(ctx: &mut Context, volume: f32) {
    platform::set_master_volume(ctx, volume);
}

/// Gets the master volume for the game.
pub fn get_master_volume(ctx: &mut Context) -> f32 {
    platform::get_master_volume(ctx)
}
