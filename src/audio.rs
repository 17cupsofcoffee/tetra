//! Functions and types relating to audio playback.

use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use rodio::source::Empty;
use rodio::{Decoder, Device, Sample, Source};

use crate::error::{Result, TetraError};
use crate::Context;

pub(crate) struct AudioContext {
    device: Option<Device>,
    master_volume: Arc<Mutex<f32>>,
}

impl AudioContext {
    pub(crate) fn new() -> AudioContext {
        let device = rodio::default_output_device();

        if let Some(active_device) = &device {
            rodio::play_raw(&active_device, Empty::new());
        }

        AudioContext {
            device,
            master_volume: Arc::new(Mutex::new(1.0)),
        }
    }
}

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
    data: Arc<[u8]>,
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

    /// Creates a new sound from a slice of binary data.
    /// 
    /// This is useful in combination with `include_bytes`, as it allows you to include
    /// your audio data directly in the binary.
    /// 
    /// Note that the data is not decoded until playback begins, so this function will not
    /// validate that the data being read is formatted correctly.
    pub fn from_data(data: &[u8]) -> Sound {
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
        self.start_source(ctx, true, false, 1.0, 1.0)
    }

    /// Plays the sound repeatedly.
    ///
    /// # Errors
    ///
    /// If there is no active audio device, a `TetraError::NoAudioDevice` will be returned.
    ///
    /// If the sound data could not be decoded, a `TetraError::FailedToDecodeAudio` will be returned.
    pub fn repeat(&self, ctx: &Context) -> Result<SoundInstance> {
        self.start_source(ctx, true, true, 1.0, 1.0)
    }

    /// Spawns a new instance of the sound that is not playing yet.
    ///
    /// # Errors
    ///
    /// If there is no active audio device, a `TetraError::NoAudioDevice` will be returned.
    ///
    /// If the sound data could not be decoded, a `TetraError::FailedToDecodeAudio` will be returned.
    pub fn spawn(&self, ctx: &Context) -> Result<SoundInstance> {
        self.start_source(ctx, false, false, 1.0, 1.0)
    }

    /// Plays the sound, with the provided settings.
    ///
    /// # Errors
    ///
    /// If there is no active audio device, a `TetraError::NoAudioDevice` will be returned.
    ///
    /// If the sound data could not be decoded, a `TetraError::FailedToDecodeAudio` will be returned.
    pub fn play_with(&self, ctx: &Context, volume: f32, speed: f32) -> Result<SoundInstance> {
        self.start_source(ctx, true, false, volume, speed)
    }

    /// Plays the sound repeatedly, with the provided settings.
    ///
    /// # Errors
    ///
    /// If there is no active audio device, a `TetraError::NoAudioDevice` will be returned.
    ///
    /// If the sound data could not be decoded, a `TetraError::FailedToDecodeAudio` will be returned.
    pub fn repeat_with(&self, ctx: &Context, volume: f32, speed: f32) -> Result<SoundInstance> {
        self.start_source(ctx, true, true, volume, speed)
    }

    /// Spawns a new instance of the sound that is not playing yet, with the provided settings.
    ///
    /// # Errors
    ///
    /// If there is no active audio device, a `TetraError::NoAudioDevice` will be returned.
    ///
    /// If the sound data could not be decoded, a `TetraError::FailedToDecodeAudio` will be returned.
    pub fn spawn_with(&self, ctx: &Context, volume: f32, speed: f32) -> Result<SoundInstance> {
        self.start_source(ctx, false, false, volume, speed)
    }

    fn start_source(
        &self,
        ctx: &Context,
        playing: bool,
        repeating: bool,
        volume: f32,
        speed: f32,
    ) -> Result<SoundInstance> {
        let controls = Arc::new(RemoteControls {
            playing: AtomicBool::new(playing),
            repeating: AtomicBool::new(repeating),
            rewind: AtomicBool::new(false),
            volume: Mutex::new(volume),
            speed: Mutex::new(speed),
        });

        let master_volume = { *ctx.audio.master_volume.lock().unwrap() };

        let source = TetraSource {
            data: Arc::clone(&self.data),
            cursor: Decoder::new(Cursor::new(Arc::clone(&self.data)))?,

            remote_master_volume: Arc::clone(&ctx.audio.master_volume),
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
            ctx.audio.device.as_ref().ok_or(TetraError::NoAudioDevice)?,
            source.convert_samples(),
        );
        Ok(SoundInstance { controls })
    }
}

#[derive(Debug)]
struct RemoteControls {
    playing: AtomicBool,
    repeating: AtomicBool,
    rewind: AtomicBool,
    volume: Mutex<f32>,
    speed: Mutex<f32>,
}

/// A handle to a single instance of a [`Sound`](./struct.Sound.html).
///
/// The audio thread will poll this for updates every 220 samples (roughly
/// every 5ms at a 44100hz sample rate).
///
/// Note that dropping a `SoundInstance` does not stop playback.
#[derive(Debug, Clone)]
pub struct SoundInstance {
    controls: Arc<RemoteControls>,
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

struct TetraSource {
    data: Arc<[u8]>,
    cursor: Decoder<Cursor<Arc<[u8]>>>,

    remote_master_volume: Arc<Mutex<f32>>,
    remote_controls: Weak<RemoteControls>,
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
            self.cursor = Decoder::new(Cursor::new(Arc::clone(&self.data))).unwrap();
            self.rewind = false;

            if let Some(controls) = self.remote_controls.upgrade() {
                controls.rewind.store(false, Ordering::SeqCst);
            }
        }

        self.cursor
            .next()
            .or_else(|| {
                if self.repeating {
                    self.cursor = Decoder::new(Cursor::new(Arc::clone(&self.data))).unwrap();
                    self.cursor.next()
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
}

impl Source for TetraSource {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        self.cursor.current_frame_len()
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.cursor.channels()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        (self.cursor.sample_rate() as f32 * self.speed) as u32
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.cursor.total_duration()
    }
}

/// Sets the master volume for the game.
///
/// The parameter is used as a multiplier - for example, `1.0` would result in
/// sounds being played back at their original volume.
pub fn set_master_volume(ctx: &mut Context, volume: f32) {
    *ctx.audio.master_volume.lock().unwrap() = volume;
}

/// Gets the master volume for the game.
pub fn get_master_volume(ctx: &mut Context) -> f32 {
    *ctx.audio.master_volume.lock().unwrap()
}
