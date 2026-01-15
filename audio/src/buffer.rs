use std::collections::BTreeMap;
//JitterBuffer reorders audio frame by sequences number 
// and outputs them in order with bounded latency 
pub struct JitterBuffer{
    buffer:BTreeMap<u32,Vec<f32>>,
    next_seq:u32,
    capacity:usize,
}

impl JitterBuffer{
    //Create a new JitterBuffer 
    //start_seq
    //capacity=max frame to hold 
    pub fn new(start_seq:u32,capacity:usize)->Self{
        Slef{
            buffer:BTreeMap::new(),
            next_seq:start_seq,
            capacity,
        }
    }

    //push a frame into buffer 
    //late frame drop autmatically 
    pub fn push(&mut self,seq:u32,frame:Vec<f32>){
        //Drop the packte that are too late 
        if seq<self.next_seq{
            return;
        }
        //Prevent unbounded growth 
        while self.buffer.len()>self.capacity{
            self.buffer.pop_first();
        }
    }

    //POp the next in order-frame 
    pub fn pop (&mut self)->Option<Vec<f32>>{
        if let Some(frame)=self.buffer.remove(&self.next_seq){
            self.next_seq+=1;
            Some(frame)
        }else{
            None
        }
    }
}
