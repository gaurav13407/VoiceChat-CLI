use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{bounded, TryRecvError};

const FRAME_MS: usize = 20;
const BUFFER_FRAMES: usize = 3;
const VOLUME_GAIN: f32 = 2.0; // Amplify volume

fn main() {
    let host = cpal::default_host();

    // List all available input devices
    println!("Available input devices:");
    let device_list: Vec<_> = host.input_devices().unwrap().collect();
    for (i, device) in device_list.iter().enumerate() {
        let name = device.name().unwrap();
        println!("  [{}] {}", i, name);
    }

    // Use pulse - it routes to whatever is the active recording device in pavucontrol
    let input = device_list
        .iter()
        .find(|d| {
            let name = d.name().unwrap().to_lowercase();
            name == "pulse" || name == "default"
        })
        .expect("No input device available")
        .clone();

    let output = host
        .default_output_device()
        .expect("No output device available");

    println!("\nUsing input  : {}", input.name().unwrap());
    println!("Using output : {}", output.name().unwrap());
    println!("\n=== IMPORTANT ===");
    println!("While this program is running:");
    println!("1. Open pavucontrol (run 'pavucontrol' in another terminal)");
    println!("2. Go to the 'Recording' tab");
    println!("3. Find this program ('ALSA plug-in')"); 
    println!("4. Select your headphone: 'Family 17h/19h HD Audio Controller Analog Stereo'");
    println!("5. Speak into your headphone - you should hear yourself!");
    println!("================\n");

    let input_cfg = input.default_input_config().unwrap();
    let output_cfg = output.default_output_config().unwrap();

    println!("Input format  : {:?}", input_cfg.sample_format());
    println!("Output format : {:?}", output_cfg.sample_format());

    let sample_rate = input_cfg.sample_rate().0 as usize;
    let in_channels = input_cfg.channels() as usize;
    let out_channels = output_cfg.channels() as usize;
    let frame_samples = sample_rate * FRAME_MS / 1000;

    println!("Sample rate   : {}", sample_rate);
    println!("Input channels: {}", in_channels);
    println!("Output channels: {}", out_channels);
    println!("Frame samples : {}", frame_samples);

    let input_stream_config = cpal::StreamConfig {
        channels: in_channels as u16,
        sample_rate: input_cfg.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };

    let output_stream_config = cpal::StreamConfig {
        channels: out_channels as u16,
        sample_rate: output_cfg.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };

    let (tx, rx) = bounded::<Vec<f32>>(BUFFER_FRAMES);

    match input_cfg.sample_format() {
        cpal::SampleFormat::F32 => {
            run_f32(
                input,
                output,
                input_stream_config,
                output_stream_config,
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
                input_stream_config,
                output_stream_config,
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
    input_config: cpal::StreamConfig,
    output_config: cpal::StreamConfig,
    out_channels: usize,
    frame_samples: usize,
    tx: crossbeam_channel::Sender<Vec<f32>>,
    rx: crossbeam_channel::Receiver<Vec<f32>>,
) {
    let mut capture_acc: Vec<f32> = Vec::new();
    let mut playback_acc: Vec<f32> = Vec::new();
    let tx_cap = tx.clone();
    let mut frames_sent = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let frames_sent_clone = frames_sent.clone();

    let input_stream = input
        .build_input_stream(
            &input_config,
            move |data: &[f32], _| {
                let channels = input_config.channels as usize;

                for frame in data.chunks(channels) {
                    let sum: f32 = frame.iter().sum();
                    let sample = (sum / channels as f32) * VOLUME_GAIN;
                    capture_acc.push(sample.clamp(-1.0, 1.0));
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
            &output_config,
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
    input_config: cpal::StreamConfig,
    output_config: cpal::StreamConfig,
    out_channels: usize,
    frame_samples: usize,
    tx: crossbeam_channel::Sender<Vec<f32>>,
    rx: crossbeam_channel::Receiver<Vec<f32>>,
) {
    let mut capture_acc: Vec<f32> = Vec::new();
    let mut playback_acc: Vec<f32> = Vec::new();
    let tx_cap = tx.clone();
    let mut frames_sent = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let frames_sent_clone = frames_sent.clone();

    let input_stream = input
        .build_input_stream(
            &input_config,
            move |data: &[i16], _| {
                let channels = input_config.channels as usize;

                for frame in data.chunks(channels) {
                    let mut sum = 0.0;
                    for s in frame {
                        sum += *s as f32 / i16::MAX as f32;
                    }
                    let sample = (sum / channels as f32) * VOLUME_GAIN;
                    capture_acc.push(sample.clamp(-1.0, 1.0));
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
            &output_config,
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

