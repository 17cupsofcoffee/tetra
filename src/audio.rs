//! Functions and types relating to audio playback.

use std::io::Cursor;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use rodio::source::{Buffered, Empty};
use rodio::{Decoder, Device as RodioDevice, Sample, Source};

use crate::error::{Result, TetraError};
use crate::fs;
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
#[derive(Debug, Clone)]
pub struct SoundInstance {
    controls: Arc<AudioControls>,
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

#[derive(Debug)]
struct AudioControls {
    playing: AtomicBool,
    repeating: AtomicBool,
    rewind: AtomicBool,
    volume: Mutex<f32>,
    speed: Mutex<f32>,
}

impl AudioControls {
    fn set_volume(&self, volume: f32) {
        *self.volume.lock().unwrap() = volume;
    }

    fn state(&self) -> SoundState {
        if self.playing.load(Ordering::SeqCst) {
            SoundState::Playing
        } else if self.rewind.load(Ordering::SeqCst) {
            SoundState::Stopped
        } else {
            SoundState::Paused
        }
    }

    fn set_state(&self, state: SoundState) {
        match state {
            SoundState::Playing => {
                self.playing.store(true, Ordering::SeqCst);
            }
            SoundState::Paused => {
                self.playing.store(false, Ordering::SeqCst);
            }
            SoundState::Stopped => {
                self.playing.store(false, Ordering::SeqCst);
                self.rewind.store(true, Ordering::SeqCst);
            }
        }
    }

    fn set_speed(&self, speed: f32) {
        *self.speed.lock().unwrap() = speed;
    }

    fn repeating(&self) -> bool {
        self.repeating.load(Ordering::SeqCst)
    }

    fn set_repeating(&self, repeating: bool) {
        self.repeating.store(repeating, Ordering::SeqCst);
    }
}

pub(crate) struct AudioDevice {
    device: Option<RodioDevice>,
    master_volume: Arc<Mutex<f32>>,
}

impl AudioDevice {
    pub(crate) fn new() -> AudioDevice {
        let device = rodio::default_output_device();

        if let Some(active_device) = &device {
            rodio::play_raw(&active_device, Empty::new());
        }

        AudioDevice {
            device,
            master_volume: Arc::new(Mutex::new(1.0)),
        }
    }

    fn master_volume(&self) -> f32 {
        *self.master_volume.lock().unwrap()
    }

    fn set_master_volume(&self, volume: f32) {
        *self.master_volume.lock().unwrap() = volume;
    }

    fn play_sound(
        &self,
        data: Arc<[u8]>,
        playing: bool,
        repeating: bool,
        volume: f32,
        speed: f32,
    ) -> Result<Arc<AudioControls>> {
        let controls = Arc::new(AudioControls {
            playing: AtomicBool::new(playing),
            repeating: AtomicBool::new(repeating),
            rewind: AtomicBool::new(false),
            volume: Mutex::new(volume),
            speed: Mutex::new(speed),
        });

        let master_volume = { *self.master_volume.lock().unwrap() };

        let data = Decoder::new(Cursor::new(data))
            .map_err(TetraError::InvalidSound)?
            .buffered();

        let source = TetraSource {
            repeat_source: data.clone(),
            data,

            remote_master_volume: Arc::clone(&self.master_volume),
            remote_controls: Arc::downgrade(&Arc::clone(&controls)),
            time_till_update: 220,

            detached: false,
            playing,
            repeating,
            rewind: false,
            master_volume,
            volume,
            speed,
        };

        rodio::play_raw(
            self.device.as_ref().ok_or(TetraError::NoAudioDevice)?,
            source.convert_samples(),
        );

        Ok(controls)
    }
}

type TetraSourceData = Buffered<Decoder<Cursor<Arc<[u8]>>>>;

struct TetraSource {
    data: TetraSourceData,
    repeat_source: TetraSourceData,

    remote_master_volume: Arc<Mutex<f32>>,
    remote_controls: Weak<AudioControls>,
    time_till_update: u32,

    detached: bool,
    playing: bool,
    repeating: bool,
    rewind: bool,
    master_volume: f32,
    volume: f32,
    speed: f32,
}

impl Iterator for TetraSource {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        // There's a lot of shenanigans in this method where we try to keep the local state and
        // the remote state in sync. I'm not sure if it'd be a better idea to just load data from the
        // controls every sample or whether that'd be too slow...

        self.time_till_update -= 1;

        if self.time_till_update == 0 {
            self.master_volume = *self.remote_master_volume.lock().unwrap();

            if let Some(controls) = self.remote_controls.upgrade() {
                self.playing = controls.playing.load(Ordering::SeqCst);

                // If we're not playing, we don't really care about updating the rest of the state.
                if self.playing {
                    self.repeating = controls.repeating.load(Ordering::SeqCst);
                    self.rewind = controls.rewind.load(Ordering::SeqCst);
                    self.volume = *controls.volume.lock().unwrap();
                    self.speed = *controls.speed.lock().unwrap();
                }
            } else {
                self.detached = true;
            }

            self.time_till_update = 220;
        }

        if !self.playing {
            return if self.detached { None } else { Some(0) };
        }

        if self.rewind {
            self.data = self.repeat_source.clone();
            self.rewind = false;

            if let Some(controls) = self.remote_controls.upgrade() {
                controls.rewind.store(false, Ordering::SeqCst);
            }
        }

        self.data
            .next()
            .or_else(|| {
                if self.repeating {
                    self.data = self.repeat_source.clone();
                    self.data.next()
                } else {
                    None
                }
            })
            .map(|v| v.amplify(self.volume).amplify(self.master_volume))
            .or_else(|| {
                if self.detached {
                    None
                } else {
                    // Report that the sound has finished.
                    if !self.rewind {
                        self.playing = false;
                        self.rewind = true;

                        if let Some(controls) = self.remote_controls.upgrade() {
                            controls.playing.store(false, Ordering::SeqCst);
                            controls.rewind.store(true, Ordering::SeqCst);
                        }
                    }

                    Some(0)
                }
            })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl Source for TetraSource {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        match self.data.current_frame_len() {
            Some(0) => self.repeat_source.current_frame_len(),
            a => a,
        }
    }

    #[inline]
    fn channels(&self) -> u16 {
        match self.data.current_frame_len() {
            Some(0) => self.repeat_source.channels(),
            _ => self.data.channels(),
        }
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        match self.data.current_frame_len() {
            Some(0) => (self.repeat_source.sample_rate() as f32 * self.speed) as u32,
            _ => (self.data.sample_rate() as f32 * self.speed) as u32,
        }
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
