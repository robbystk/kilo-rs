extern crate termios;

use std::io;
use std::io::Read;
use std::os::unix::io::AsRawFd;

use termios::*;

macro_rules! ctrl_key {
    ($k:literal) => {($k) as u8 & 0x1f}
}

fn enable_raw_mode() -> Termios {
    let stdin = io::stdin().as_raw_fd();

    let orig_termios = Termios::from_fd(stdin).expect("tcgetattr");
    let mut raw = orig_termios;

    raw.c_lflag &= !(ECHO | ICANON | ISIG | IEXTEN);
    raw.c_iflag &= !(IXON | ICRNL | BRKINT | INPCK | ISTRIP);
    raw.c_oflag &= !(OPOST);
    raw.c_cflag |= CS8;
    raw.c_cc[VMIN] = 0;
    raw.c_cc[VTIME] = 1;

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
        let c = match io::stdin().bytes().next() {
            None => '\0' as u8,
            Some(r) => r.expect("read error"),
        };

        // quit on 'q'
        if c == ctrl_key!('q') {
            break;
        }

        // print character
        if char::from(c).is_ascii_control() {
            print!("{}\r\n", c);
        } else {
            print!("{} ({})\r\n", c, char::from(c));
        }
    }

    reset_mode(orig_termios);
}
