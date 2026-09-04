//! UI: Layouts, Widgets, Overlays. Clean minimalist design with Nerd Font icons.

use crate::app::{App, Focus, Input, Mode, SortMode};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Bar, BarChart, Block, BorderType, Borders, List, ListItem, Paragraph,
};
use ratatui::Frame;

/// Nerd Font Icons (Material Design Range, U+F0001–U+F1AAF)
mod icon {
    pub const DECK: &str = "\u{f0bc6}";         // nf-md-cards
    pub const CARD: &str = "\u{f0baa}";         // nf-md-card_text_outline
    pub const SEARCH: &str = "\u{f0349}";       // nf-md-magnify
    pub const PLUS: &str = "\u{f0415}";         // nf-md-plus
    pub const PIN: &str = "\u{f099d}";          // nf-md-pin
    pub const CHAT: &str = "\u{f0b18}";         // nf-md-chat
    pub const BOT: &str = "\u{f06a3}";          // nf-md-robot
    pub const PRICE: &str = "\u{f0167}";        // nf-md-cash
    pub const STAR: &str = "\u{f04d3}";         // nf-md-star
    pub const STAR_OUTLINE: &str = "\u{f04da}"; // nf-md-star_outline
    pub const SORT: &str = "\u{f044b}";         // nf-md-sort_variant
    pub const FILTER: &str = "\u{f033d}";       // nf-md-filter_variant
    pub const CHEVRON: &str = "\u{f0142}";      // nf-md-chevron_right
    pub const CHECK: &str = "\u{f012c}";        // nf-md-check
    pub const CLOSE: &str = "\u{f0156}";        // nf-md-close
    pub const INFO: &str = "\u{f02fc}";         // nf-md_information
    pub const KEY: &str = "\u{f0328}";          // nf-md-key
    pub const NOTE: &str = "\u{f0818}";         // nf-md-note_text
    pub const SCROLL: &str = "\u{f09ee}";       // nf-md-file_document_outline
    pub const SPARKLES: &str = "\u{f0de5}";     // nf-md-sparkles
    pub const HAND: &str = "\u{f02f7}";         // nf-md-hand_pointing_right
    pub const PENCIL: &str = "\u{f03eb}";       // nf-md-pencil
    pub const ARROW_UP: &str = "\u{f05dc}";     // nf-md-arrow_up_thick
    pub const ARROW_DOWN: &str = "\u{f05d5}";   // nf-md-arrow_down_thick
}

/// Regionen nach dem Draw: Bild-Slots für Kitty, Panel-Rect für Maus-Fokus.
pub struct UiRegions {
    pub cover: Rect,
    pub preview: Rect,
    pub panel: Rect,
    pub main: Rect,
}

pub fn draw(f: &mut Frame<'_>, app: &App<'_>) -> UiRegions {
    let full = f.area();
    if full.width == 0 || full.height == 0 {
        return UiRegions {
            cover: Rect::default(),
            preview: Rect::default(),
            panel: Rect::default(),
            main: Rect::default(),
        };
    }
    let [_content, cmdline_area, status] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(full);
    cmdline(f, app, cmdline_area);
    status_bar(f, app, status);

    // Main | Agent-Panel (persistent ab ~100 cols)
    let panel_w = if full.width >= 100 {
        (full.width * 3 / 10).clamp(36, 48)
    } else {
        0
    };
    if panel_w > 0 {
        let [main, panel] =
            Layout::horizontal([Constraint::Min(20), Constraint::Length(panel_w)])
                .areas(full);
        let mut regions = render_main(f, app, main);
        agent_panel(f, app, panel);
        regions.panel = panel;
        regions
    } else {
        let mut r = render_main(f, app, full);
        r.panel = Rect::default();
        r
    }
}

fn status_bar(f: &mut Frame<'_>, app: &App<'_>, area: Rect) {
    let th = &app.theme;
    let hints: Vec<Span<'_>> = match (app.focus, app.mode, app.input) {
        (Focus::Agent, _, _) => vec![
            Span::styled(icon::HAND, Style::new().fg(th.dim)),
            Span::styled(" enter send  ", Style::new().fg(th.dim)),
            Span::styled(icon::DECK, Style::new().fg(th.dim)),
            Span::styled("/deck pick  ", Style::new().fg(th.dim)),
            Span::styled(icon::KEY, Style::new().fg(th.dim)),
            Span::styled("tab back", Style::new().fg(th.dim)),
        ],
        (_, _, Input::Import) => vec![
            Span::styled(icon::SCROLL, Style::new().fg(th.dim)),
            Span::styled(" paste list  ", Style::new().fg(th.dim)),
            Span::styled(icon::CHECK, Style::new().fg(th.dim)),
            Span::styled("ctrl+s import  ", Style::new().fg(th.dim)),
            Span::styled(icon::CLOSE, Style::new().fg(th.dim)),
            Span::styled(" esc", Style::new().fg(th.dim)),
        ],
        (_, _, Input::NewDeck) => vec![
            Span::styled(icon::PENCIL, Style::new().fg(th.dim)),
            Span::styled(" name  ", Style::new().fg(th.dim)),
            Span::styled(icon::CHEVRON, Style::new().fg(th.dim)),
            Span::styled(" enter  ", Style::new().fg(th.dim)),
            Span::styled(icon::CHEVRON, Style::new().fg(th.dim)),
            Span::styled(" then format", Style::new().fg(th.dim)),
        ],
        (_, Mode::Home, Input::Query) => vec![
            Span::styled(icon::FILTER, Style::new().fg(th.dim)),
            Span::styled(" filter decks  ", Style::new().fg(th.dim)),
            Span::styled(icon::CLOSE, Style::new().fg(th.dim)),
            Span::styled(" esc clear", Style::new().fg(th.dim)),
        ],
        (_, Mode::Home, _) => vec![
            Span::styled(icon::ARROW_UP, Style::new().fg(th.dim)),
            Span::styled(icon::ARROW_DOWN, Style::new().fg(th.dim)),
            Span::styled(" deck  ", Style::new().fg(th.dim)),
            Span::styled(icon::CHEVRON, Style::new().fg(th.dim)),
            Span::styled(" open  ", Style::new().fg(th.dim)),
            Span::styled(icon::PLUS, Style::new().fg(th.dim)),
            Span::styled(" new  ", Style::new().fg(th.dim)),
            Span::styled(icon::SCROLL, Style::new().fg(th.dim)),
            Span::styled(" import  ", Style::new().fg(th.dim)),
            Span::styled(icon::FILTER, Style::new().fg(th.dim)),
            Span::styled(" filter  ", Style::new().fg(th.dim)),
            Span::styled(icon::KEY, Style::new().fg(th.dim)),
            Span::styled(" q quit", Style::new().fg(th.dim)),
        ],
        (_, Mode::DeckView, _) => vec![
            Span::styled(icon::ARROW_UP, Style::new().fg(th.dim)),
            Span::styled(icon::ARROW_DOWN, Style::new().fg(th.dim)),
            Span::styled(" card  ", Style::new().fg(th.dim)),
            Span::styled(icon::PLUS, Style::new().fg(th.dim)),
            Span::styled(" add  ", Style::new().fg(th.dim)),
            Span::styled(icon::CHEVRON, Style::new().fg(th.dim)),
            Span::styled(" details  ", Style::new().fg(th.dim)),
            Span::styled(icon::STAR, Style::new().fg(th.dim)),
            Span::styled(" stats  ", Style::new().fg(th.dim)),
            Span::styled(icon::SORT, Style::new().fg(th.dim)),
            Span::styled(" sort  ", Style::new().fg(th.dim)),
            Span::styled(icon::PIN, Style::new().fg(th.dim)),
            Span::styled(" pin  ", Style::new().fg(th.dim)),
            Span::styled(icon::CLOSE, Style::new().fg(th.dim)),
            Span::styled(" esc", Style::new().fg(th.dim)),
        ],
        (_, Mode::CardView, _) => vec![
            Span::styled(icon::PLUS, Style::new().fg(th.dim)),
            Span::styled(" add  ", Style::new().fg(th.dim)),
            Span::styled(icon::PIN, Style::new().fg(th.dim)),
            Span::styled(" pin  ", Style::new().fg(th.dim)),
            Span::styled(icon::CLOSE, Style::new().fg(th.dim)),
            Span::styled(" esc back", Style::new().fg(th.dim)),
        ],
        (_, Mode::Search, _) => vec![],
    };
    let mut line = hints;
    if !app.status.is_empty() {
        line.push(Span::styled("  ", Style::new()));
        line.push(Span::styled(app.status.clone(), Style::new().fg(th.accent)));
    }
    f.render_widget(Paragraph::new(Line::from(line)), area);
}

fn cmdline(f: &mut Frame<'_>, app: &App<'_>, area: Rect) {
    let th = &app.theme;
    let text = match app.focus {
        Focus::Agent => app.agent_input.clone(),
        Focus::Main => app.query.lines().last().unwrap_or("").to_string(),
    };
    let prompt = match (app.focus, app.input) {
        (Focus::Agent, _) => Span::styled(
            format!(" {} chat {} ", icon::BOT, icon::CHEVRON),
            Style::new().fg(th.accent),
        ),
        (_, Input::None | Input::Token) => Span::styled(
            format!(" {} ", icon::SEARCH),
            Style::new().fg(th.dim),
        ),
        (_, Input::Query) => Span::styled(
            format!(" {} filter ", icon::FILTER),
            Style::new().fg(th.accent),
        ),
        (_, Input::NewDeck) => Span::styled(
            format!(" {} new deck {} ", icon::PLUS, icon::CHEVRON),
            Style::new().fg(th.accent),
        ),
        (_, Input::Import) => Span::styled(
            format!(" {} paste {} ", icon::SCROLL, icon::CHEVRON),
            Style::new().fg(th.accent),
        ),
    };
    let cursor = Span::styled(
        "_",
        Style::new().fg(if app.input_active() || app.focus == Focus::Agent {
            th.accent
        } else {
            th.dim
        }),
    );
    f.render_widget(
        Line::from(vec![prompt, Span::styled(text, Style::new().fg(th.fg)), cursor]),
        area,
    );
}

impl App<'_> {
    pub fn input_active(&self) -> bool {
        self.input != Input::None || self.focus == Focus::Agent
    }
}

// ── Main ─────────────────────────────────────────────────────────

fn render_main(f: &mut Frame<'_>, app: &App<'_>, area: Rect) -> UiRegions {
    match app.mode {
        Mode::Home => dashboard(f, app, area),
        Mode::DeckView | Mode::CardView => detail(f, app, area),
        Mode::Search => search(f, app, area),
    }
}

/// Startseite: Decks als saubere Zeilen, Stats rechts.
fn dashboard(f: &mut Frame<'_>, app: &App<'_>, area: Rect) -> UiRegions {
    let th = &app.theme;
    let vis = app.visible_decks();

    let [list_area, stats_area] = if area.width > 60 {
        Layout::horizontal([Constraint::Percentage(62), Constraint::Fill(1)]).areas(area)
    } else {
        [area, Rect::default()]
    };

    // Section header
    if list_area.height > 0 {
        f.render_widget(
            Line::from(vec![
                Span::styled(format!(" {} decks", icon::DECK), Style::new().fg(th.accent).add_modifier(Modifier::BOLD)),
                if !app.query.is_empty() {
                    Span::styled("  (filtered)", Style::new().fg(th.dim))
                } else {
                    Span::raw("")
                },
            ]),
            Rect { x: list_area.x, y: list_area.y, width: list_area.width, height: 1 },
        );
    }

    // Deck list
    let list_inner = Rect { y: list_area.y + 1, height: list_area.height.saturating_sub(1), ..list_area };
    if list_inner.width > 8 && list_inner.height >= 1 && !vis.is_empty() {
        let visible = list_inner.height as usize;
        let max_start = vis.len().saturating_sub(visible);
        let start = app.deck_sel.saturating_sub(visible.saturating_sub(1)).min(max_start);

        for row in 0..visible.min(vis.len().saturating_sub(start)) {
            let deck_idx = vis[start + row];
            let d = &app.decks[deck_idx];
            let value = app.deck_values.get(deck_idx).copied().unwrap_or(0.0);
            let selected = row == app.deck_sel.saturating_sub(start) && app.deck_sel < vis.len();
            let y = list_inner.y + row as u16;

            let marker = if selected { icon::CHEVRON } else { " " };
            let marker_style = if selected {
                Style::new().fg(th.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(th.dim)
            };

            // Format badge color
            let fmt_color = match d.fmt.as_str() {
                "commander" => th.accent,
                "modern" | "pioneer" | "standard" => Color::Rgb(90, 190, 120),
                _ => th.dim,
            };

            let line = Line::from(vec![
                Span::styled(format!(" {} ", marker), marker_style),
                Span::styled(icon::DECK, Style::new().fg(if selected { th.accent } else { th.dim })),
                Span::styled(" ", Style::new()),
                Span::styled(
                    truncate(&d.name, 24),
                    if selected {
                        Style::new().fg(th.fg).add_modifier(Modifier::BOLD)
                    } else {
                        Style::new().fg(th.fg)
                    },
                ),
                Span::styled("  ", Style::new()),
                Span::styled(
                    format!("{}{}", icon::CARD, d.count),
                    Style::new().fg(if selected { th.accent } else { th.dim }),
                ),
                Span::styled("  ", Style::new()),
                Span::styled(
                    format!("{}{value:.2}", icon::PRICE),
                    Style::new().fg(th.accent),
                ),
                Span::styled("  ", Style::new()),
                Span::styled(&d.fmt, Style::new().fg(fmt_color)),
            ]);
            f.render_widget(line, Rect { x: list_inner.x, y, width: list_inner.width, height: 1 });
        }
    } else if vis.is_empty() && list_inner.height > 0 {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!(" {} no decks match", icon::INFO), Style::new().fg(th.dim)),
            ])),
            list_inner,
        );
    }

    // Stats sidebar
    if stats_area.width > 4 {
        let total_q: i64 = app.decks.iter().map(|d| d.count).sum();
        let total_eur: f64 = app.deck_values.iter().sum();
        let deck_count = app.decks.len();
        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(format!(" {} decks ", icon::DECK), Style::new().fg(th.dim)),
                Span::styled(deck_count.to_string(), Style::new().fg(th.fg).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!(" {} cards ", icon::DECK), Style::new().fg(th.dim)),
                Span::styled(total_q.to_string(), Style::new().fg(th.fg).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!(" {} value ", icon::PRICE), Style::new().fg(th.dim)),
                Span::styled(format!("€{total_eur:.2}"), Style::new().fg(th.accent).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(Span::styled(" ── keys ──", Style::new().fg(th.dim))),
            Line::from(vec![
                Span::styled(format!("  {} enter ", icon::CHEVRON), Style::new().fg(th.dim)),
                Span::styled("open deck", Style::new().fg(th.dim)),
            ]),
            Line::from(vec![
                Span::styled(format!("  {} /     ", icon::FILTER), Style::new().fg(th.dim)),
                Span::styled("filter", Style::new().fg(th.dim)),
            ]),
            Line::from(vec![
                Span::styled(format!("  {} n     ", icon::PLUS), Style::new().fg(th.dim)),
                Span::styled("new deck", Style::new().fg(th.dim)),
            ]),
            Line::from(vec![
                Span::styled(format!("  {} i     ", icon::SCROLL), Style::new().fg(th.dim)),
                Span::styled("import", Style::new().fg(th.dim)),
            ]),
            Line::from(vec![
                Span::styled(format!("  {} tab   ", icon::CHAT), Style::new().fg(th.dim)),
                Span::styled("chat", Style::new().fg(th.dim)),
            ]),
            Line::from(vec![
                Span::styled(format!("  {} q     ", icon::CLOSE), Style::new().fg(th.dim)),
                Span::styled("quit", Style::new().fg(th.dim)),
            ]),
        ];
        f.render_widget(Paragraph::new(lines), stats_area);
    }

    UiRegions {
        cover: Rect::default(),
        preview: Rect::default(),
        panel: Rect::default(),
        main: area,
    }
}

/// Deck-/Karten-Detail: Liste links, Bild rechts.
fn detail(f: &mut Frame<'_>, app: &App<'_>, area: Rect) -> UiRegions {
    let img_w = ((area.width.min(120) as f32) * 0.26) as u16;
    let [table, preview] =
        Layout::horizontal([Constraint::Min(10), Constraint::Length(img_w)]).areas(area);

    match app.mode {
        Mode::DeckView => deck_table(f, app, table),
        Mode::CardView => card_details(f, app, table),
        _ => {}
    }
    if app.stats_open && app.mode == Mode::DeckView {
        stats_overlay(f, app, area);
    }

    UiRegions {
        cover: Rect::default(),
        preview,
        panel: Rect::default(),
        main: area,
    }
}

fn deck_table(f: &mut Frame<'_>, app: &App<'_>, area: Rect) {
    let th = &app.theme;
    let sort_label = match app.sort {
        SortMode::Alpha => "α",
        SortMode::Category => "cat",
    };

    // Section header with deck name
    let deck_name = app.active_deck.as_ref().map(|d| d.name.as_str()).unwrap_or("");
    f.render_widget(
        Line::from(vec![
            Span::styled(format!(" {} ", icon::DECK), Style::new().fg(th.accent).add_modifier(Modifier::BOLD)),
            Span::styled(truncate(deck_name, 30), Style::new().fg(th.fg).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("  {} {} sorted:{}", icon::SORT, icon::ARROW_DOWN, sort_label),
                Style::new().fg(th.dim),
            ),
        ]),
        Rect { x: area.x, y: area.y, width: area.width, height: 1 },
    );

    // Column header
    let header = Rect { y: area.y + 1, height: 1, ..area };
    f.render_widget(
        Line::from(vec![
            Span::styled("    ", Style::new()),
            Span::styled("NAME", Style::new().fg(th.dim).add_modifier(Modifier::BOLD)),
            Span::styled("        ", Style::new()),
            Span::styled("COST", Style::new().fg(th.dim).add_modifier(Modifier::BOLD)),
            Span::styled("  ", Style::new()),
            Span::styled("TYPE", Style::new().fg(th.dim).add_modifier(Modifier::BOLD)),
        ]),
        header,
    );

    // Card rows
    let body = Rect { y: area.y + 2, height: area.height.saturating_sub(3), ..area };
    let mut rows: Vec<Line<'_>> = Vec::new();
    let mut line_index: usize = 0;
    let mut sel_line: Option<usize> = None;
    let mut last_cat = String::new();

    for (i, e) in app.entries.iter().enumerate() {
        if app.sort == SortMode::Category && e.category != last_cat {
            last_cat = e.category.clone();
            let cat_icon = match last_cat.as_str() {
                "Creatures" => icon::CARD,
                "Instants" | "Sorceries" => icon::SPARKLES,
                "Enchantments" | "Artifacts" => icon::STAR,
                "Lands" => icon::DECK,
                _ => "·",
            };
            let cat_name = last_cat.clone();
            rows.push(Line::from(vec![
                Span::styled(format!(" {cat_icon} "), Style::new().fg(th.accent)),
                Span::styled(cat_name, Style::new().fg(th.accent).add_modifier(Modifier::BOLD)),
            ]));
            line_index += 1;
        }
        if i == app.entry_sel {
            sel_line = Some(line_index);
        }
        rows.push(Line::from(vec![
            Span::styled(format!("  {:>2} ", e.qty), Style::new().fg(th.dim)),
            Span::styled(
                format!("{:<28}", truncate(&e.card.name, 27)),
                Style::new().fg(th.fg),
            ),
            Span::styled(
                format!("{:<11}", e.card.mana_cost),
                Style::new().fg(th.dim),
            ),
            Span::styled(
                truncate(&e.card.type_line, 20),
                Style::new().fg(th.dim),
            ),
        ]));
        line_index += 1;
    }

    let items: Vec<ListItem<'_>> = rows.iter().cloned().map(ListItem::new).collect();
    f.render_widget(List::new(items), body);

    // Floating selection highlight
    if let Some(sl) = sel_line {
        let anim = app.entry_anim.round() as usize;
        let headers_before: usize = if app.sort == SortMode::Category {
            app.entries
                .iter()
                .take(anim.max(1).min(app.entries.len()))
                .map(|e| e.category.clone())
                .collect::<Vec<_>>()
                .windows(2)
                .filter(|w| w[0] != w[1])
                .count()
                + 1
        } else {
            0
        };
        let y = body.y + (sl - if app.sort == SortMode::Category { headers_before.min(sl) } else { 0 }) as u16;
        let hl = Rect { x: body.x, y, width: body.width, height: 1 };
        if let Some(sel) = app.entries.get(anim.min(app.entries.len().saturating_sub(1))) {
            let line = Line::from(vec![
                Span::styled(format!(" {} ", icon::CHEVRON), Style::new().fg(th.accent).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:>2} ", sel.qty),
                    Style::new().fg(th.accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    truncate(&sel.card.name, 27),
                    Style::new().fg(th.fg).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{:<11}", sel.card.mana_cost), Style::new().fg(th.dim)),
                Span::styled(truncate(&sel.card.type_line, 20), Style::new().fg(th.dim)),
            ]);
            f.render_widget(ClearWrap, hl);
            f.render_widget(line, hl);
        }
    }

    // Footer
    let total_q: i64 = app.entries.iter().map(|e| e.qty).sum();
    let total_eur: f64 = app.entries.iter().map(|e| e.eur()).sum();
    let sum = Rect { y: area.bottom().saturating_sub(1), height: 1, ..area };
    f.render_widget(
        Line::from(vec![
            Span::styled(format!(" {} {} cards", icon::DECK, total_q), Style::new().fg(th.fg)),
            Span::styled("  ", Style::new()),
            Span::styled(format!("{} €{total_eur:.2}", icon::PRICE), Style::new().fg(th.accent)),
        ]),
        sum,
    );
}

fn card_details(f: &mut Frame<'_>, app: &App<'_>, area: Rect) {
    let th = &app.theme;
    let Some(c) = &app.current_card else { return };
    let eur = c.prices.get("eur").and_then(|v| v.as_str()).unwrap_or("—");
    let usd = c.prices.get("usd").and_then(|v| v.as_str()).unwrap_or("—");
    let pinned = app.store.pinned.contains(&c.name);

    // Card name header
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", if pinned { icon::PIN } else { icon::CARD }),
                Style::new().fg(if pinned { Color::Rgb(255, 180, 60) } else { th.dim }),
            ),
            Span::styled(&c.name, Style::new().fg(th.fg).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("   ", Style::new()),
            Span::styled(&c.mana_cost, Style::new().fg(th.dim)),
            Span::styled("  ", Style::new()),
            Span::styled(&c.type_line, Style::new().fg(th.dim)),
        ]),
        Line::default(),
        // Price line
        Line::from(vec![
            Span::styled("   ", Style::new()),
            Span::styled(icon::PRICE, Style::new().fg(th.accent)),
            Span::styled(format!(" €{eur}"), Style::new().fg(th.fg)),
            Span::styled("   $", Style::new().fg(th.dim)),
            Span::styled(usd, Style::new().fg(th.dim)),
        ]),
        // Rarity + set + artist
        Line::from(vec![
            Span::styled("   ", Style::new()),
            Span::styled(rarity_icon(&c.rarity), Style::new().fg(rarity_color(&c.rarity))),
            Span::styled(" ", Style::new()),
            Span::styled(&c.rarity, Style::new().fg(th.dim)),
            Span::styled(" · ", Style::new().fg(th.dim)),
            Span::styled(&c.set_code, Style::new().fg(th.dim)),
            Span::styled(" · ", Style::new().fg(th.dim)),
            Span::styled(&c.artist, Style::new().fg(th.dim)),
        ]),
        Line::default(),
    ];

    // Oracle text
    for l in wrap(&c.oracle_text, area.width.saturating_sub(4) as usize).into_iter().take(12) {
        lines.push(Line::from(vec![
            Span::styled("   ", Style::new()),
            Span::styled(l, Style::new().fg(th.fg)),
        ]));
    }
    lines.push(Line::default());

    // Legality badges
    let formats = ["standard", "modern", "pioneer", "legacy", "commander"];
    let legal: Vec<String> = formats
        .iter()
        .filter(|k| c.legalities.contains_key(**k))
        .map(|k| k.to_string())
        .collect();
    if !legal.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("   ", Style::new()),
            Span::styled(icon::CHECK, Style::new().fg(Color::Rgb(90, 190, 120))),
            Span::styled(" ", Style::new()),
            Span::styled(legal.join("  "), Style::new().fg(Color::Rgb(90, 190, 120))),
        ]));
    }

    // P/T or Loyalty
    if let Some(power) = c.power.as_deref() {
        if let Some(toughness) = c.toughness.as_deref() {
            lines.push(Line::from(vec![
                Span::styled("   ", Style::new()),
                Span::styled(icon::STAR, Style::new().fg(th.dim)),
                Span::styled(format!(" {}/{}", power, toughness), Style::new().fg(th.fg)),
            ]));
        }
    }
    if let Some(loyalty) = c.loyalty.as_deref() {
        lines.push(Line::from(vec![
            Span::styled("   ", Style::new()),
            Span::styled(icon::STAR, Style::new().fg(th.dim)),
            Span::styled(format!(" loyalty {loyalty}"), Style::new().fg(th.fg)),
        ]));
    }

    // Flavor text
    if let Some(flavor) = c.flavor_text.as_deref() {
        if !flavor.is_empty() {
            lines.push(Line::default());
            for l in wrap(flavor, area.width.saturating_sub(6) as usize).into_iter().take(4) {
                lines.push(Line::from(vec![
                    Span::styled("   ", Style::new()),
                    Span::styled(l, Style::new().fg(Color::Rgb(150, 150, 160)).add_modifier(Modifier::ITALIC)),
                ]));
            }
        }
    }

    f.render_widget(Paragraph::new(lines), area);
}

// ── Stats-Overlay ────────────────────────────────────────────────

fn stats_overlay(f: &mut Frame<'_>, app: &App<'_>, area: Rect) {
    let th = &app.theme;
    let w = (area.width * 4 / 5).min(80);
    let h = (area.height * 4 / 5).min(24);
    let box_r = Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    };
    f.render_widget(ClearWrap, box_r);
    let deck_name = app
        .active_deck
        .as_ref()
        .map(|d| d.name.as_str())
        .unwrap_or("");
    f.render_widget(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(th.accent))
            .title(Line::from(vec![
                Span::styled(
                    format!(" {} stats ", icon::STAR),
                    Style::new().fg(th.bg).bg(th.accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {deck_name} "), Style::new().fg(th.dim)),
            ])),
        box_r,
    );

    let entries = &app.entries;
    let inner = Rect { x: box_r.x + 2, y: box_r.y + 2, width: box_r.width - 4, height: box_r.height - 3 };

    // Manakurve
    let mut cmc = [0u64; 8];
    let mut types: std::collections::HashMap<String, u64> = Default::default();
    let mut colors = [0u64; 5]; // W U B R G
    let mut total_eur = 0.0f64;
    for e in entries {
        cmc[(e.card.cmc.round() as usize).min(7)] += e.qty as u64;
        *types.entry(e.category.clone()).or_default() += e.qty as u64;
        for (i, col) in ["W", "U", "B", "R", "G"].iter().enumerate() {
            if e.card.colors_raw.contains(col) {
                colors[i] += e.qty as u64;
            }
        }
        total_eur += e.eur();
    }

    let chart_h = (inner.height * 40 / 100).max(5);
    let cmc_chart = BarChart::default()
        .data(&[
            ("0", cmc[0]), ("1", cmc[1]), ("2", cmc[2]), ("3", cmc[3]),
            ("4", cmc[4]), ("5", cmc[5]), ("6", cmc[6]), ("7+", cmc[7]),
        ])
        .bar_style(Style::new().fg(th.accent))
        .bar_width((inner.width / 12).max(3))
        .label_style(Style::new().fg(th.dim));

    let mut top_types: Vec<(String, u64)> = types.into_iter().collect();
    top_types.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    top_types.truncate(6);
    let type_data: Vec<Bar> = top_types
        .iter()
        .map(|(n, c)| {
            Bar::default()
                .label(truncate(n, 11))
                .value(*c)
                .style(Style::new().fg(th.fg))
        })
        .collect();
    let type_chart =
        BarChart::default().data(ratatui::widgets::BarGroup::new(type_data));

    let color_labels = ["W", "U", "B", "R", "G"];
    let color_data: Vec<(&str, u64)> = color_labels.iter().zip(colors.iter()).map(|(&l, &c)| (l, c)).collect();
    let color_chart = BarChart::default()
        .data(&color_data)
        .bar_width(6)
        .bar_style(Style::new().fg(th.fg));

    let total_q: i64 = entries.iter().map(|e| e.qty).sum();
    let lands: i64 = entries.iter().filter(|e| e.category == "Lands").map(|e| e.qty).sum();
    let spells: i64 = total_q - lands;
    let avg_cmc = if spells > 0 {
        entries.iter().filter(|e| e.category != "Lands")
            .map(|e| e.card.cmc * e.qty as f64).sum::<f64>() / spells as f64
    } else {
        0.0
    };
    let unique = entries.len();

    let [cmc_area, rest_top] = Layout::vertical([Constraint::Length(chart_h), Constraint::Fill(1)])
        .areas(inner);
    let [type_area, color_area] = Layout::horizontal([Constraint::Percentage(55), Constraint::Fill(1)])
        .areas(rest_top);

    f.render_widget(cmc_chart, cmc_area);
    f.render_widget(type_chart, type_area);
    f.render_widget(color_chart, color_area);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!(" {} {total_q} cards", icon::DECK), Style::new().fg(th.fg)),
            Span::styled(format!("  {unique} unique"), Style::new().fg(th.dim)),
            Span::styled(format!("  avg {avg_cmc:.1}"), Style::new().fg(th.dim)),
            Span::styled(format!("  {lands} lands"), Style::new().fg(th.dim)),
            Span::styled(format!("  {} €{total_eur:.2}", icon::PRICE), Style::new().fg(th.accent)),
        ])),
        Rect { y: inner.bottom().saturating_sub(1), height: 1, ..inner },
    );
}

// ── Search ───────────────────────────────────────────────────────

fn search(f: &mut Frame<'_>, app: &App<'_>, area: Rect) -> UiRegions {
    let th = &app.theme;
    let items: Vec<ListItem<'_>> = app
        .suggestions
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let eur = c.prices.get("eur").and_then(|v| v.as_str()).unwrap_or("");
            let marker = if i == app.sug_sel { icon::CHEVRON } else { " " };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", marker),
                    if i == app.sug_sel {
                        Style::new().fg(th.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::new().fg(th.dim)
                    },
                ),
                Span::styled(icon::CARD, Style::new().fg(if i == app.sug_sel { th.accent } else { th.dim })),
                Span::styled(" ", Style::new()),
                Span::styled(
                    truncate(&c.name, 32),
                    if i == app.sug_sel {
                        Style::new().fg(th.fg).add_modifier(Modifier::BOLD)
                    } else {
                        Style::new().fg(th.fg)
                    },
                ),
                Span::styled("  ", Style::new()),
                Span::styled(
                    format!("{}{eur}", icon::PRICE),
                    Style::new().fg(th.accent),
                ),
                Span::styled("  ", Style::new()),
                Span::styled(truncate(&c.type_line, 24), Style::new().fg(th.dim)),
            ]))
        })
        .collect();

    let inner_h = items.len() as u16 + 1;
    let box_r = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2),
        height: inner_h.min(area.height),
    };
    let title = if app.add_mode {
        Line::from(vec![
            Span::styled(
                format!(" {} add ", icon::PLUS),
                Style::new().fg(th.bg).bg(th.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" enter adds +1", Style::new().fg(th.dim)),
        ])
    } else {
        Line::default()
    };
    f.render_widget(
        List::new(items).block(
            Block::new()
                .borders(Borders::TOP)
                .border_style(Style::new().fg(th.dim))
                .title(title),
        ),
        box_r,
    );

    UiRegions {
        cover: Rect::default(),
        preview: Rect::default(),
        panel: Rect::default(),
        main: area,
    }
}

// ── Agent-Panel ──────────────────────────────────────────────────

const SPIN: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn agent_panel(f: &mut Frame<'_>, app: &App<'_>, area: Rect) {
    let th = &app.theme;
    let focused = app.focus == Focus::Agent;

    // Hintergrund: leicht dunkler für visuelle Trennung
    let bg_style = Style::new().bg(Color::Rgb(30, 32, 38));
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            let cell = &mut f.buffer_mut()[(x, y)];
            cell.set_style(bg_style);
        }
    }

    // Vertikaler Trennstrich am linken Rand
    for y in area.y..area.bottom() {
        let cell = &mut f.buffer_mut()[(area.x, y)];
        cell.set_symbol("│");
        cell.set_style(Style::new().fg(if focused { th.accent } else { th.dim }));
    }

    // Inneres Panel (nach Trennstrich)
    let inner = Rect { x: area.x + 1, width: area.width.saturating_sub(1), ..area };

    let [head, body, pins_r, input_r] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(if app.store.pinned.is_empty() && app.store.notes.is_empty() { 1 } else { 2 }),
        Constraint::Length(1),
    ])
    .areas(inner);

    // Header-Bar
    let header_bg = if focused { th.accent } else { Color::Rgb(60, 64, 72) };
    for x in head.x..head.right() {
        let cell1 = &mut f.buffer_mut()[(x, head.y)];
        cell1.set_style(Style::new().bg(header_bg));
        let cell2 = &mut f.buffer_mut()[(x, head.y + 1)];
        cell2.set_style(Style::new().bg(Color::Rgb(30, 32, 38)));
    }

    // Status
    let state: String = if app.agent_busy {
        format!("{} thinking", SPIN[app.spin_frame % SPIN.len()])
    } else if app.cfg.ready() {
        "ready".into()
    } else {
        "no key".into()
    };
    let head_style = if focused {
        Style::new().fg(th.bg).bg(header_bg).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(th.fg).bg(Color::Rgb(60, 64, 72))
    };
    f.render_widget(
        Line::from(vec![
            Span::styled(format!(" {} chat ", icon::CHAT), head_style),
            Span::styled(
                format!(" {state}"),
                Style::new().fg(if focused { th.bg } else { th.dim }).bg(header_bg),
            ),
        ]),
        Rect { x: head.x, y: head.y, width: head.width, height: 1 },
    );

    // Deck + Skills
    let skill_dots: Vec<Span<'_>> = app
        .store
        .skills
        .iter()
        .map(|k| {
            Span::styled(
                if k.enabled { icon::STAR } else { icon::STAR_OUTLINE },
                Style::new().fg(if k.enabled { th.accent } else { th.dim }),
            )
        })
        .collect();
    let deck_span = Span::styled(
        format!(
            " {} deck:{}",
            icon::DECK,
            app.active_deck.as_ref().map(|d| truncate(&d.name, 14)).unwrap_or_else(|| "—".into())
        ),
        Style::new().fg(th.dim),
    );
    let mut l2 = vec![Span::styled(" ", Style::new()), deck_span];
    l2.extend(skill_dots);
    f.render_widget(Line::from(l2), Rect { y: head.y + 1, height: 1, ..head });

    // Skill hint
    f.render_widget(
        Line::from(Span::styled(
            format!(" {} /skill name — toggle", icon::INFO),
            Style::new().fg(Color::Rgb(80, 84, 92)),
        )),
        Rect { y: head.y + 2, height: 1, ..head },
    );

    // Chat oder Deck-Picker
    if app.deck_picker {
        render_deck_picker(f, app, body);
    } else {
        render_chat(f, app, body);
    }

    // Pins/Notes
    let mut parts: Vec<Span<'_>> = vec![
        Span::styled(format!(" {} ", icon::PIN), Style::new().fg(th.dim)),
    ];
    if app.store.pinned.is_empty() && app.store.notes.is_empty() {
        parts.push(Span::styled("(p pin · /note)", Style::new().fg(th.dim)));
    } else {
        let names: Vec<String> = app.store.pinned.iter().take(3).cloned().collect();
        if names.is_empty() {
            parts.push(Span::styled("—", Style::new().fg(th.dim)));
        } else {
            parts.push(Span::styled(names.join(" · "), Style::new().fg(th.fg)));
        }
        if !app.store.notes.is_empty() {
            parts.push(Span::styled(
                format!(" {}{}", icon::NOTE, app.store.notes.len()),
                Style::new().fg(th.dim),
            ));
        }
    }
    f.render_widget(Line::from(parts), pins_r);

    // Input
    let cursor = Span::styled(
        "_",
        Style::new().fg(if focused { th.accent } else { th.dim }),
    );
    f.render_widget(
        Line::from(vec![
            Span::styled(
                format!(" {} ", icon::HAND),
                Style::new().fg(if focused { th.accent } else { th.dim }),
            ),
            Span::styled(app.agent_input.clone(), Style::new().fg(th.fg)),
            cursor,
        ]),
        input_r,
    );
}

fn render_chat(f: &mut Frame<'_>, app: &App<'_>, body: Rect) {
    let th = &app.theme;
    let width = body.width.saturating_sub(4) as usize;
    let mut lines: Vec<Line<'_>> = Vec::new();
    for m in &app.chat {
        let (style, label, icon_str) = if m.user {
            (Style::new().fg(th.accent), "you", icon::HAND)
        } else {
            (Style::new().fg(th.fg), "mtg", icon::BOT)
        };
        let wrapped = wrap(&m.text, width.max(10));
        for (i, l) in wrapped.into_iter().enumerate() {
            let prefix = if i == 0 {
                format!(" {icon_str} {label} ")
            } else {
                "         ".to_string()
            };
            lines.push(Line::from(vec![
                Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                Span::styled(l, style),
            ]));
        }
        lines.push(Line::default());
    }
    let visible = body.height as usize;
    let start = lines.len().saturating_sub(visible);
    f.render_widget(Paragraph::new(lines[start..].to_vec()), body);
}

fn render_deck_picker(f: &mut Frame<'_>, app: &App<'_>, body: Rect) {
    let th = &app.theme;
    let items: Vec<ListItem<'_>> = app
        .decks
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let marker = if i == app.deck_pick_sel { icon::CHEVRON } else { " " };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", marker),
                    if i == app.deck_pick_sel {
                        Style::new().fg(th.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::new().fg(th.dim)
                    },
                ),
                Span::styled(icon::DECK, Style::new().fg(if i == app.deck_pick_sel { th.accent } else { th.dim })),
                Span::styled(" ", Style::new()),
                Span::styled(
                    truncate(&d.name, 16),
                    if i == app.deck_pick_sel {
                        Style::new().fg(th.fg).add_modifier(Modifier::BOLD)
                    } else {
                        Style::new().fg(th.fg)
                    },
                ),
                Span::styled(
                    format!("  {}{}", icon::CARD, d.count),
                    Style::new().fg(th.dim),
                ),
            ]))
        })
        .collect();
    let _active_id = app.active_deck.as_ref().map(|d| d.id);
    f.render_widget(
        List::new(items).block(
            Block::new()
                .borders(Borders::TOP)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(th.dim))
                .title(Span::styled(
                    format!(
                        " {} pick deck ({}) ",
                        icon::DECK,
                        app.active_deck.as_ref().map(|d| truncate(&d.name, 10)).unwrap_or_else(|| "—".into())
                    ),
                    Style::new().fg(th.dim),
                )),
        ),
        body,
    );
}

// ── Helpers ──────────────────────────────────────────────────────

struct ClearWrap;

impl ratatui::widgets::Widget for ClearWrap {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        for x in area.x..area.right() {
            for y in area.y..area.bottom() {
                buf[(x, y)].reset();
            }
        }
    }
}

pub fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let cut: String = s.chars().take(n.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}

fn rarity_icon(rarity: &str) -> &'static str {
    match rarity {
        "mythic" => icon::STAR,
        "rare" => icon::STAR,
        "uncommon" => "◆",
        _ => "·",
    }
}

fn rarity_color(rarity: &str) -> Color {
    match rarity {
        "mythic" => Color::Rgb(255, 120, 40),
        "rare" => Color::Rgb(255, 200, 60),
        "uncommon" => Color::Rgb(180, 180, 200),
        _ => Color::Rgb(120, 120, 130),
    }
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let w = width.max(10);
    let mut out = Vec::new();
    for para in text.split('\n') {
        let mut line = String::new();
        for word in para.split_whitespace() {
            push_word(&mut line, word, &mut out, w);
        }
        out.push(std::mem::take(&mut line));
    }
    out.retain(|l| !l.is_empty());
    out
}

fn push_word(line: &mut String, word: &str, out: &mut Vec<String>, w: usize) {
    let mut rest = word;
    while rest.len() > w {
        let (head, tail) = rest.split_at(w);
        if !line.is_empty() {
            out.push(std::mem::take(line));
        }
        out.push(head.to_string());
        rest = tail;
    }
    if !rest.is_empty() {
        if !line.is_empty() && line.len() + rest.len() + 1 > w {
            out.push(std::mem::take(line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(rest);
    }
}
