/*** includes ***/
use std::env::args;
use std::io;
use std::io::{Write};
use std::str;

extern crate termios;
use termios::Termios;

extern crate libc;

mod input;
mod output;
mod terminal;
mod file_io;

use input::{EditorKey, editor_read_key};
use terminal::{enable_raw_mode, reset_mode, get_window_size};
use file_io::{editor_open};

/*** macros ***/
macro_rules! ctrl_key {
    ($k:literal) => {($k) as u8 & 0x1f}
}

/*** data ***/
const KILO_VERSION: &str = "0.0.1";

/// Stores editor configuration such as terminal size
pub struct EditorConfig {
    cx: usize,
    cy: usize,
    orig_termios: Termios,
    rows: usize,
    cols: usize,
    erows: Vec<String>,
}

impl EditorConfig {
    /// Initializes the configuration
    ///
    /// Includes enabling raw mode and saving the original terminal
    /// configuration for restoration upon exit.
    fn setup(filename: &Option<&str>) -> EditorConfig {
        let orig_termios = enable_raw_mode();
        let (rows, cols) = get_window_size()
            .expect("Could not get window size");

        let erows = match filename {
            Some(filename) => editor_open(filename).unwrap(),
            None => Vec::new(),
        };

        EditorConfig {
            cx: 0,
            cy: 0,
            orig_termios,
            rows,
            cols,
            erows,
        }
    }

    /// Move cursor as appropriate given a key
    fn move_cursor(&mut self, key: &EditorKey) {
        use EditorKey::*;
        match key {
            ArrowLeft | Char(b'h') => { if self.cx > 0 { self.cx -= 1; } },
            ArrowDown | Char(b'j') => { if self.cy < self.rows - 1{ self.cy += 1; } },
            ArrowUp | Char(b'k') => { if self.cy > 0 { self.cy -= 1; } },
            ArrowRight | Char(b'l') => { if self.cx < self.cols - 1 { self.cx += 1; } },
            PageUp => for _ in 0..self.rows { self.move_cursor(&ArrowUp) },
            PageDown => for _ in 0..self.rows { self.move_cursor(&ArrowDown) },
            Home => self.cx = 0,
            End => self.cx = self.cols - 1,
            _ => ()
        }
    }

    /// Read and process a keypress
    ///
    /// Returns true if it's time to stop
    fn process_keypress(&mut self) -> bool {
        match editor_read_key() {
            Some(EditorKey::Char(c)) if c == ctrl_key!(b'q') => true,
            Some(k) => {
                self.move_cursor(&k);
                false
            },
            None => false,
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

/*** init ***/

pub fn run() {
    let argv = args().collect::<Vec<String>>();
    let filename = if argv.len() > 1 {
        Some(argv[1].as_str())
    } else {
        None
    };
    let mut cfg = EditorConfig::setup(&filename);

    loop {
        output::editor_refresh_screen(&cfg);
        if cfg.process_keypress() { break; }
    }

    reset_mode(cfg.orig_termios);
}
