use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

use crossbeam_channel::Sender;
use opus::{Application, Channels, Encoder, Decoder};

use crate::buffer::JitterBuffer;
use crate::voice::VoicePacket;

//startup UDP networking:
//send audio frame
//receives packets
//apllies jitter buffer 
//outputs ordered frame to playback 
pub fn start_udp(
    sender_id:u32,
    sender_addr:&str,
    recv_bind:&str,
    playback_tx:Sender<Vec<f32>>,
)->anyhow::Result<UdpHandle>{
    let send_socket=UdpSocket::bind("0.0.0.0:0")?;
    send_socket.connect(sender_addr)?;
    
    let recv_socket=UdpSocket::bind(recv_bind)?;
    recv_socket.set_nonblocking(true)?;

    let recv_socket=recv_socket.try_clone()?;


    //Receiver thread 
    thread::spawn(move ||{
        let mut buf=[0u8;65536];
        let mut jitter:Option<JitterBuffer>=None;
        
        // Create Opus decoder
        let mut decoder = Decoder::new(
            48_000,
            Channels::Mono,
        ).expect("Failed to create Opus decoder");

        loop{
            match recv_socket.recv_from(&mut buf){
                Ok((n,_))=>{
                    if let Some(pkt)=VoicePacket::decode(&buf[..n]){
                        //Drop our own packets
                        if pkt.sender_id==sender_id{
                            continue;
                        }
                        
                        // Decode Opus to PCM
                        let mut pcm = vec![0f32; 960]; // 20 ms @ 48kHz
                        let decoded = decoder
                            .decode_float(&pkt.payload, &mut pcm, false)
                            .unwrap_or(0);
                        pcm.truncate(decoded);
                        
                        let jb=jitter.get_or_insert_with(|| JitterBuffer::new(pkt.seq,3));
                        jb.push(pkt.seq, pcm);

                        //Drain ready Frame 
                        while let Some(frame)=jb.pop(){
                            let _=playback_tx.try_send(frame);
                        }
                    }
                }

                Err(_)=>{
                    thread::sleep(Duration::from_millis(1));

                }
            }
        }
    });

    // Create Opus encoder (48kHz for CD-quality audio)
    let encoder = Encoder::new(
        48_000,
        Channels::Mono,
        Application::Voip,
    )?;

    Ok(UdpHandle{
        sender_id,
        socket:send_socket,
        seq:0,
        encoder,
    })
}



//Handle used by audio capture to send frame 
pub struct UdpHandle{
    sender_id:u32,
    socket:UdpSocket,
    seq:u32,
    encoder:Encoder,
}

impl UdpHandle{
    //Send one audio frame over UDP 
    pub fn send_frame(&mut self, mut pcm: Vec<f32>) {
        // Opus at 48kHz expects 960 samples for 20ms
        // We get 882 samples @ 44.1kHz, so pad with zeros
        while pcm.len() < 960 {
            pcm.push(0.0);
        }
        pcm.truncate(960);
        
        let mut out = vec![0u8; 4000]; // enough for any Opus frame

        let len = self.encoder
            .encode_float(&pcm, &mut out)
            .expect("Opus encode failed");

        out.truncate(len);

        let pkt = VoicePacket {
            sender_id: self.sender_id,
            seq: self.seq,
            payload: out,
        };

        let _ = self.socket.send(&pkt.encode());
        self.seq = self.seq.wrapping_add(1);
    }
}
