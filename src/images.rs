//! Bild-Loader-Thread: Scryfall-JPEG → RGB → Kitty. Nur Memory-LRU.

use std::collections::HashMap;
use std::io::Read;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;

pub struct Decoded {
    pub rgb: Vec<u8>,
    pub w: u32,
    pub h: u32,
}

pub enum Msg {
    Loaded(String, Decoded),
    Failed(String),
}

struct Entry {
    data: Decoded,
    kitty_id: Option<u32>,
}

pub struct Cache {
    map: HashMap<String, Entry>,
    order: Vec<String>,
    cap: usize,
}

pub struct Evicted {
    pub url: String,
    pub kitty_id: Option<u32>,
}

impl Cache {
    pub fn new(cap: usize) -> Self {
        Self { map: HashMap::new(), order: Vec::new(), cap }
    }

    pub fn get(&self, url: &str) -> Option<&Decoded> {
        self.map.get(url).map(|e| &e.data)
    }

    pub fn contains(&self, url: &str) -> bool {
        self.map.contains_key(url)
    }

    pub fn get_kitty_id(&self, url: &str) -> Option<u32> {
        self.map.get(url).and_then(|e| e.kitty_id)
    }

    pub fn insert(&mut self, url: String, d: Decoded) -> Option<Evicted> {
        if self.map.contains_key(&url) {
            return None;
        }
        let mut evicted = None;
        while self.order.len() >= self.cap {
            let old = self.order.remove(0);
            if let Some(e) = self.map.remove(&old) {
                evicted = Some(Evicted { url: old, kitty_id: e.kitty_id });
                break;
            }
        }
        self.order.push(url.clone());
        self.map.insert(url, Entry { data: d, kitty_id: None });
        evicted
    }

    pub fn set_kitty_id(&mut self, url: &str, id: u32) {
        if let Some(e) = self.map.get_mut(url) {
            e.kitty_id = Some(id);
        }
    }
}

/// Globaler Request-Sender (UI-Thread → Loader-Thread).
static REQ: Mutex<Option<Sender<String>>> = Mutex::new(None);

pub fn request(url: String) {
    if let Ok(guard) = REQ.lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(url);
        }
    }
}

pub fn spawn_loader() -> Receiver<Msg> {
    let (tx, rx_out) = std::sync::mpsc::channel::<Msg>();
    let (tx_req, rx_req) = std::sync::mpsc::channel::<String>();
    *REQ.lock().unwrap() = Some(tx_req);

    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("mtg-multiverse-studio/0.3")
            .build();
        for url in rx_req {
            match fetch_decode(&agent, &url) {
                Some(d) => {
                    let _ = tx.send(Msg::Loaded(url, d));
                }
                None => {
                    let _ = tx.send(Msg::Failed(url));
                }
            }
        }
    });
    rx_out
}

fn fetch_decode(agent: &ureq::Agent, url: &str) -> Option<Decoded> {
    let resp = agent.get(url).call().ok()?;
    let mut bytes = Vec::new();
    resp.into_reader().read_to_end(&mut bytes).ok()?;
    let mut dec = jpeg_decoder::Decoder::new(&bytes[..]);
    let pixels = dec.decode().ok()?;
    let info = dec.info()?;
    // kitty f=24 erwartet exakt 3 Bytes/Pixel (RGB)
    use jpeg_decoder::PixelFormat::*;
    let rgb = match info.pixel_format {
        RGB24 => pixels,
        L8 => pixels.iter().flat_map(|&l| [l, l, l]).collect(),
        _ => return None,
    };
    Some(Decoded {
        rgb,
        w: u32::from(info.width),
        h: u32::from(info.height),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::kitty;

    #[test]
    fn fetch_real_scryfall_jpeg() {
        // kleine Karte laden und dekodieren
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(10))
            .build();
        let url = "https://cards.scryfall.io/small/front/7/6/7673784e-db4b-43a1-8d55-1bb9fc1e284f.jpg?1782681979";
        match fetch_decode(&agent, url) {
            Some(d) => {
                assert_eq!(d.rgb.len(), (d.w * d.h * 3) as usize);
                assert!(d.w > 100);
            }
            None => panic!("fetch/decode failed for {url}"),
        }
    }
}
