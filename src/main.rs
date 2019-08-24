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

/*** data ***/
struct EditorConfig {
    orig_termios: Option<Termios>,
}

static mut E: EditorConfig = EditorConfig {
    orig_termios: None,
};

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

/// Draw each row of the screen
fn editor_draw_rows() {
    let mut stdout = io::stdout();

    for _ in 0..24 {
        stdout.write(b"~\r\n").unwrap();
    }
}

/// Refresh the text on the screen
fn editor_refresh_screen() {
    let mut stdout = io::stdout();
    // clear screen
    stdout.write(b"\x1b[2J").unwrap();
    // move cursor to top left
    stdout.write(b"\x1b[H").unwrap();
    // draw a column of tildes like vim
    editor_draw_rows();
    stdout.write(b"\x1b[H").unwrap();
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
        println!("wat");
        let mut stdout = io::stdout();
        stdout.write(b"\x1b[2J").unwrap();
        stdout.write(b"\x1b[H").unwrap();
        stdout.flush().unwrap();
        reset_mode(orig);
        exit(0);
    }
}

/*** init ***/

fn main() {
    unsafe {
        E.orig_termios = Some(enable_raw_mode());
    };

    loop {
        editor_refresh_screen();
        unsafe {
            editor_process_keypress(E.orig_termios.unwrap());
        }
    }
}
