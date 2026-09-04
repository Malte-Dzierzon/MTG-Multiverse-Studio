//! Auto-Theme: Terminal-Farben per OSC 10/11/4 abfragen (Kitty & co.).

use ratatui::style::Color;

#[derive(Clone, Debug)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub dim: Color,
    pub wubrg: [Color; 5], // W U B R G
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark_fallback()
    }
}

impl Theme {
    fn dark_fallback() -> Self {
        Self {
            bg: Color::Rgb(24, 24, 28),
            fg: Color::Rgb(220, 220, 224),
            accent: Color::Rgb(122, 162, 247),
            dim: Color::Rgb(120, 120, 130),
            wubrg: [
                Color::Rgb(240, 240, 230),
                Color::Rgb(90, 160, 245),
                Color::Rgb(60, 60, 70),
                Color::Rgb(235, 90, 80),
                Color::Rgb(70, 190, 100),
            ],
        }
    }

    fn light_fallback() -> Self {
        Self {
            bg: Color::Rgb(245, 245, 242),
            fg: Color::Rgb(40, 40, 46),
            accent: Color::Rgb(40, 90, 200),
            dim: Color::Rgb(140, 140, 148),
            wubrg: [
                Color::Rgb(90, 90, 96),
                Color::Rgb(30, 110, 220),
                Color::Rgb(35, 35, 42),
                Color::Rgb(200, 55, 45),
                Color::Rgb(25, 140, 60),
            ],
        }
    }
}

fn luminance(c: Color) -> f64 {
    match c {
        Color::Rgb(r, g, b) => {
            (0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b)) / 255.0
        }
        _ => 0.0,
    }
}

fn parse_rgb(spec: &str) -> Option<Color> {
    if let Some(rest) = spec.strip_prefix("rgb:") {
        let conv = |s: &str| u16::from_str_radix(s.trim(), 16).ok().map(|v| (v >> 8) as u8);
        let mut it = rest.split('/');
        return Some(Color::Rgb(conv(it.next()?)?, conv(it.next()?)?, conv(it.next()?)?));
    }
    if let Some(rest) = spec.strip_prefix('#') {
        if rest.len() >= 6 {
            return Some(Color::Rgb(
                u8::from_str_radix(&rest[0..2], 16).ok()?,
                u8::from_str_radix(&rest[2..4], 16).ok()?,
                u8::from_str_radix(&rest[4..6], 16).ok()?,
            ));
        }
    }
    None
}

fn osc_value(raw: &str) -> &str {
    raw.split('\x07').next().unwrap_or("").split("\x1b\\").next().unwrap_or("")
}

/// Queries senden und Antworten von /dev/tty lesen.
pub fn probe() -> Theme {
    use std::io::{Read, Write};

    let mut buf = Vec::new();
    let mut tty = match std::fs::OpenOptions::new().read(true).write(true).open("/dev/tty") {
        Ok(f) => f,
        Err(_) => return Theme::default(),
    };

    // Non-blocking: wir wollen nicht ewig auf Antworten warten.
    #[cfg(unix)]
    unsafe {
        use std::os::fd::AsRawFd;
        let fd = tty.as_raw_fd();
        let flags = libc::fcntl(fd, libc::F_GETFL);
        if flags >= 0 {
            libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
    }

    // OSC 11 background, OSC 10 foreground, OSC 4 Palette 1-6
    if write!(
        tty,
        "\x1b]11;?\x1b\\\x1b]10;?\x1b\\\x1b]4;1;2;3;4;5;6;?\x1b\\"
    )
    .and_then(|_| tty.flush())
    .is_err()
    {
        return Theme::default();
    }

    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(300);
    let mut byte = [0u8; 1];
    while std::time::Instant::now() < deadline && buf.len() < 4096 {
        match tty.read(&mut byte) {
            Ok(1) => {
                buf.push(byte[0]);
                if buf.ends_with(b"\x1b\\") && buf.iter().filter(|&&b| b == b'\x1b').count() >= 5 {
                    break;
                }
            }
            Ok(_) => {} // EOF oder weitere Bytes
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::Interrupted =>
            {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(_) => break,
        }
    }
    let text = String::from_utf8_lossy(&buf);

    let find = |code: &str| -> Option<Color> {
        text.split("\x1b]")
            .filter_map(|part| part.strip_prefix(code))
            .find_map(|rest| rest.split(';').nth(1))
            .map(osc_value)
            .and_then(parse_rgb)
    };

    let bg = find("11").unwrap_or(Color::Rgb(24, 24, 28));
    let fg = find("10").unwrap_or(Color::Rgb(220, 220, 224));

    let mut pal: [Option<Color>; 7] = Default::default();
    for part in text.split("\x1b]") {
        let Some(rest) = part.strip_prefix("4;") else { continue };
        let segs: Vec<&str> = osc_value(rest).split(';').collect();
        for pair in segs.chunks(2) {
            if let (Ok(idx), Some(val)) = (pair[0].parse::<usize>(), pair.get(1)) {
                if idx < 7 {
                    pal[idx] = parse_rgb(val);
                }
            }
        }
    }

    let base = if luminance(bg) < 0.5 {
        Theme::dark_fallback()
    } else {
        Theme::light_fallback()
    };
    let pick = |i: usize, fb: Color| pal[i].unwrap_or(fb);

    Theme {
        bg,
        fg,
        accent: pick(4, base.accent),
        dim: mix(fg, bg, 0.45),
        wubrg: [
            mix(fg, bg, 0.05),          // W
            pick(4, base.wubrg[1]),     // U
            mix(pick(0, base.fg), bg, 0.75), // B
            pick(1, base.wubrg[3]),     // R
            pick(2, base.wubrg[4]),     // G
        ],
    }
}

fn mix(a: Color, b: Color, t: f64) -> Color {
    let get = |c: Color| match c {
        Color::Rgb(r, g, bl) => (f64::from(r), f64::from(g), f64::from(bl)),
        _ => (128.0, 128.0, 128.0),
    };
    let (ar, ag, ab) = get(a);
    let (br, bg_, bb) = get(b);
    Color::Rgb(
        (ar + (br - ar) * t) as u8,
        (ag + (bg_ - ag) * t) as u8,
        (ab + (bb - ab) * t) as u8,
    )
}
