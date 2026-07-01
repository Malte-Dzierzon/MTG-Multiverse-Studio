# MTG Multiverse Studio — Feature Research & Datenquellen

> **Recherche:** Juli 2026  
> **Quellen:** Scryfall API, MTGGoldfish, EDHREC, Cardmarket, Moxfield, Archidekt

---

## 1. Kartendaten & Bulk-Import (Scryfall)

**Aktuelles Problem:** Unsere Datenbank ist leer. `search_cards` findet nix.

**Scryfall Bulk Data** liefert komplette Kartendumps als JSON/JSONL.gz:

| Dataset | Größe | Beschreibung |
|---------|-------|-------------|
| **Oracle Cards** | ~180 MB | 1 Karte pro Oracle-ID (dedupliziert nach Regeltext) — **perfekt für Offline-Suche** |
| **Unique Artwork** | ~260 MB | 1 Karte pro Artwork (beste Scans) — **perfekt für Artbook** |
| **Default Cards** | ~550 MB | Alle englischen Karten + fremdsprachige Exklusive |
| **All Cards** | ~2.5 GB | Jeder Druck in jeder Sprache |

**Empfehlung:** `Oracle Cards` für die Kartensuche + `Unique Artwork` für Bilder.  
➡ Braucht einen **Bulk-Import**-Befehl, der das JSON streamt und in SQLite speichert.

---

## 2. Preise (Real-Time & Verlauf)

### Scryfall (kostenlos, aber nicht live)
```json
{
  "usd": "0.25",
  "usd_foil": "1.50",
  "usd_etched": null,
  "eur": "0.20",
  "eur_foil": "1.20",
  "eur_etched": null,
  "tix": "0.03"
}
```
⚠️ Preise sind veraltet (werden nur ~1x täglich aktualisiert).

### Cardmarket API (empfohlen für EU-Preise)
- **Single Card Price:** Verkaufspreis in EUR
- **Price Guide:**
  - `avg` — Durchschnittspreis
  - `low` — Günstigster
  - `high` — Höchster
  - `trend` — Preistrend
- **Historical Data:** Preisverlauf über Zeit
- Braucht API-Key (kostenlos für Hobby-Projekte)

### TCGPlayer API (empfohlen für US-Preise)
- `low`, `mid`, `high`, `market` Preis
- Preisverlauf, Verkaufsvolumen
- Braucht Affiliate-Partner-Account

**Empfehlung:** Scryfall als Fallback + Cardmarket (EU) + TCGPlayer (US) als "Live-Abfrage"-Option. Preise in SQLite cachen und alle 6h updaten.

---

## 3. Deckbau-Features (von Moxfield, Archidekt, MTGGoldfish)

### Moxfield — Der Clean-Editor
| Feature | Details |
|---------|---------|
| Drag & Drop | Karten per Suche + Quantität |
| Auto-Kategorisierung | Creatures, Spells, Lands etc. |
| Mana-Kurve | Balkendiagramm nach CMC |
| Farbbalance | Kreisdiagramm |
| Legality-Check | Rot/Grün pro Format |
| Deck Sharing | Öffentliche URL + Embed |
| Primär/Secondary | Mainboard + Sideboard |
| Packages | Vorgefertigte Kartengruppen |

### Archidekt — Der Auto-Kategorisierer
| Feature | Details |
|---------|---------|
| Auto-Categorization | Sortiert Karten nach Funktion (Ramp, Draw, Removal…) |
| Bracket Scoring | Bewertet Deck-Stärke (1-5) für Commander |
| Mana Curve | Erweiterte Curve + Mana-Quellen-Analyse |
| Goldfishing | Simuliertes Ziehen |
| Price Breakdown | Kosten pro Kategorie |
| Tags | Eigene Tags + Farbe pro Karte |

### MTGGoldfish — Metagame-Analyse
| Feature | Details |
|---------|---------|
| **Metagame %** | Welches Deck wird wie oft gespielt? |
| **Deck Cost** | Gesamtpreis in $ |
| **Format-Ranking** | Standard, Modern, Pioneer, Commander… |
| **Card Prices** | +Preis-Charts mit Verlauf |
| **Turnierdaten** | Top-8 Decks von Events |

### Gemeinsame Kern-Features (müssen rein)
- ✅ Mana-Kurve (haben wir schon im Backend!)
- ✅ Farbbalance (haben wir schon!)
- ❌ Legality-Check pro Format
- ❌ Goldfishing / Starthand-Simulator
- ❌ Price-Breakdown (was kostet mein Deck?)
- ❌ Auto-Kategorisierung (Ramp, Draw, Removal, etc.)

---

## 4. EDHREC — Commander-Synergien

EDHREC ist die #1 Seite für **Commander**:

| Feature | Beschreibung |
|---------|-------------|
| **Synergy-Ranking** | "Welche Karten werden am häufigsten mit Kommandant X gespielt?" |
| **Card Tags** | Community-getaggte Funktionen (Ramp, Boardwipe, Tutor…) |
| **Top Cards** | Beliebteste Karten in Farbe/Farbkombination |
| **Deck Stats** | Ø Mana-Wert, Ø Kartentypen-Verteilung |
| **Precon Upgrades** | "Was sollte man in einem Precon-Deck als erstes tauschen?" |

Das sind **alles reine Backend-Analysen** — man braucht nur genug Deck-Daten (können von EDHREC/Archidekt/Moxfield via API oder Scraping kommen).

---

## 5. Was wir jetzt schon haben (Backend) ✅

| Feature | Status |
|---------|--------|
| SQLite mit 6 Tabellen | ✅ Fertig |
| Karten suchen (lokal) | ✅ Fertig |
| Sammlung verwalten | ✅ Fertig (collection_repo) |
| Decks erstellen + verwalten | ✅ Fertig (deck_repo) |
| Karten zu Deck hinzufügen | ✅ Fertig |
| Mana-Kurve berechnen | ✅ Fertig (deck_service) |
| Farbbalance berechnen | ✅ Fertig (deck_service) |
| Lore-Einträge + Markdown | ✅ Fertig (lore_service) |
| Scryfall-Client (Rate-Limited) | ✅ Fertig |
| 3-Stufen-Cache (DB→LRU→API) | ✅ Fertig |

## 6. Was noch fehlt (Backend-Seite) 🔴

| Feature | Warum wichtig | Quellen |
|---------|--------------|---------|
| **Bulk-Import** | DB ist leer — ohne Import nix | Scryfall Bulk Data |
| **Live-Preise** | Cardmarket + TCGPlayer | Cardmarket API, TCGPlayer API |
| **Preisverlauf** | "Steigt oder fällt Karte X?" | Cardmarket Historical |
| **Format-Legality** | Rot/Grün pro Format | Scryfall (haben wir die Daten, aber kein Check) |
| **Auto-Kategorisierung** | Ramp, Draw, Removal, etc. | Machine Learning / Regeln |
| **Goldfishing** | Starthand ziehen | Eigene Berechnung |
| **Synergie-Analyse** | "Was passt zu Kommandant X?" | EDHREC-Daten + eigene Analyse |
| **Deck Import/Export** | CSV, Arena, Moxfield-Link | Parsing |
| **Metagame-Statistiken** | "Was wird gerade gespielt?" | MTGGoldfish / Eigene Daten |

---

## 7. Nächste konkrete Schritte (meine Empfehlung)

1. 🔴 **Bulk-Import bauen** — Scryfall Oracle Cards JSON laden → in SQLite speichern. Mit Fortschrittsbalken.
2. 🔴 **CLI-Test-Harness** — `cargo run -- search "Black Lotus"` und `cargo run -- import` 
3. 🟡 **Cardmarket-Preis-Integration** — API-Abfrage + Caching
4. 🟡 **Format-Legality-Check** — Frontend sagt "gültig"/"ungültig"
5. 🟢 **Goldfishing** — 7 Karten ziehen, mulligan, wiederholen
6. 🔵 **Synergie-Analyse** — Die Königsdisziplin
