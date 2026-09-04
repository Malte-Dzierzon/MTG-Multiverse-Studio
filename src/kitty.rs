//! Kitty Graphics Protocol: transmit RGB → place at cursor cell → delete.

use std::io::Write;

const CHUNK: usize = 4096; // base64 bytes pro Nachricht

pub struct Kitty {
    pub enabled: bool,
    next_id: u32,
}

impl Default for Kitty {
    fn default() -> Self {
        Self {
            enabled: kitty_supported(),
            next_id: 100,
        }
    }
}

fn kitty_supported() -> bool {
    std::env::var("KITTY_WINDOW_ID").is_ok()
        || std::env::var("KITTY_PID").is_ok()
        || std::env::var("TERM")
            .map(|t| t.contains("kitty"))
            .unwrap_or(false)
}

impl Kitty {
    /// RGB-Daten hochladen, gibt Image-ID zurück.
    pub fn transmit(&mut self, rgb: &[u8], w: u32, h: u32) -> Option<u32> {
        if !self.enabled {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        write_stdout(&build_transmit_cmd(rgb, w, h, id));
        Some(id)
    }

    /// Bild an Zellposition platzieren. Nur Breite (c) angeben — kitty
    /// berechnet die Höhe aspect-getreu (beide Angaben würden stretchen).
    pub fn place(&mut self, id: u32, col: u16, row: u16, cols: u16) {
        if !self.enabled || cols == 0 {
            return;
        }
        // Cursor zur Zielzelle, dann Platzieren ohne Cursorbewegung.
        write_stdout(&format!(
            "\x1b[{};{}H\x1b_Ga=p,i={id},q=2,c={cols},C=1,z=-1\x1b\\",
            row + 1,
            col + 1
        ));
    }

    /// Alle Placements vom Screen entfernen.
    pub fn clear_all(&self) {
        if self.enabled {
            write_stdout("\x1b_Ga=d,d=A,q=2\x1b\\");
        }
    }

    /// Alle Placements eines Bildes entfernen (Daten bleiben).
    pub fn delete_placements(&self, id: u32) {
        if self.enabled {
            write_stdout(&format!("\x1b_Ga=d,d=p,i={id},q=2\x1b\\"));
        }
    }

    /// Bild-Daten endgültig freigeben.
    pub fn delete_data(&self, id: u32) {
        if self.enabled {
            write_stdout(&format!("\x1b_Ga=d,d=i,i={id},q=2\x1b\\"));
        }
    }
}

fn write_stdout(s: &str) {
    let mut out = std::io::stdout();
    out.write_all(s.as_bytes()).ok();
    out.flush().ok();
}

/// Chunked Transmit nach Kitty-Spec:
/// erster Chunk trägt alle Control-Keys, Folge-Chunks NUR m=1, letzter m=0.
fn build_transmit_cmd(rgb: &[u8], w: u32, h: u32, id: u32) -> String {
    let b64 = base64_encode(rgb);
    let mut out = String::with_capacity(b64.len() + 64);
    if b64.is_empty() {
        return format!("\x1b_Gf=24,q=2,s={w},h={h},i={id},m=0;\x1b\\");
    }
    let chunks: Vec<&str> = b64
        .as_bytes()
        .chunks(CHUNK)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect();
    let total = chunks.len();
    for (i, payload) in chunks.iter().enumerate() {
        let m = if i + 1 < total { 1 } else { 0 };
        out.push_str("\x1b_G");
        if i == 0 {
            // s/v = Bildmaße in Pixeln; h wäre Crop-Height (Spec-Tabelle)!
            out.push_str(&format!("f=24,q=2,s={w},v={h},i={id},m={m};{payload}"));
        } else {
            out.push_str(&format!("m={m};{payload}"));
        }
        out.push_str("\x1b\\");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_image_single_chunk_with_full_header() {
        let cmd = build_transmit_cmd(&[0u8; 12], 2, 2, 7);
        assert_eq!(cmd.matches("\x1b_G").count(), 1);
        assert!(cmd.starts_with("\x1b_Gf=24,q=2,s=2,v=2,i=7,m=0;"));
        assert!(cmd.ends_with("\x1b\\"));
    }

    #[test]
    fn large_image_chunks_follow_spec() {
        // 300x300 px → 270000 bytes RGB → ~360KB base64 → ~88 Chunks
        let rgb = vec![128u8; 300 * 300 * 3];
        let cmd = build_transmit_cmd(&rgb, 300, 300, 42);
        let frames: Vec<&str> = cmd.split("\x1b\\").filter(|s| s.contains("\x1b_G")).collect();
        let first_ok = frames[0].contains("f=24,q=2,s=300,v=300,i=42,m=1;");
        let second_ok = frames[1].starts_with("\x1b_Gm=1;");
        let last_ok = frames.last().unwrap().starts_with("\x1b_Gm=0;");
        assert!(first_ok && second_ok && last_ok, "{first_ok} {second_ok} {last_ok}");
        // Nur der erste Frame hat Control-Keys außer m
        for f in &frames[1..] {
            assert!(!f.contains("f=24") && !f.contains("v=300"), "{f:?}");
        }
        // Alle Chunks außer dem letzten sind Vielfache von 4
        for f in frames[..frames.len() - 1].iter() {
            let payload = f.split(';').nth(1).unwrap();
            assert_eq!(payload.len() % 4, 0);
        }
    }

    #[test]
    fn base64_known_vectors() {
        assert_eq!(base64_encode(b"Ma"), "TWE=");
        assert_eq!(base64_encode(b"Man"), "TWFu");
        assert_eq!(base64_encode(b""), "");
    }
}

/// Kleiner Base64-Encoder (Standard-Alphabet, ohne Padding-Anforderungen).
fn base64_encode(data: &[u8]) -> String {
    const TBL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let b = [
            c[0],
            *c.get(1).unwrap_or(&0),
            *c.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(TBL[(n >> 18) as usize & 63] as char);
        out.push(TBL[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { TBL[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { TBL[n as usize & 63] as char } else { '=' });
    }
    out
}
