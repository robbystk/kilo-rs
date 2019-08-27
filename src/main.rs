/*** includes ***/
use std::io;
use std::io::{Read, Write, Error, ErrorKind};
use std::os::unix::io::AsRawFd;
use std::str;

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
        let orig_termios = enable_raw_mode();
        let (rows, cols) = get_window_size()
            .expect("Could not get window size");

        EditorConfig {
            orig_termios,
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

fn get_cursor_position() -> Result<(u16, u16), std::io::Error> {
    // TODO: rework error handling
    io::stdout().write(b"\x1b[6n").unwrap();
    io::stdout().flush().unwrap();
    let cpr: Vec<u8> = io::stdin().bytes()
        .fuse()
        .map(|e| e.unwrap())
        .collect();

    if cpr[0] != '\x1b' as u8 || cpr[1] != '[' as u8 {
        return Err(Error::new(ErrorKind::Other,
            "invalid character position report"));
    }
    let data: Vec<u16> = str::from_utf8(&cpr[1..]).unwrap()
        .trim_matches(|c| c == 'R' || c == '[')
        .split(';')
        .map(|s| s.parse().expect("parse error"))
        .collect();

    Ok((data[0], data[1]))
}

fn get_window_size() -> Result<(u16, u16), std::io::Error> {
    let mut ws = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let mut ret_val = -1;
    unsafe {
        ret_val = ioctl(io::stdin().as_raw_fd(), TIOCGWINSZ, &mut ws);
    }

    if ret_val == -1 || ws.ws_row == 0 || ws.ws_col == 0 {
        io::stdout().write(b"\x1b[999B\x1b[999C").unwrap();
        return get_cursor_position();
    }

    Ok((ws.ws_row, ws.ws_col))
}

/*** output ***/

/// Draw each row of the screen
///
/// Currently we have no lines, so it just draws a tilde at the beginning of
/// each line, like vim.
fn editor_draw_rows(config: &EditorConfig, buf: &mut String) {
    for i in 0..config.rows {
        buf.push('~');
        // clear remainder of row
        buf.push_str("\x1b[K");
        if i < config.rows - 1 {
            buf.push_str("\r\n");
        }
    }
}

/// Refresh the text on the screen
fn editor_refresh_screen(config: &EditorConfig) {
    let mut buf = String::from("");

    // hide cursor
    buf.push_str("\x1b[?25l");
    // move cursor to top left
    buf.push_str("\x1b[H");
    // draw a column of tildes like vim
    editor_draw_rows(config, &mut buf);
    // move cursor back to upper left
    buf.push_str("\x1b[H");
    // show cursor
    buf.push_str("\x1b[?25h");

    io::stdout().write(&buf.as_bytes()).unwrap();
    // make sure things get written
    io::stdout().flush().unwrap()
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
