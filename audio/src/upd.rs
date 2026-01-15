use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

use crossbeam_channel::Sender;

use crate::buffer::JitterBuffer;
use crate::time::voice::VoicePacket;

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
)->UdpHandle{
    let send_socket=UdpSocket::bind("0.0.0.0:0")
        .expect("Failed to connect send socket");
    send_socket
        .connect(sender_addr)
        .expect("Failed to connect send socket");
    let recv_socket=UdpSocket::bind(recv_bind)
        .expect("Failed to bind recv  socket");
    recv_socket
        .set_nonblocking(true)
        .expect("Failed to set nonblocking");

    let recv_socket=recv_socket.try_clone().unwrap();


    //Receiver thread 
    thread::spawn(move ||{
        let mut buf=[0u8;65536];
        let mut jitter:Option<JitterBuffer>=None;

        loop{
            match recv_socket(&mut buf){
                Ok(n)=>{
                    if let Some(pkt)=VoicePacket::decode(&buf[..n]){
                        //Drop our own packets
                        if pkt.sender_id==sender_id{
                            continue;
                        }
                        let jb=jitter.get_or_insert_with(|| JitterBuffer::new(pkt.seq,3));
                        jb.push(pkt.seq,pkt.samples);

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

    UdpHandle{
        sender_id,
        socket:send_socket,
        seq:0,
    }


}



//Handle used by audio capture to send frame 
pub struct UdpHandle{
    sender_id:u32,
    socket:UdpSocket,
    seq:u32,
}

impl UdpHandle{
    //Send one audio frame over UDP 
    pub fn send_frame(&mut self,samples:Vec<f32>){
        let pkt=VoicePacket{
            sender_id:self.sender_id,
            seq:self.seq,
            samples,
        };

        let data=pkt.encoded();
        let _=self.socket.send(&data);

        self.seq=self.seq.wrapping_add(1);
    }
}
