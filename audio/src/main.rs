mod buffer;
mod voice;
mod upd;
mod playback;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::bounded;
use std::env;

/* ================= CONFIG ================= */

const FRAME_MS: usize = 20;
const BUFFER_FRAMES: usize = 4;
const VOLUME_GAIN: f32 = 1.5;

/* ================= MAIN ================= */

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage:");
        eprintln!("  <sender_id> <send_addr> <recv_bind>");
        eprintln!("Example:");
        eprintln!("  1 127.0.0.1:9002 0.0.0.0:9001");
        return;
    }

    let sender_id: u32 = args[1].parse().unwrap();
    let send_addr = &args[2];
    let recv_bind = &args[3];

    let host = cpal::default_host();

    /* ==== INPUT DEVICE (Pulse / Bluetooth) ==== */

    let input = host
        .input_devices()
        .unwrap()
        .find(|d| {
            let n = d.name().unwrap().to_lowercase();
            n == "pulse" || n == "default"
        })
        .expect("No input device");

    let output = host.default_output_device().expect("No output device");

    let input_cfg = input.default_input_config().unwrap();
    let output_cfg = output.default_output_config().unwrap();

    let sample_rate = input_cfg.sample_rate().0 as usize;
    let channels = input_cfg.channels() as usize;
    let frame_samples = sample_rate * FRAME_MS / 1000;

    println!("Sender ID      : {}", sender_id);
    println!("Sample rate    : {}", sample_rate);
    println!("Channels       : {}", channels);
    println!("Frame samples  : {}", frame_samples);
    println!("Send to        : {}", send_addr);
    println!("Recv bind      : {}", recv_bind);

    let (tx_play, rx_play) = bounded::<Vec<f32>>(BUFFER_FRAMES);

    /* ================= UDP NETWORK ================= */
    
    // Start UDP networking with jitter buffer
    let mut udp_handle = upd::start_udp(
        sender_id,
        send_addr,
        recv_bind,
        tx_play.clone(),
    )?;

    /* ================= INPUT STREAM (CAPTURE & SEND) ================= */

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
                    let frame: Vec<f32> =
                        capture_acc.drain(..frame_samples).collect();
                    
                    // Send frame using UDP handle with VoicePacket encoding
                    udp_handle.send_frame(frame);
                }
            },
            err_fn,
            None,
        )
        .unwrap();

    /* ================= OUTPUT STREAM (PLAYBACK) ================= */

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
                            out[written + i * out_channels + ch] =
                                playback_acc[i];
                        }
                    }

                    playback_acc.drain(..n);
                    written += n * out_channels;
                }
            },
            err_fn,
            None,
        )
        .unwrap();

    input_stream.play().unwrap();
    output_stream.play().unwrap();

    println!("Voice chat running. Press ENTER to stop.");
    let _ = std::io::stdin().read_line(&mut String::new());
}

/* ================= ERROR ================= */

fn err_fn(err: cpal::StreamError) {
    eprintln!("Stream error: {}", err);
}
