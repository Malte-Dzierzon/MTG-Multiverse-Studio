mod agent;
mod app;
mod db;
mod images;
mod kitty;
mod theme;
mod ui;

use app::App;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;
use std::collections::HashMap;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut db_path = "assets/mtg_multiverse_studio.db".to_string();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--db" => db_path = args.next().unwrap_or(db_path),
            "-h" | "--help" => {
                println!("mtg-tui {} — MTG Multiverse Studio", env!("CARGO_PKG_VERSION"));
                println!("\nUsage: mtg-tui [--db /path/to/database.db]");
                return Ok(());
            }
            _ => {}
        }
    }
    if !std::path::Path::new(&db_path).exists() {
        eprintln!("✗ db not found: {db_path}");
        std::process::exit(1);
    }
    let database = db::Db::open(&db_path).expect("open db");

    // Raw mode zuerst (kein Echo), dann Theme/App — alles VOR dem
    // Alternate Screen, damit Debug-Ausgaben die Oberfläche nicht vermüllen.
    crossterm::terminal::enable_raw_mode()?;
    let thm = theme::probe();

    let mut kitty_gfx = kitty::Kitty::default();
    let img_rx = images::spawn_loader();
    let agent_cfg = agent::AgentConfig::from_env();
    let agent_rx = agent::spawn_worker(agent_cfg.clone());
    let scry_rx = agent::spawn_scryfall_worker();
    let mut app = App::new(&database, thm, agent_cfg);

    if std::env::var("MTG_TUI_DEBUG").is_ok() {
        eprintln!(
            "[debug] kitty.enabled={} cover_url={:?}",
            kitty_gfx.enabled, app.cover_url
        );
    }

    let mut terminal = ratatui::init();
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::event::EnableBracketedPaste,
        crossterm::event::EnableMouseCapture
    );
    kitty_gfx.clear_all();

    let mut cache = images::Cache::new(64);
    let mut placements: HashMap<u8, Placed> = HashMap::new(); // slot 0=cover 1=preview

    let result = loop_app(
        &mut terminal,
        &mut app,
        &mut kitty_gfx,
        &mut cache,
        &img_rx,
        &agent_rx,
        &scry_rx,
        &mut placements,
    );

    kitty_gfx.clear_all();
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::event::DisableBracketedPaste,
        crossterm::event::DisableMouseCapture
    );
    ratatui::restore();
    result
}

struct Placed {
    url: String,
    rect: Rect,
}

#[allow(clippy::too_many_arguments)]
fn loop_app(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App<'_>,
    kitty_gfx: &mut kitty::Kitty,
    cache: &mut images::Cache,
    img_rx: &std::sync::mpsc::Receiver<images::Msg>,
    agent_rx: &std::sync::mpsc::Receiver<agent::Out>,
    scry_rx: &std::sync::mpsc::Receiver<(String, Vec<agent::ScryfallHit>)>,
    placements: &mut HashMap<u8, Placed>,
) -> std::io::Result<()> {
    let mut quit = false;
    let mut last_frame = std::time::Instant::now();
    while !quit {
        // ── draw + Bild-Placements ──
        let mut regions = ui::UiRegions {
            cover: Rect::default(),
            preview: Rect::default(),
            panel: Rect::default(),
            main: Rect::default(),
        };
        terminal.draw(|f| {
            app.cover_loading = app
                .cover_url
                .as_deref()
                .map(|u| !cache.contains(u) && !app.failed.contains(u))
                .unwrap_or(false);
            regions = ui::draw(f, app);
        })?;

        place_images(kitty_gfx, cache, app, &regions, placements);
        drain_images(img_rx, cache, kitty_gfx, app, placements);
        while let Ok(out) = agent_rx.try_recv() {
            app.on_agent_reply(out);
            last_frame = std::time::Instant::now(); // redraw erzwingen
        }
        while let Ok((q, hits)) = scry_rx.try_recv() {
            app.on_scryfall_result(q, hits);
            last_frame = std::time::Instant::now();
        }

        // ── events ──
        let animating = app.tick(dt_secs(last_frame));
        last_frame = std::time::Instant::now();
        let timeout = if animating || app.loading {
            std::time::Duration::from_millis(16)
        } else {
            std::time::Duration::from_millis(150)
        };

        if crossterm::event::poll(timeout)? {
            match crossterm::event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Char(c)
                            if app.input_active()
                                && !key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            app.char_input(c);
                        }
                        code => {
                            quit = app.key(code, key.modifiers);
                        }
                    }
                }
                Event::Paste(s) => app.paste(&s),
                Event::Mouse(mouse) if mouse.kind == crossterm::event::MouseEventKind::Down(
                    crossterm::event::MouseButton::Left) =>
                {
                    if regions.panel.width > 0 && mouse.column >= regions.panel.x {
                        app.focus = crate::app::Focus::Agent;
                    } else if mouse.column < regions.main.right()
                        && mouse.row < regions.main.bottom()
                    {
                        app.focus = crate::app::Focus::Main;
                    }
                }
                Event::Resize(_, _) => {
                    kitty_gfx.clear_all();
                    placements.clear();
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn dt_secs(last: std::time::Instant) -> f32 {
    last.elapsed().as_secs_f32().clamp(0.001, 0.1)
}

fn place_images(
    kitty_gfx: &mut kitty::Kitty,
    cache: &mut images::Cache,
    app: &mut App<'_>,
    regions: &ui::UiRegions,
    placements: &mut HashMap<u8, Placed>,
) {
    let slots: [(u8, SlotRef); 2] = [
        (0, SlotRef { url: app.cover_url.as_deref(), rect: regions.cover }),
        (1, SlotRef { url: app.preview_url.as_deref(), rect: regions.preview }),
    ];

    for (slot_id, slot) in slots {
        let Some(url) = slot.url else {
            // Kein Bild gewünscht → stehengebliebenes Placement entfernen
            if let Some(prev) = placements.remove(&slot_id) {
                if let Some(pid) = cache.get_kitty_id(&prev.url) {
                    kitty_gfx.delete_placements(pid);
                }
            }
            continue;
        };
        let rect_valid = slot.rect.width > 3 && slot.rect.height > 3;
        if !rect_valid {
            if let Some(prev) = placements.remove(&slot_id) {
                if let Some(pid) = cache.get_kitty_id(&prev.url) {
                    kitty_gfx.delete_placements(pid);
                }
            }
            continue;
        }

        if !cache.contains(url) {
            // Evicted/noch nicht geladen: altes Placement dieser URL ist tot
            if let Some(prev) = placements.get(&slot_id) {
                if prev.url == url {
                    placements.remove(&slot_id);
                }
            }
            if !app.requested.contains(url) && !app.failed.contains(url) {
                app.requested.insert(url.to_string());
                images::request(url.to_string());
            }
            continue;
        }

        let same = placements
            .get(&slot_id)
            .map(|p| p.url == url && p.rect == slot.rect)
            .unwrap_or(false);
        if same {
            continue;
        }

        // Alte Placements dieser Slot-Position entfernen
        if let Some(prev) = placements.remove(&slot_id) {
            if let Some(pid) = cache.get_kitty_id(&prev.url) {
                kitty_gfx.delete_placements(pid);
            }
        }

        let Some(data) = cache.get(url) else { continue };
        let id = match cache.get_kitty_id(url) {
            Some(id) => id,
            None => {
                if let Some(new_id) = kitty_gfx.transmit(&data.rgb, data.w, data.h) {
                    cache.set_kitty_id(url, new_id);
                    new_id
                } else {
                    continue;
                }
            }
        };
        kitty_gfx.place(id, slot.rect.x, slot.rect.y, slot.rect.width);
        placements.insert(slot_id, Placed { url: url.to_string(), rect: slot.rect });
    }
}

struct SlotRef<'a> {
    url: Option<&'a str>,
    rect: Rect,
}

fn drain_images(
    rx: &std::sync::mpsc::Receiver<images::Msg>,
    cache: &mut images::Cache,
    kitty_gfx: &mut kitty::Kitty,
    app: &mut App<'_>,
    placements: &mut HashMap<u8, Placed>,
) {
    while let Ok(msg) = rx.try_recv() {
        match msg {
            images::Msg::Loaded(url, d) => {
                if let Some(ev) = cache.insert(url.clone(), d) {
                    // Evictete URL: Grafik löschen UND alle Referenzen aufräumen
                    if let Some(id) = ev.kitty_id {
                        kitty_gfx.delete_data(id);
                    }
                    placements.retain(|_, p| p.url != ev.url);
                    app.requested.remove(&ev.url);
                }
            }
            images::Msg::Failed(url) => {
                app.failed.insert(url.clone());
                app.requested.remove(&url);
            }
        }
    }
}
