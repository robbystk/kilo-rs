extern crate termios;

use std::io;
use std::io::Read;
use std::os::unix::io::AsRawFd;

use termios::*;

fn enable_raw_mode() {
    let stdin = io::stdin().as_raw_fd();

    let mut raw = Termios::from_fd(stdin).unwrap();

    raw.c_lflag &= !(ECHO);

    tcsetattr(stdin, TCSAFLUSH, & mut raw).unwrap();
}

fn main() {
    enable_raw_mode();

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
