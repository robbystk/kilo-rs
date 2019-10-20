use std::io;
use std::io::{Read, Write, Error, ErrorKind};
use std::os::unix::io::AsRawFd;
use std::str;
use termios::{BRKINT, CS8, ECHO, ICANON, ISIG, IEXTEN, IXON, ICRNL, INPCK,
    ISTRIP, OPOST, Termios, tcsetattr, TCSAFLUSH, VMIN, VTIME};
use libc::{ioctl, winsize, TIOCGWINSZ};

/// Enables raw mode in the terminal
///
/// This includes setting a timeout of 0.1 seconds for reading stdin.  Saves and
/// returns the original configuration so that the calling code can return the
/// terminal to its original state using reset_mode() below.
pub fn enable_raw_mode() -> Termios {
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
pub fn reset_mode(orig_mode: Termios) {
    let stdin = io::stdin().as_raw_fd();

    tcsetattr(stdin, TCSAFLUSH, & orig_mode).unwrap();
}

pub fn get_cursor_position() -> Result<(usize, usize), std::io::Error> {
    // TODO: rework error handling
    io::stdout().write(b"\x1b[6n").unwrap();
    io::stdout().flush().unwrap();

    // cursor position report
    let cpr: Vec<u8> = io::stdin().bytes()
        .fuse()
        .map(|e| e.unwrap())
        .collect();

    if cpr[0] != b'\x1b' || cpr[1] != b'[' {
        return Err(Error::new(ErrorKind::Other,
            "invalid cursor position report"));
    }
    let data: Vec<usize> = str::from_utf8(&cpr[1..]).unwrap()
        .trim_matches(|c| c == 'R' || c == '[')
        .split(';')
        .map(|s| s.parse().expect("parse error"))
        .collect();

    Ok((data[0], data[1]))
}

pub fn get_window_size() -> Result<(usize, usize), std::io::Error> {
    let mut ws = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let ret_val = unsafe {
        ioctl(io::stdin().as_raw_fd(), TIOCGWINSZ, &mut ws)
    };

    if ret_val == -1 || ws.ws_row == 0 || ws.ws_col == 0 {
        io::stdout().write(b"\x1b[999B\x1b[999C").unwrap();
        return get_cursor_position();
    }

    Ok((ws.ws_row as usize, ws.ws_col as usize))
}
