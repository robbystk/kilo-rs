/*** includes ***/
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;

extern crate termios;
use termios::*;

extern crate libc;
use libc::{ioctl, winsize, TIOCGWINSZ};

/*** macros ***/
macro_rules! ctrl_key {
    ($k:literal) => {($k) as u8 & 0x1f}
}

/*** data ***/

/// Stores editor configuration such as terminal size
struct EditorConfig {
    orig_termios: Termios,
    rows: u16,
    cols: u16,
}

impl EditorConfig {
    /// Initializes the configuration
    ///
    /// Includes enabling raw mode and saving the original terminal
    /// configuration for restoration upon exit.
    fn setup() -> EditorConfig {
        let (rows, cols) = get_window_size();

        EditorConfig {
            orig_termios: enable_raw_mode(),
            rows,
            cols,
        }
    }
}

impl Drop for EditorConfig {
    fn drop(&mut self) {
        // clear screen and restore terminal settings
        let mut stdout = io::stdout();
        stdout.write(b"\x1b[2J").unwrap();
        stdout.write(b"\x1b[H").unwrap();
        stdout.flush().unwrap();
        reset_mode(self.orig_termios);
    }
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

fn get_window_size() -> (u16, u16) {
    let mut ws = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    unsafe {
        ioctl(io::stdin().as_raw_fd(), TIOCGWINSZ, &mut ws);
    }

    (ws.ws_row, ws.ws_col)
}

/*** output ***/

/// Draw each row of the screen
///
/// Currently we have no lines, so it just draws a tilde at the beginning of
/// each line, like vim.
fn editor_draw_rows(config: &EditorConfig) {
    let mut stdout = io::stdout();

    for i in 0..config.rows {
        stdout.write(b"~").unwrap();
        if i < config.rows - 1 {
            stdout.write(b"\r\n").unwrap();
        }
    }
    stdout.flush().unwrap();
}

/// Refresh the text on the screen
fn editor_refresh_screen(config: &EditorConfig) {
    let mut stdout = io::stdout();
    // clear screen
    stdout.write(b"\x1b[2J").unwrap();
    // move cursor to top left
    stdout.write(b"\x1b[H").unwrap();
    // draw a column of tildes like vim
    editor_draw_rows(config);
    stdout.write(b"\x1b[H").unwrap();
    // make sure things get written
    stdout.flush().unwrap()
}

/*** input ***/

/// Read and process a keypress
///
/// Currently handls Ctrl-q to quit logic and prints the character or character
/// code if it's a control character.
fn editor_process_keypress() -> bool {
    let c = editor_read_key();

    // quit on Ctrl-q
    c == ctrl_key!('q')
}

/*** init ***/

fn main() {
    let cfg = EditorConfig::setup();

    loop {
        editor_refresh_screen(&cfg);
        if editor_process_keypress() {break;}
    }

    reset_mode(cfg.orig_termios);
}
