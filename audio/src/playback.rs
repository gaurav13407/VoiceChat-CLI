use cpal::traits::{DeviceTrait,Hosttrait,StreamTrait};
use crossbeam_channel::{Receiver,TryRecvError};

//start audio playback from received audio frame 
//this function blocks until enter is pressed.

pub fn start_playback(rx:Receiver<Vec<f32>>){
    let host=cpal::default_host();
    let output=host 
        .default_output_device()
        .expect("No output device is available");

    let cfg=output.default_output_config()
        .expect("Failed to get output config");

    println!("Playback device:{}",output.name().unwrap());
    println!("Playback format:{:?}",cfg.sample_format());
    println!("PLayback rate: {}",cfg.sample_format().0);
    println!("Playback chans :{}",cfg.channels());


    let stream_cfg=cpal::StreamConfig{
        channels:cfg.channels(),
        sample_rate:cfg.sample_rate(),
        buffer_size:cpal::BufferSize::Default,
    };

    let channels=cfg.channels() as usize;
    let mut playback_acc:Vec<f32>=Vec::new();


    let stream=match cfg.sample_format(){
        cpal::SampleFormat::F32=>output
            .build_output_stream(
                &stream_cfg,
                move |out: &mut [f32],_|{
                    write_output(out,channels,&rx,&mut playback_acc);
                },
                err_fn,
                None,
            )
            .unwrap(),

            cpal::SampleFormat::I16=>output.build_output_stream(
                &stream_cfg,
                move|out:&mut[i16],_|{
                    write_output_i16(out,channels,&rx,&mut playback_acc);
                },
                err_fn,
                None,
            )
                .unwrap(),
                _ =>panic!("Unsupported output format"),
    };

    stream.play().unwrap();
    println!("Playback running.Press Enter to stop.");
    let _=std::io::stdin().read_line(&mut String::new());
}

//Helpers 
fn write_output(
    out:&mut[f32],
    channels:usize,
    rx:&Receiver<Vec<f32>>,
    acc:&mut Vec<f32>,
){
    let mut written=0;

    while written<out.len(){
        if acc.is_empty(){
            match rx.try_recv(){
                Ok(frame)=>*acc=frame,
                Err(TryRecvError::Empty)=>{
                    out[written..].fill(0.0);
                    return;
                }
                Err(_)=>return,
            }
        }

        let frame=(out.len()-written)/channels;
        let n=frames.min(acc.len());

        for i in 0..n{
            for ch in 0..channels{
                out[written+i*channels+ch]=acc[i];
            }
        }
        acc.drain(..n);
        written+=n*channels;
    }
}

fn write_output_i16(
    out: &mut [i16],
    channels: usize,
    rx: &Receiver<Vec<f32>>,
    acc: &mut Vec<f32>,
) {
    let mut written = 0;

    while written < out.len() {
        if acc.is_empty() {
            match rx.try_recv() {
                Ok(frame) => *acc = frame,
                Err(TryRecvError::Empty) => {
                    out[written..].fill(0);
                    return;
                }
                Err(_) => return,
            }
        }

        let frames = (out.len() - written) / channels;
        let n = frames.min(acc.len());

        for i in 0..n {
            let sample = (acc[i] * i16::MAX as f32) as i16;
            for ch in 0..channels {
                out[written + i * channels + ch] = sample;
            }
        }

        acc.drain(..n);
        written += n * channels;
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("Playback error: {}", err);
}

