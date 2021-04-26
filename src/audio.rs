//! Functions and types relating to audio playback.

use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use lewton::inside_ogg::OggStreamReader;
use lewton::samples::InterleavedSamples;
// use hound::{SampleFormat, WavReader};
use oddio::{
    Controlled, Filter, Frame, Frames, FramesSignal, Gain, Handle, Mixer, Signal, Speed, Stop,
};

use crate::error::Result;
use crate::Context;

/// Sound data that can be played back.
///
/// All of the playback methods on this type return a [`SoundInstance`] that
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
/// | OGG Vorbis | `audio_vorbis` | Yes |
/// | MP3 | `audio_mp3` | Yes |
/// | FLAC | `audio_flac` | No |
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
#[derive(Debug, Clone)] // TODO: This used to impl PartialEq, do we still want that?
pub struct Sound {
    pub(crate) data: Arc<Frames<[f32; 2]>>,
}

impl Sound {
    /// Creates a new sound from the given file.
    ///
    /// Note that the data is not decoded until playback begins, so this function will not
    /// validate that the data being read is formatted correctly.
    ///
    /// # Errors
    ///
    /// * [`TetraError::FailedToLoadAsset`] will be returned if the file could not be loaded.
    pub fn new<P>(path: P) -> Result<Sound>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).expect("TODO");
        let mut stream = OggStreamReader::new(file).unwrap();
        let mut samples = vec![];

        while let Some(packet) = stream
            .read_dec_packet_generic::<InterleavedSamples<f32>>()
            .expect("TODO")
        {
            // TODO: Check for mixed channel counts
            samples.extend(packet.samples);
        }

        let data = match stream.ident_hdr.audio_channels {
            1 => Frames::from_iter(
                stream.ident_hdr.audio_sample_rate,
                samples.into_iter().map(|s| [s, s]),
            ),
            2 => {
                let stereo = oddio::frame_stereo(&mut samples);
                Frames::from_slice(stream.ident_hdr.audio_sample_rate, stereo)
            }
            _ => todo!(),
        };

        // let mut reader = WavReader::open(path).expect("TODO");
        // let spec = reader.spec();

        // let data = match spec.sample_format {
        //     SampleFormat::Float => {
        //         let samples = reader.samples::<f32>().map(|s| s.unwrap_or(0.0));

        //         match spec.channels {
        //             1 => Frames::from_iter(spec.sample_rate, samples.map(|s| [s, s])),
        //             2 => {
        //                 let mut buffer = samples.collect::<Vec<_>>();
        //                 let stereo = oddio::frame_stereo(&mut buffer);
        //                 Frames::from_slice(spec.sample_rate, stereo)
        //             }
        //             _ => todo!(),
        //         }
        //     }

        //     SampleFormat::Int => {
        //         let max_int = (1 << spec.bits_per_sample) / 2;
        //         let scale = 1.0 / max_int as f32;

        //         let samples = reader
        //             .samples::<i32>()
        //             .map(|s| s.unwrap_or(0) as f32 * scale);

        //         match spec.channels {
        //             1 => Frames::from_iter(spec.sample_rate, samples.map(|s| [s, s])),
        //             2 => {
        //                 let mut buffer = samples.collect::<Vec<_>>();
        //                 let stereo = oddio::frame_stereo(&mut buffer);
        //                 Frames::from_slice(spec.sample_rate, stereo)
        //             }
        //             _ => todo!(),
        //         }
        //     }
        // };

        Ok(Sound { data })
    }

    /// Creates a new sound from a slice of binary data, encoded in one of Tetra's supported
    /// file formats.
    ///
    /// This is useful in combination with [`include_bytes`](std::include_bytes), as it
    /// allows you to include your audio data directly in the binary.
    ///
    /// Note that the data is not decoded until playback begins, so this function will not
    /// validate that the data being read is formatted correctly.
    pub fn from_file_data(data: &[u8]) -> Sound {
        todo!()
    }

    /// Plays the sound.
    ///
    /// # Errors
    ///
    /// * [`TetraError::NoAudioDevice`] will be returned if no audio device is active.
    /// * [`TetraError::InvalidSound`] will be returned if the sound data could not be decoded.
    pub fn play(&self, ctx: &mut Context) -> Result<SoundInstance> {
        ctx.audio
            .play_sound(Arc::clone(&self.data), true, false, 1.0, 1.0)
            .map(|handle| SoundInstance { handle })
    }

    /// Plays the sound repeatedly.
    ///
    /// # Errors
    ///
    /// * [`TetraError::NoAudioDevice`] will be returned if no audio device is active.
    /// * [`TetraError::InvalidSound`] will be returned if the sound data could not be decoded.
    pub fn repeat(&self, ctx: &mut Context) -> Result<SoundInstance> {
        ctx.audio
            .play_sound(Arc::clone(&self.data), true, true, 1.0, 1.0)
            .map(|handle| SoundInstance { handle })
    }

    /// Spawns a new instance of the sound that is not playing yet.
    ///
    /// # Errors
    ///
    /// * [`TetraError::NoAudioDevice`] will be returned if no audio device is active.
    /// * [`TetraError::InvalidSound`] will be returned if the sound data could not be decoded.
    pub fn spawn(&self, ctx: &mut Context) -> Result<SoundInstance> {
        ctx.audio
            .play_sound(Arc::clone(&self.data), false, false, 1.0, 1.0)
            .map(|handle| SoundInstance { handle })
    }

    /// Plays the sound, with the provided settings.
    ///
    /// # Errors
    ///
    /// * [`TetraError::NoAudioDevice`] will be returned if no audio device is active.
    /// * [`TetraError::InvalidSound`] will be returned if the sound data could not be decoded.
    pub fn play_with(&self, ctx: &mut Context, volume: f32, speed: f32) -> Result<SoundInstance> {
        ctx.audio
            .play_sound(Arc::clone(&self.data), true, false, volume, speed)
            .map(|handle| SoundInstance { handle })
    }

    /// Plays the sound repeatedly, with the provided settings.
    ///
    /// # Errors
    ///
    /// * [`TetraError::NoAudioDevice`] will be returned if no audio device is active.
    /// * [`TetraError::InvalidSound`] will be returned if the sound data could not be decoded.
    pub fn repeat_with(&self, ctx: &mut Context, volume: f32, speed: f32) -> Result<SoundInstance> {
        ctx.audio
            .play_sound(Arc::clone(&self.data), true, true, volume, speed)
            .map(|handle| SoundInstance { handle })
    }

    /// Spawns a new instance of the sound that is not playing yet, with the provided settings.
    ///
    /// # Errors
    ///
    /// * [`TetraError::NoAudioDevice`] will be returned if no audio device is active.
    /// * [`TetraError::InvalidSound`] will be returned if the sound data could not be decoded.
    pub fn spawn_with(&self, ctx: &mut Context, volume: f32, speed: f32) -> Result<SoundInstance> {
        ctx.audio
            .play_sound(Arc::clone(&self.data), false, false, volume, speed)
            .map(|handle| SoundInstance { handle })
    }
}

/// A handle to a single instance of a [`Sound`].
///
/// The audio thread will poll this for updates every 220 samples (roughly
/// every 5ms at a 44100hz sample rate).
///
/// Cloning a `SoundInstance` will create a new handle to the same instance,
/// rather than creating a new instance.
///
/// Note that dropping a `SoundInstance` does not stop playback, and the underlying
/// data will not be freed until playback has finished. This means that dropping a
/// [repeating](SoundInstance::set_repeating) `SoundInstance` without stopping it
/// first will cause the sound to loop forever.
// TODO: This used to be Debug and Clone
pub struct SoundInstance {
    handle: TetraHandle,
}

impl SoundInstance {
    /// Plays the sound if it is stopped, or resumes the sound if it is paused.
    pub fn play(&mut self) {
        self.set_state(SoundState::Playing)
    }

    /// Stops the sound. If playback is resumed, it will start over from the
    /// beginning.
    pub fn stop(&mut self) {
        self.set_state(SoundState::Stopped);
    }

    /// Pauses the sound. If playback is resumed, it will continue
    /// from the point where it was paused.
    pub fn pause(&mut self) {
        self.set_state(SoundState::Paused);
    }

    /// Returns the current state of playback.
    pub fn state(&self) -> SoundState {
        todo!()
    }

    /// Sets the current state of playback.
    ///
    /// In most cases, using the [`play`](SoundInstance::play), [`stop`](SoundInstance::stop) and
    /// [`pause`](SoundInstance::pause) methods is easier than explicitly setting a state, but
    /// this may be useful when, for example, defining transitions from one state to another.
    pub fn set_state(&mut self, state: SoundState) {
        match state {
            SoundState::Playing => {
                self.handle.control::<Pauseable<_>, _>().set_paused(false);
            }
            SoundState::Paused => {
                self.handle.control::<Pauseable<_>, _>().set_paused(true);
            }
            SoundState::Stopped => {
                todo!()
            }
        }
    }

    /// Sets the volume of the sound.
    ///
    /// The parameter is used as a multiplier - for example, `1.0` would result in the
    /// sound being played back at its original volume.
    pub fn set_volume(&mut self, volume: f32) {
        todo!()
    }

    /// Sets the speed (and by extension, the pitch) of the sound.
    ///
    /// The parameter is used as a multiplier - for example, `1.0` would result in the
    /// sound being played back at its original speed.
    pub fn set_speed(&mut self, speed: f32) {
        todo!()
    }

    /// Sets whether the sound should repeat or not.
    pub fn set_repeating(&mut self, repeating: bool) {
        todo!()
    }

    /// Toggles whether the sound should repeat or not.
    pub fn toggle_repeating(&mut self) {
        todo!()
    }
}

/// The states that playback of a [`SoundInstance`] can be in.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SoundState {
    /// The sound is currently playing.
    ///
    /// If a [`SoundInstance`] is created via [`Sound::play`], [`Sound::play_with`],
    /// [`Sound::repeat`] or [`Sound::repeat_with`], it will be in this state
    /// initially.
    Playing,

    /// The sound is paused. If playback is resumed, it will continue
    /// from the point where it was paused.
    ///
    /// If a [`SoundInstance`] is created via [`Sound::spawn`] or [`Sound::spawn_with`],
    /// it will be in this state initially.
    Paused,

    /// The sound has stopped, either manually or as a result of it reaching
    /// the end of the audio data. If playback is resumed, it will start
    /// over from the beginning of the sound.
    ///
    /// This state will never occur while a [`SoundInstance`] is set
    /// to be [`repeating`](SoundInstance::set_repeating).
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

pub(crate) struct AudioDevice {
    stream: Stream,
    mixer_handle: Handle<Mixer<[f32; 2]>>,
    master_volume: Arc<AtomicU32>,
}

impl AudioDevice {
    pub(crate) fn new() -> AudioDevice {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("TODO");
        let output_config = device.default_output_config().expect("TODO").config();
        let sample_rate = output_config.sample_rate.0;

        let (mut mixer_handle, mut mixer) = oddio::split(Mixer::new());

        let stream = device
            .build_output_stream(
                &output_config,
                move |data: &mut [f32], _| {
                    let samples = oddio::frame_stereo(data);

                    for s in &mut samples[..] {
                        *s = [0.0, 0.0];
                    }

                    oddio::run(&mixer, sample_rate, samples);
                },
                move |err| {
                    eprintln!("{}", err);
                },
            )
            .expect("TODO");

        stream.play().expect("TODO");

        AudioDevice {
            stream,
            mixer_handle,
            master_volume: Arc::new(AtomicU32::new(1.0f32.to_bits())),
        }
    }

    fn master_volume(&self) -> f32 {
        f32::from_bits(self.master_volume.load(Ordering::SeqCst))
    }

    fn set_master_volume(&self, volume: f32) {
        self.master_volume.store(volume.to_bits(), Ordering::SeqCst);
    }

    fn play_sound(
        &mut self,
        data: Arc<Frames<[f32; 2]>>,
        playing: bool,
        repeating: bool,
        volume: f32,
        speed: f32,
    ) -> Result<TetraHandle> {
        let source = FramesSignal::new(data, 0.0);

        let source = Gain::new(source);
        let source = Speed::new(source);
        let source = Pauseable::new(source, !playing);

        let mut handle = self.mixer_handle.control::<Mixer<_>, _>().play(source);

        handle.control::<Gain<_>, _>().set_gain(volume);
        handle.control::<Speed<_>, _>().set_speed(speed);

        Ok(handle)
    }
}

type TetraHandle = Handle<Stop<Pauseable<Speed<Gain<FramesSignal<[f32; 2]>>>>>>;

// TODO: Everything below should be replaced with stuff built-in to Oddio, or I should get Ralith to
// make sure it's not disgustingly broken.

struct Pauseable<T: ?Sized> {
    paused: AtomicBool,
    inner: T,
}

impl<T> Pauseable<T> {
    fn new(signal: T, paused: bool) -> Self {
        Self {
            paused: AtomicBool::new(paused),
            inner: signal,
        }
    }
}

impl<T: Signal> Signal for Pauseable<T>
where
    T::Frame: Frame,
{
    type Frame = T::Frame;

    fn sample(&self, interval: f32, out: &mut [T::Frame]) {
        if self.paused.load(Ordering::Relaxed) {
            for frame in out {
                *frame = T::Frame::ZERO;
            }
        } else {
            self.inner.sample(interval, out);
        }
    }

    fn remaining(&self) -> f32 {
        // TODO: Is this right?
        self.inner.remaining()
    }
}

impl<T> Filter for Pauseable<T> {
    type Inner = T;

    fn inner(&self) -> &T {
        &self.inner
    }
}

struct PauseableControl<'a, T>(&'a Pauseable<T>);

unsafe impl<'a, T: 'a> Controlled<'a> for Pauseable<T> {
    type Control = PauseableControl<'a, T>;

    unsafe fn make_control(signal: &'a Pauseable<T>) -> Self::Control {
        PauseableControl(signal)
    }
}

impl<'a, T> PauseableControl<'a, T> {
    pub fn set_paused(&mut self, paused: bool) {
        self.0.paused.store(paused, Ordering::Relaxed);
    }
}
