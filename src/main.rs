use std::io;
use std::io::Read;

fn main() {
    loop {
        if let Some(Ok(c)) = io::stdin().bytes().next() {
            if c == 'q' as u8 {
                break;
            }
        } else {
            break;
        }
    }
}
