use std::io;
use std::io::Read;

#[derive(PartialEq)]
pub enum EditorKey {
    Char(u8),
    ArrowUp,
    ArrowDown,
    ArrowRight,
    ArrowLeft,
    PageUp,
    PageDown,
    Home,
    End,
    Delete,
}

/*** input ***/

/// Try to read a single byte from stdin
///
/// If successful, returns `Some(<byte>)`,
/// if the timeout expires, returns `None`,
/// and panics if an error occurrs.
fn read_byte() -> Option<u8> {
    io::stdin().bytes().next().map(|r| r.expect("read error"))
}

/// Read and interpret a keypress from stdin
///
/// Returns `None` if the timeout expires
pub fn editor_read_key() -> Option<EditorKey> {
    use EditorKey::*;
    match read_byte() {
        Some(c) if c == b'\x1b' => Some({
            let mut seq = [None, None, None];
            seq[0] = read_byte();
            seq[1] = read_byte();
            if seq[0].is_some() && seq[1].is_some() {
                if seq[0] == Some(b'[') {
                    match seq[1].unwrap() {
                        b'0'..=b'9' => {
                            seq[2] = read_byte();
                            if seq[2] == Some(b'~') {
                                match seq[1].unwrap() {
                                    b'1' => Home,
                                    b'3' => Delete,
                                    b'4' => End,
                                    b'5' => PageUp,
                                    b'6' => PageDown,
                                    b'7' => Home,
                                    b'8' => End,
                                    _ => Char(b'\x1b'),
                                }
                            } else {
                                Char(b'\x1b')
                            }
                        },
                        b'A' => ArrowUp,
                        b'B' => ArrowDown,
                        b'C' => ArrowRight,
                        b'D' => ArrowLeft,
                        b'F' => End,
                        b'H' => Home,
                        _ => Char(b'\x1b'),
                    }
                } else if seq[0] == Some(b'O') {
                    match seq[1].unwrap() {
                        b'F' => End,
                        b'H' => Home,
                        _ => Char(b'\x1b'),
                    }
                } else {
                    Char(b'\x1b')
                }
            } else {
                Char(b'\x1b')
            }
        }),
        Some(c) => Some(Char(c)),
        None => None,
    }
}

