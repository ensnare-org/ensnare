// Copyright (c) 2024 Mike Tsao

//! [CpalAudioService] provides channel-based communication with the
//! [cpal](https://crates.io/crates/cpal) audio interface.

use crate::{CrossbeamChannel, ProvidesService};
use core::fmt::Debug;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, FromSample, Sample as CpalSample, SizedSample, Stream, StreamConfig,
    SupportedStreamConfig,
};
use crossbeam::{
    channel::{Receiver, Sender},
    queue::ArrayQueue,
};
use delegate::delegate;
use std::sync::Arc;

/// The fundamental type of an audio sample.
pub type AudioSampleType = f32;
/// (left channel, right channel)
pub type AudioStereoSampleType = (AudioSampleType, AudioSampleType);

/// A ring buffer of stereo samples that the audio stream consumes.
struct AudioQueue(Arc<ArrayQueue<AudioStereoSampleType>>);
impl AudioQueue {
    fn new(buffer_size: usize) -> Self {
        Self(Arc::new(ArrayQueue::new(buffer_size)))
    }

    delegate! {
        to self.0 {
            fn len(&self) -> usize;
            fn capacity(&self) -> usize;
            fn pop(&self) -> Option<AudioStereoSampleType>;
            fn force_push(&self, frame: AudioStereoSampleType) -> Option<AudioStereoSampleType>;
        }
    }
}
impl Clone for AudioQueue {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

/// A [CpalAudioServiceInput] tells [CpalAudioService] what to do.
#[derive(Debug)]
pub enum CpalAudioServiceInput {
    /// Asks the service to exit.
    Quit,
    /// Provides audio frames for the audio interface. They will be added to the
    /// service's internal ring buffer and consumed as needed. TODO: I'm unsure
    /// whether Arc<Vec<>> is an efficient way to send this data over a channel.
    Frames(Arc<Vec<AudioStereoSampleType>>),
    /// Starts the underlying audio interface. It's unnecessary to send this for
    /// every new service, because a new service plays automatically upon
    /// creation.
    Play,
    /// Pauses the underlying audio interface.
    Pause,
    /// Sets the audio interface's sample rate. If the audio interface doesn't
    /// support this rate, then the set operation fails, and the prior rate
    /// remains. TODO not yet functional
    SetSampleRate(usize),
    /// Changes the period size, which is the basis for the internal audio
    /// buffer size. As an example, a 44.1KHz sample rate and a 512-frame period
    /// means that the audio will have at least an 11.6-millisecond delay (512 /
    /// 44100 = .0116). TODO not yet functional
    SetPeriodSize(usize),
}

/// A [CpalAudioServiceEvent] informs clients what's going on.
#[derive(Debug)]
pub enum CpalAudioServiceEvent {
    /// The service has initialized or reinitialized. Provides the new sample
    /// rate and channel count.
    Reset(usize, u8),
    /// The audio interface needs audio frames ASAP. Provide the specified
    /// number with [CpalAudioServiceInput::Frames].
    FramesNeeded(usize),
    /// Sent when the audio interface asked for more frames than we had
    /// available in the ring buffer.
    Underrun,
}

/// Wrapper for cpal structs. [WrappedStream] exists for two reasons: first, to
/// implement [core::fmt::Debug] for the structs that don't, and second, because
/// the stream needs to live in its own thread, so we manage that here.
struct WrappedStream {
    queue: AudioQueue,

    sample_rate: usize,
    channel_count: u8,
}
impl Debug for WrappedStream {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WrappedStream")
            .field("config", &"(skipped)")
            .field("cpal_stream", &"(skipped)")
            .field("queue", &self.queue.0)
            .finish()
    }
}
impl WrappedStream {
    /// period_size is the size, in frames, of a single group of frames in the
    /// audio buffer. <https://www.alsa-project.org/wiki/FramesPeriods>
    pub fn new_with(
        period_size: usize,
        sender: &Sender<CpalAudioServiceEvent>,
        receiver: &Receiver<CpalAudioServiceInput>,
    ) -> anyhow::Result<Self> {
        let (_host, device, config) = Self::host_device_setup()?;

        // The buffer size is a multiple of the period size. It's a good idea to
        // have at least two so that the hardware can consume one while the
        // software is generating one
        // <https://www.alsa-project.org/wiki/FramesPeriods>.
        //
        // We have three because I was getting an occasional buffer overrun (too
        // much production, not enough consumption), which caused the ring
        // buffer to overflow. I think this is masking an error in how this
        // service is emitting [CpalAudioServiceEvent::FramesNeeded] to the
        // client. TODO fix that
        let buffer_size = period_size * 3;
        let queue = AudioQueue::new(buffer_size);

        // Stream creation needs to live in its own thread because it isn't
        // `Send`. See <https://github.com/RustAudio/cpal/issues/818> for more
        // discussion.
        let receiver = receiver.clone();
        let config_clone = config.clone();
        let queue_clone = queue.clone();
        let sender = sender.clone();
        std::thread::spawn(move || {
            let queue_clone_2 = queue_clone.clone();
            match Self::stream_setup_for(
                &device,
                config_clone.clone(),
                period_size,
                queue_clone.clone(),
                sender,
            ) {
                Ok(cpal_stream) => {
                    while let Ok(input) = receiver.recv() {
                        match input {
                            CpalAudioServiceInput::Frames(frames) => {
                                for frame in frames.iter() {
                                    if queue_clone_2.force_push(*frame).is_some() {
                                        eprintln!("Caution: audio buffer overrun");
                                    };
                                }
                            }
                            CpalAudioServiceInput::Play => {
                                let _ = cpal_stream.play();
                            }
                            CpalAudioServiceInput::Pause => {
                                let _ = cpal_stream.pause();
                            }
                            CpalAudioServiceInput::Quit => {
                                break;
                            }
                            CpalAudioServiceInput::SetSampleRate(_new_sample_rate) => {
                                todo!();
                            }
                            CpalAudioServiceInput::SetPeriodSize(_new_period_size) => {
                                todo!();
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Failed while setting up audio stream: {e:?}"),
            }
        });
        Ok(Self {
            queue,
            sample_rate: config.sample_rate().0 as usize,
            channel_count: config.channels() as u8,
        })
    }

    /// Returns the default host, device, and stream config (all of which are
    /// cpal concepts).
    fn host_device_setup(
    ) -> anyhow::Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error>
    {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
        let config = device.default_output_config()?;

        let config = SupportedStreamConfig::new(
            config.channels(),
            config.sample_rate(),
            *config.buffer_size(),
            config.sample_format(),
        );
        Ok((host, device, config))
    }

    /// Creates and returns a Stream for the given device and config. The Stream
    /// will consume the data in the supplied [AudioQueue]. This function is
    /// actually a wrapper around the generic [stream_make<T>()].
    fn stream_setup_for(
        device: &cpal::Device,
        config: SupportedStreamConfig,
        period_size: usize,
        queue: AudioQueue,
        sender: Sender<CpalAudioServiceEvent>,
    ) -> anyhow::Result<Stream, anyhow::Error> {
        let sample_format = config.sample_format();
        let mut config: StreamConfig = config.into();

        // We set buffer size here, rather than in host_device_setup(), because
        // it's troublesome to create a [cpal::SupportedBufferSize] on the fly.
        config.buffer_size = BufferSize::Fixed(period_size as u32);

        match sample_format {
            cpal::SampleFormat::I8 => {
                Self::stream_make::<i8>(&config, device, period_size, queue, sender)
            }
            cpal::SampleFormat::I16 => {
                Self::stream_make::<i16>(&config, device, period_size, queue, sender)
            }
            cpal::SampleFormat::I32 => {
                Self::stream_make::<i32>(&config, device, period_size, queue, sender)
            }
            cpal::SampleFormat::I64 => {
                Self::stream_make::<i64>(&config, device, period_size, queue, sender)
            }
            cpal::SampleFormat::U8 => {
                Self::stream_make::<u8>(&config, device, period_size, queue, sender)
            }
            cpal::SampleFormat::U16 => {
                Self::stream_make::<u16>(&config, device, period_size, queue, sender)
            }
            cpal::SampleFormat::U32 => {
                Self::stream_make::<u32>(&config, device, period_size, queue, sender)
            }
            cpal::SampleFormat::U64 => {
                Self::stream_make::<u64>(&config, device, period_size, queue, sender)
            }
            cpal::SampleFormat::F32 => {
                Self::stream_make::<f32>(&config, device, period_size, queue, sender)
            }
            cpal::SampleFormat::F64 => {
                Self::stream_make::<f64>(&config, device, period_size, queue, sender)
            }
            _ => panic!("Unexpected sample format {sample_format:?}"),
        }
    }

    /// Generic portion of stream_setup_for().
    fn stream_make<T>(
        config: &cpal::StreamConfig,
        device: &cpal::Device,
        period_size: usize,
        queue: AudioQueue,
        sender: Sender<CpalAudioServiceEvent>,
    ) -> Result<Stream, anyhow::Error>
    where
        T: SizedSample + FromSample<AudioSampleType>,
    {
        let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

        let channel_count = config.channels as usize;
        let stream = device.build_output_stream(
            config,
            move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                Self::on_window(output, channel_count, period_size, &queue, &sender)
            },
            err_fn,
            None,
        )?;
        Ok(stream)
    }

    /// cpal callback that supplies samples from the AudioQueue, converting them
    /// if needed to the stream's expected data type.
    fn on_window<T>(
        output: &mut [T],
        channel_count: usize,
        period_size: usize,
        queue: &AudioQueue,
        sender: &Sender<CpalAudioServiceEvent>,
    ) where
        T: CpalSample + FromSample<AudioSampleType>,
    {
        let have_len = queue.len();
        let need_len = output.len();

        // Calculate how many frames we should request.
        let request_len = if have_len < need_len {
            // We're at risk of underrun. Increase work amount beyond what we're
            // about to consume.
            need_len * 2
        } else if have_len > need_len * 2 {
            // We are far ahead of the current window's needs. Replace only half
            // of the current request.
            need_len / 2
        } else {
            // We're keeping up. Replace exactly what we're about to consume.
            need_len
        }
        .min(period_size);

        for frame in output.chunks_exact_mut(channel_count) {
            if let Some(sample) = queue.pop() {
                frame[0] = T::from_sample(sample.0);
                if channel_count > 1 {
                    frame[1] = T::from_sample(sample.1);
                }
            } else {
                let _ = sender.send(CpalAudioServiceEvent::Underrun);

                // No point in continuing to loop.
                break;
            }
        }

        // Don't ask for more than the queue can hold.
        let request_len = (queue.capacity() - queue.len()).min(request_len);

        // Request the frames.
        let _ = sender.send(CpalAudioServiceEvent::FramesNeeded(request_len));
    }

    fn sample_rate(&self) -> usize {
        self.sample_rate
    }

    fn channel_count(&self) -> u8 {
        self.channel_count
    }
}

/// [CpalAudioService] provides channel-based communication with the cpal audio
/// interface.
#[derive(Debug)]
pub struct CpalAudioService {
    inputs: CrossbeamChannel<CpalAudioServiceInput>,
    events: CrossbeamChannel<CpalAudioServiceEvent>,

    /// The cpal audio stream.
    #[allow(dead_code)]
    stream: WrappedStream,
}
impl Default for CpalAudioService {
    fn default() -> Self {
        Self::new_with(None)
    }
}
impl ProvidesService<CpalAudioServiceInput, CpalAudioServiceEvent> for CpalAudioService {
    fn receiver(&self) -> &crossbeam::channel::Receiver<CpalAudioServiceEvent> {
        &self.events.receiver
    }

    fn sender(&self) -> &Sender<CpalAudioServiceInput> {
        &self.inputs.sender
    }
}
impl CpalAudioService {
    /// A reasonable period size. This value is on the upper edge of perceptible
    /// latency for 44.1KHz (512 / 44100 = 11.6 milliseconds).
    const SUGGESTED_PERIOD_SIZE: usize = 512;

    /// Creates a new [CpalAudioService] with an internal buffer whose size is
    /// based on the given period size, or a reasonable default if none is
    /// provided. A "period" is a chunk of the audio buffer that the audio
    /// interface reads at once. The buffer is actually an integer multiple of
    /// that size to give the software some slack time to fill the buffer while
    /// the hardware audio interface is draining it.
    ///
    /// Read <https://news.ycombinator.com/item?id=9388558> for food for
    /// thought.
    pub fn new_with(period_size: Option<usize>) -> Self {
        let inputs: CrossbeamChannel<CpalAudioServiceInput> = Default::default();
        let events: CrossbeamChannel<CpalAudioServiceEvent> = Default::default();
        let period_size = period_size.unwrap_or(Self::SUGGESTED_PERIOD_SIZE);
        match WrappedStream::new_with(period_size, &events.sender, &inputs.receiver) {
            Ok(stream) => {
                let audio_service = Self {
                    inputs,
                    events,
                    stream,
                };
                let _ = audio_service
                    .events
                    .sender
                    .send(CpalAudioServiceEvent::Reset(
                        audio_service.stream.sample_rate(),
                        audio_service.stream.channel_count(),
                    ));

                audio_service
            }
            Err(e) => panic!("While creating CpalAudioService: {e:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_queue() {
        let queue = AudioQueue::new(8);
        assert_eq!(queue.pop(), None);

        queue.force_push((0.5, -0.5));
        assert_eq!(queue.pop(), Some((0.5, -0.5)));
    }
}
