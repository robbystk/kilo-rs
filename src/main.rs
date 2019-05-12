extern crate termios;

use std::io;
use std::io::Read;
use std::os::unix::io::AsRawFd;

use termios::*;

fn enable_raw_mode() -> Termios {
    let stdin = io::stdin().as_raw_fd();

    let orig_termios = Termios::from_fd(stdin).unwrap();
    let mut raw = orig_termios;

    raw.c_lflag &= !(ECHO);

    tcsetattr(stdin, TCSAFLUSH, & mut raw).unwrap();

    orig_termios
}

fn reset_mode(orig_mode: Termios) {
    let stdin = io::stdin().as_raw_fd();

    tcsetattr(stdin, TCSAFLUSH, & orig_mode).unwrap();
}

fn main() {
    let orig_termios = enable_raw_mode();

    loop {
        if let Some(Ok(c)) = io::stdin().bytes().next() {
            if c == 'q' as u8 {
                break;
            }
        } else {
            break;
        }
    }

    reset_mode(orig_termios);
}
