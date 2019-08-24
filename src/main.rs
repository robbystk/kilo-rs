/*** includes ***/
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::process::exit;

extern crate termios;
use termios::*;

/*** macros ***/
macro_rules! ctrl_key {
    ($k:literal) => {($k) as u8 & 0x1f}
}

/*** terminal ***/

/// Enables raw mode in the terminal
///
/// This includes setting a timeout of 0.1 seconds for reading stdin.  Saves and
/// returns the original configuration so that the calling code can return the
/// terminal to its original state using reset_mode() below.
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

/// Reset the terminal to its original state
fn reset_mode(orig_mode: Termios) {
    let stdin = io::stdin().as_raw_fd();

    tcsetattr(stdin, TCSAFLUSH, & orig_mode).unwrap();
}

/// Read a keypress from stdin
///
/// Waits until a keypress or error is recieved.
fn editor_read_key() -> u8 {
    loop {
        if let Some(r) = io::stdin().bytes().next() {
            return r.expect("read error");
        }
    }
}

/*** output ***/

fn editor_refresh_screen() {
    let mut stdout = io::stdout();
    // clear screen
    stdout.write(b"\x1b[2J").unwrap();
    // make sure things get written
    stdout.flush().unwrap()
}

/*** input ***/

/// Read and process a keypress
///
/// Currently handls Ctrl-q to quit logic and prints the character or character
/// code if it's a control character.
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

/*** init ***/

fn main() {
    let orig_termios = enable_raw_mode();

    loop {
        editor_refresh_screen();
        editor_process_keypress(orig_termios);
    }
}
