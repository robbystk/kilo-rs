use std::io;
use std::io::Write;

use crate::{KILO_VERSION, EditorConfig};

/// Draw each row of the screen
///
/// Currently we have no lines, so this draws a tilde at the beginning of
/// each line, like vim, and prints a centered welcome message a third
/// of the way down the screen.
fn editor_draw_rows(config: &EditorConfig, buf: &mut String) {
    for i in 0..config.rows {
        if i < config.erows.len() {
            buf.push_str(&config.erows[i]);
        } else {
            if i == config.rows / 3 {
                let mut welcome = format!("Kilo Editor -- version {}", KILO_VERSION);

                // truncate to terminal width or less
                let mut welcome_len = welcome.len();
                while welcome_len > config.cols || !welcome.is_char_boundary(welcome_len) {
                    welcome_len -= 1;
                }
                welcome.truncate(welcome_len);

                let mut padding = (config.cols - welcome.len()) / 2;
                if padding > 0 {
                    buf.push('~');
                    padding -= 1;
                }
                for _ in 0..padding { buf.push(' '); }
                buf.push_str(&welcome);
            } else {
                buf.push('~');
            }
        }
        // clear remainder of row
        buf.push_str("\x1b[K");
        if i < config.rows - 1 {
            buf.push_str("\r\n");
        }
    }
}

/// Refresh the text on the screen
pub fn editor_refresh_screen(config: &EditorConfig) {
    let mut buf = String::from("");

    // hide cursor
    buf.push_str("\x1b[?25l");
    // move cursor to top left
    buf.push_str("\x1b[H");
    // draw a column of tildes like vim
    editor_draw_rows(config, &mut buf);
    // move cursor back to upper left
    buf.push_str(format!("\x1b[{};{}H", config.cy + 1, config.cx + 1).as_str());
    // show cursor
    buf.push_str("\x1b[?25h");

    io::stdout().write(&buf.as_bytes()).unwrap();
    // make sure things get written
    io::stdout().flush().unwrap()
}
