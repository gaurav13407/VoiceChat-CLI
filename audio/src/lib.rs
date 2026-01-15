// Audio library for VoiceChat-CLI
// Provides voice capture, encoding, and streaming

pub mod buffer;
pub mod voice;
pub mod upd;
pub mod playback;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::bounded;
use std::sync::{Arc, Mutex};
use std::thread;

const FRAME_MS: usize = 20;
const BUFFER_FRAMES: usize = 4;
const VOLUME_GAIN: f32 = 1.5;

pub struct VoiceSession {
    _input_stream: cpal::Stream,
    _output_stream: cpal::Stream,
}

impl VoiceSession {
    /// Start voice chat session
    /// sender_id: unique ID for this peer
    /// send_addr: where to send audio (peer's receive address)
    /// recv_bind: where to listen for incoming audio
    pub fn start(
        sender_id: u32,
        send_addr: &str,
        recv_bind: &str,
    ) -> anyhow::Result<Self> {
        let host = cpal::default_host();

        let input = host
            .input_devices()?
            .find(|d| {
                let n = d.name().unwrap_or_default().to_lowercase();
                n == "pulse" || n == "default"
            })
            .ok_or_else(|| anyhow::anyhow!("No input device"))?;

        let output = host.default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No output device"))?;

        let input_cfg = input.default_input_config()?;
        let output_cfg = output.default_output_config()?;

        let sample_rate = input_cfg.sample_rate().0 as usize;
        let channels = input_cfg.channels() as usize;
        let frame_samples = sample_rate * FRAME_MS / 1000;



        let (tx_play, rx_play) = bounded::<Vec<f32>>(BUFFER_FRAMES);

        // Start UDP networking
        let mut udp_handle = upd::start_udp(
            sender_id,
            send_addr,
            recv_bind,
            tx_play.clone(),
        )?;

        // Input stream config
        let input_config = cpal::StreamConfig {
            channels: channels as u16,
            sample_rate: input_cfg.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        let output_config = cpal::StreamConfig {
            channels: output_cfg.channels(),
            sample_rate: output_cfg.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        // Build input stream
        let mut capture_acc: Vec<f32> = Vec::new();
        let input_stream = input
            .build_input_stream(
                &input_config,
                move |data: &[f32], _| {
                    for frame in data.chunks(channels) {
                        let sum: f32 = frame.iter().sum();
                        let sample = (sum / channels as f32) * VOLUME_GAIN;
                        capture_acc.push(sample.clamp(-1.0, 1.0));
                    }

                    while capture_acc.len() >= frame_samples {
                        let frame: Vec<f32> = capture_acc.drain(..frame_samples).collect();
                        udp_handle.send_frame(frame);
                    }
                },
                err_fn,
                None,
            )?;

        // Build output stream
        let mut playback_acc: Vec<f32> = Vec::new();
        let out_channels = output_config.channels as usize;

        let output_stream = output
            .build_output_stream(
                &output_config,
                move |out: &mut [f32], _| {
                    use crossbeam_channel::TryRecvError;

                    let mut written = 0;

                    while written < out.len() {
                        if playback_acc.is_empty() {
                            match rx_play.try_recv() {
                                Ok(frame) => playback_acc = frame,
                                Err(TryRecvError::Empty) => {
                                    out[written..].fill(0.0);
                                    return;
                                }
                                Err(_) => return,
                            }
                        }

                        let frames = (out.len() - written) / out_channels;
                        let n = frames.min(playback_acc.len());

                        for i in 0..n {
                            for ch in 0..out_channels {
                                out[written + i * out_channels + ch] = playback_acc[i];
                            }
                        }

                        playback_acc.drain(..n);
                        written += n * out_channels;
                    }
                },
                err_fn,
                None,
            )?;

        input_stream.play()?;
        output_stream.play()?;

        Ok(VoiceSession {
            _input_stream: input_stream,
            _output_stream: output_stream,
        })
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("[VOICE] Stream error: {}", err);
}
