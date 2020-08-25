use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use rodio::source::{Buffered, Empty};
use rodio::{Decoder, Device as RodioDevice, Sample, Source};

use crate::error::{Result, TetraError};

pub use rodio::decoder::DecoderError;

pub struct AudioControls {
    playing: AtomicBool,
    repeating: AtomicBool,
    rewind: AtomicBool,
    volume: Mutex<f32>,
    speed: Mutex<f32>,
}

impl AudioControls {
    pub fn play(&self) {
        self.playing.store(true, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.playing.store(false, Ordering::SeqCst);
        self.rewind.store(true, Ordering::SeqCst);
    }

    pub fn pause(&self) {
        self.playing.store(false, Ordering::SeqCst);
    }

    pub fn set_volume(&self, volume: f32) {
        *self.volume.lock().unwrap() = volume;
    }

    pub fn set_speed(&self, speed: f32) {
        *self.speed.lock().unwrap() = speed;
    }

    pub fn repeating(&self) -> bool {
        self.repeating.load(Ordering::SeqCst)
    }

    pub fn set_repeating(&self, repeating: bool) {
        self.repeating.store(repeating, Ordering::SeqCst);
    }
    
    pub fn finished(&self) -> bool{
        self.rewind.load(Ordering::SeqCst)
    }
}

pub struct AudioDevice {
    device: Option<RodioDevice>,
    master_volume: Arc<Mutex<f32>>,
}

impl AudioDevice {
    pub fn new() -> AudioDevice {
        let device = rodio::default_output_device();

        if let Some(active_device) = &device {
            rodio::play_raw(&active_device, Empty::new());
        }

        AudioDevice {
            device,
            master_volume: Arc::new(Mutex::new(1.0)),
        }
    }

    pub fn master_volume(&self) -> f32 {
        *self.master_volume.lock().unwrap()
    }

    pub fn set_master_volume(&self, volume: f32) {
        *self.master_volume.lock().unwrap() = volume;
    }

    pub fn play_sound(
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
