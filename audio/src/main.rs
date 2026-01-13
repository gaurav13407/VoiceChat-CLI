use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{bounded, TryRecvError};

const FRAME_MS: usize = 20;
const BUFFER_FRAMES: usize = 3;

fn main() {
    let host = cpal::default_host();

    let input = host
        .input_devices()
        .unwrap()
        .find(|d| d.name().unwrap().contains("pipewire"))
        .expect("No pipewire input");

    let output = host
        .output_devices()
        .unwrap()
        .find(|d| d.name().unwrap().contains("pipewire"))
        .expect("No pipewire output");

    println!("Using input  : {}", input.name().unwrap());
    println!("Using output : {}", output.name().unwrap());

    let input_cfg = input.default_input_config().unwrap();
    let output_cfg = output.default_output_config().unwrap();

    println!("Input format  : {:?}", input_cfg.sample_format());
    println!("Output format : {:?}", output_cfg.sample_format());

    let sample_rate = input_cfg.sample_rate().0 as usize;
    let in_channels = input_cfg.channels() as usize;
    let out_channels = output_cfg.channels() as usize;
    let frame_samples = sample_rate * FRAME_MS / 1000;

    println!("Sample rate   : {}", sample_rate);
    println!("Frame samples : {}", frame_samples);

    let stream_config = cpal::StreamConfig {
        channels: in_channels as u16,
        sample_rate: input_cfg.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };

    let (tx, rx) = bounded::<Vec<f32>>(BUFFER_FRAMES);

    match input_cfg.sample_format() {
        cpal::SampleFormat::F32 => {
            run_f32(
                input,
                output,
                stream_config,
                out_channels,
                frame_samples,
                tx,
                rx,
            );
        }
        cpal::SampleFormat::I16 => {
            run_i16(
                input,
                output,
                stream_config,
                out_channels,
                frame_samples,
                tx,
                rx,
            );
        }
        _ => panic!("Unsupported sample format"),
    }
}

/* ================= F32 PATH ================= */

fn run_f32(
    input: cpal::Device,
    output: cpal::Device,
    config: cpal::StreamConfig,
    out_channels: usize,
    frame_samples: usize,
    tx: crossbeam_channel::Sender<Vec<f32>>,
    rx: crossbeam_channel::Receiver<Vec<f32>>,
) {
    let mut capture_acc: Vec<f32> = Vec::new();
    let mut playback_acc: Vec<f32> = Vec::new();
    let tx_cap = tx.clone();

    let input_stream = input
        .build_input_stream(
            &config,
            move |data: &[f32], _| {
                let channels = config.channels as usize;

                for frame in data.chunks(channels) {
                    let sum: f32 = frame.iter().sum();
                    capture_acc.push(sum / channels as f32);
                }

                while capture_acc.len() >= frame_samples {
                    let frame =
                        capture_acc.drain(..frame_samples).collect();
                    let _ = tx_cap.try_send(frame);
                }
            },
            err_fn,
            None,
        )
        .unwrap();

    let output_stream = output
        .build_output_stream(
            &config,
            move |out: &mut [f32], _| {
                let mut written = 0;

                while written < out.len() {
                    if playback_acc.is_empty() {
                        match rx.try_recv() {
                            Ok(frame) => playback_acc = frame,
                            Err(TryRecvError::Empty) => {
                                out[written..].fill(0.0);
                                return;
                            }
                            Err(_) => return,
                        }
                    }

                    let frames =
                        (out.len() - written) / out_channels;
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

    println!("Voice loopback running. Press ENTER to stop.");
    let _ = std::io::stdin().read_line(&mut String::new());
}

/* ================= I16 PATH ================= */

fn run_i16(
    input: cpal::Device,
    output: cpal::Device,
    config: cpal::StreamConfig,
    out_channels: usize,
    frame_samples: usize,
    tx: crossbeam_channel::Sender<Vec<f32>>,
    rx: crossbeam_channel::Receiver<Vec<f32>>,
) {
    let mut capture_acc: Vec<f32> = Vec::new();
    let mut playback_acc: Vec<f32> = Vec::new();
    let tx_cap = tx.clone();

    let input_stream = input
        .build_input_stream(
            &config,
            move |data: &[i16], _| {
                let channels = config.channels as usize;

                for frame in data.chunks(channels) {
                    let mut sum = 0.0;
                    for s in frame {
                        sum += *s as f32 / i16::MAX as f32;
                    }
                    capture_acc.push(sum / channels as f32);
                }

                while capture_acc.len() >= frame_samples {
                    let frame =
                        capture_acc.drain(..frame_samples).collect();
                    let _ = tx_cap.try_send(frame);
                }
            },
            err_fn,
            None,
        )
        .unwrap();

    let output_stream = output
        .build_output_stream(
            &config,
            move |out: &mut [i16], _| {
                let mut written = 0;

                while written < out.len() {
                    if playback_acc.is_empty() {
                        match rx.try_recv() {
                            Ok(frame) => playback_acc = frame,
                            Err(TryRecvError::Empty) => {
                                out[written..].fill(0);
                                return;
                            }
                            Err(_) => return,
                        }
                    }

                    let frames =
                        (out.len() - written) / out_channels;
                    let n = frames.min(playback_acc.len());

                    for i in 0..n {
                        let sample =
                            (playback_acc[i] * i16::MAX as f32)
                                as i16;
                        for ch in 0..out_channels {
                            out[written + i * out_channels + ch] =
                                sample;
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

    println!("Voice loopback running. Press ENTER to stop.");
    let _ = std::io::stdin().read_line(&mut String::new());
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("Stream error: {}", err);
}

