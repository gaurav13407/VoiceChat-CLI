use rand::Rng;

const CODE_LEN:usize=4;
const CHARSET:&[u8]=b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

///Generate a room code like XXXX-YYYY
pub fn generate_room_code()->String{
    let mut rng=rand::thread_rng();

    let part1:String=(0..CODE_LEN)
        .map(|_| {
            let idx=rng.gen_range(0,CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    let part2:String=(0..CODE_LEN)
        .map(|_| {
            let idx=rng.gen_range(0,CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("{}-{}",part1,part2)
}


//Validate room code format XXXX-YYYY
pub fn validate_room_code(code:&str)->bool{
    let parts:Vec<&str>=code.split('-').collect();
    if parts.len()!=2{
        return false;
    }
    for part in parts{
        if part.len()!=CODE_LEN{
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_uppercase()||c.is_ascii_digit()){
            return false;
        }
    }
    true
}
