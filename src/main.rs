extern crate termios;

use std::io;
use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::process::exit;

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

fn editor_read_key() -> u8 {
    loop {
        if let Some(r) = io::stdin().bytes().next() {
            return r.expect("read error");
        }
    }
}

fn editor_process_keypress(orig: Termios) {
    let c = editor_read_key();

    // quit on Ctrl-q
    if c == ctrl_key!('q') {
        reset_mode(orig);
        exit(0);
    }

    // print character
    if char::from(c).is_ascii_control() {
        print!("{}\r\n", c);
    } else {
        print!("{} ({})\r\n", c, char::from(c));
    }
}

fn main() {
    let orig_termios = enable_raw_mode();

    loop {
        editor_process_keypress(orig_termios);
    }
}
