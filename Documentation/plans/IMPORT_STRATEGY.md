# MTG Multiverse Studio — Scryfall Bulk-Import & Optimierungsstrategie

> **Strategie für effizientes Laden aller Kartendaten, Bilder & Preise**
> Stand: Juli 2026

---

## 1. Die drei Import-Stufen

Da wir **25.000+ Karten** (Oracle) bzw. **100.000+ Prints** (Default Cards) haben, muss der Import optimiert sein:

```
Stufe 1: Kartendaten (JSON)     → SQLite (einmalig, ~180 MB)
Stufe 2: Bilder (Scryfall CDN)  → Stream (on-demand) oder lokal cachen (optional)
Stufe 3: Preise (Cardmarket)    → Cachen & regelmäßig updaten
```

---

## 2. Stufe 1: Kartendaten-Bulk-Import

### Quelle
Scryfall Bulk Data: **Oracle Cards** (~180 MB, ~25.000 Karten, dedupliziert nach Regeltext)
- Endpunkt: `https://data.scryfall.io/oracle-cards/oracle-cards-20260701090258.json`
- Format: JSON (Array von Card-Objekten) oder JSONL.gz
- Täglich aktualisiert

### Optimierte Pipeline

```
1. HTTP GET https://data.scryfall.io/oracle-cards/...json
   │
2. Streaming JSON-Parser (serde_json::StreamDeserializer)
   → Jede Karte einzeln parsen, nicht alles in RAM!
   → RAM-Bedarf: ~1 MB statt 180 MB
   │
3. Für jede Karte:
   a. In CardDb konvertieren (scryfall_card_to_carddb())
   b. In SQLite einfügen (card_repo::insert_card())
   c. Ggf. Set in sets-Tabelle (INSERT OR IGNORE)
   │
4. Fortschritt: alle 1000 Karten → tracing::info!()
   │
5. Am Ende: VACUUM + ANALYZE für Query-Performance
```

### Rust-Implementierung (Pseudocode)

```rust
pub async fn import_oracle_cards(
    db: &rusqlite::Connection,
    download_uri: &str,
) -> Result<u64> {
    // 1. Streaming HTTP-Download
    let response = reqwest::get(download_uri).await?;
    let stream = response.bytes_stream();
    
    // 2. Streaming JSON parse (mit serde_json::from_reader ist NICHT streaming!
    //    Stattdessen: Chunked read + Deserializer)
    use serde_json::Deserializer;
    let reader = stream.into_async_read();
    let deser = Deserializer::from_reader(reader);
    let cards: Vec<ScryfallCard> = ScryfallCard::deserialize(deser)?;
    
    // 3. Batch-Insert (Transaktion!)
    //    NICHT jede Karte einzeln — 100er-Batches in einer Transaktion!
    let tx = db.unchecked_transaction()?;
    for (i, card) in cards.iter().enumerate() {
        let card_db = scryfall_card_to_carddb(card, &card.set);
        card_repo::insert_card(&tx, &card_db)?;
        
        if i % 100 == 0 {
            tracing::info!("Importiert: {} Karten", i);
        }
    }
    tx.commit()?;
    
    Ok(cards.len() as u64)
}
```

### Performance-Verbesserungen

| Optimierung | Effekt |
|------------|--------|
| **JSONL statt JSON** | Zeilenweises Parsen möglich (Streaming!) |
| **Batch-Insert** (100er-Transaktionen) | 10-50x schneller als Einzel-Inserts |
| **PRAGMA synchronous = OFF** | 2x schneller (risikobehaftet — nur für Import) |
| **PRAGMA cache_size = -80000** | Mehr RAM-Cache (80 MB) für schnelleren Import |
| **Ohne FTS5-Trigger** | Keine Volltext-Updates während Import |

---

## 3. Stufe 2: Bilder — Stream vs. Lokal

### Option A: Streaming (Standard, kein Speicher)
```rust
// Frontend bekommt image_url_small / image_url_large von Scryfall CDN
// Lädt Bilder direkt von https://cards.scryfall.io/...
// Kein Speicher auf Disk — aber Internet nötig
```
- ✅ 0 GB Speicher
- ✅ Immer aktuell  
- ❌ Kein Offline-Modus für Bilder
- ❌ Ladezeit bei langsamer Verbindung

### Option B: Lokaler Cache (optional, volle Kontrolle)
```rust
// Alle Bilder als WebP in ~/assets/cache/ speichern
// Frontend lädt von lokaler Datei statt CDN
```
- ✅ Volle Offline-Fähigkeit
- ✅ Schnellster Zugriff
- ❌ ~10-30 GB Speicher (alle Drucke in hoch)
- ❌ Erster Import dauert Stunden

### Empfohlene Strategie: Hybrid
```
Standard: Bild-URLs von Scryfall streamen (kein Speicher)
Option:   "Alle Bilder cachen" (User entscheidet, ob er 10GB+ opfern will)

Technisch:
- image_uris_json in cards-Tabelle speichert ALLE URLs
- Frontend prüft: Cache existiert? → lokal. Sonst → Scryfall CDN
```

---

## 4. Stufe 3: Preise — Cardmarket + TCGPlayer

### Cardmarket API (EU, empfohlen)

| Endpunkt | Beschreibung |
|----------|-------------|
| `GET /products/{id}/priceGuide` | Low, Avg, High, Trend |
| `GET /products/{id}` | Produktinfos + Bilder |
| Benötigt: API-Key (kostenlos für Hobbynutzer) |

**Preis-Caching-Strategie:**
```sql
-- Neue Tabelle für Preisverlauf
CREATE TABLE IF NOT EXISTS price_history (
    card_id TEXT REFERENCES cards(id),
    source TEXT NOT NULL,  -- 'cardmarket', 'tcgplayer', 'scryfall'
    price_type TEXT NOT NULL,  -- 'low', 'avg', 'high', 'trend'
    value REAL,
    currency TEXT DEFAULT 'EUR',
    fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_price_card ON price_history(card_id, source, fetched_at DESC);
```

**Abfrage-Intervall:**
- Normale Karten: **alle 24h**
- Beliebte/Suchende Karten: **alle 6h**  
- Keine Bulk-Abfrage (würde Cardmarket überlasten)
- Nur Preise für tatsächlich angefragte Karten laden

### Scryfall Preise (Fallback)
- In ScryfallCard.prices enthalten
- Kostenlos, aber ungenau (1x täglich aktualisiert)
- Als Fallback für "Offline"-Modus

---

## 5. Zusammenfassung Import-Architektur

```
┌─────────────────────────────────────────────────────┐
│                   App start                         │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ 1. init_db() → Tabellen + Migrationen               │
└──────────────────┬──────────────────────────────────┘
                   │ DB leer?
                   ▼
┌─────────────────────────────────────────────────────┐
│ 2. Bulk-Import (optional, per CLI)                   │
│    cargo run -- import oracle                        │
│    → ~180 MB JSON streamen                          │
│    → 25.000 Karten in SQLite                        │
│    → Fortschritt: tracing                            │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ 3. Bilder (on-demand)                                │
│    User sucht "Black Lotus"                          │
│    → image_url_small aus DB                         │
│    → React zeigt Bild von Scryfall CDN              │
│    → Optional: Lokaler Cache (User-Entscheid)       │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ 4. Preise (lazy-loaded + gecached)                   │
│    User fragt Preis an                               │
│    → Cache < 24h alt? → zurück                       │
│    → Sonst: Cardmarket API → speichern               │
│    → Fallback: Scryfall-Preis aus DB                 │
└─────────────────────────────────────────────────────┘
```
