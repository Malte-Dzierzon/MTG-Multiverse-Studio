# Story & Lore Datenstruktur

## Übersicht

Dieser Ordner enthält die gesamten **Story-, Lore- und Artbook-Daten** für MTG Multiverse Studio.  
Da Magic: The Gathering über 30+ Jahre an Lore umfasst, ist eine strukturierte Organisation essenziell.

---

## Verzeichnisstruktur

```
assets/stories/
├── en/                    # Englisch (primäre Sprache)
│   ├── planes/            # Plane-Profile
│   │   ├── dominaria.md   # Eine Datei pro Plane
│   │   ├── theros.md
│   │   └── ...
│   ├── characters/        # Character-Profile
│   │   ├── gideon-jurgen.md
│   │   ├── liliana-vess.md
│   │   └── ...
│   ├── eras/              # Zeitepochen (chronologisch)
│   │   ├── era_01_origin.md      # Ur-MTG / Alpha 1993
│   │   ├── era_28_beyond-suspect.md  # Aktuellste Saga
│   │   └── ...
│   ├── sagas/             # Große Story-Arcs (z.B. Gatewatch, War of the Spark)
│   │   ├── saga_gatewatch.md
│   │   └── ...
│   └── articles/          # Einzelne Kurzgeschichten aus MagicStory
│       ├── 2024/           # Nach Jahr sortiert
│       └── ...
├── de/                    # Deutsch (Übersetzungen)
│   ├── planes/
│   ├── characters/
│   └── ...
└── assets/                # Bilder zu den Stories
    ├── artbook/            # Artworks (hohe Auflösung)
    │   ├── dominaria/
    │   ├── theros/
    │   └── ...
    └── covers/             # Cover-Art von Sets/Eras

```

---

## Dateiformate & Metadaten

### Markdown mit YAML Frontmatter (Empfohlen)

Jede Story-Datei nutzt **Markdown** und beginnt mit YAML-Metadaten:

```yaml
---
title: "War of the Spark"
type: saga              # saga | era | plane | character | article
plane: Ravnica          # Welche Plane ist betroffen?
year: 2019
sets: ["Dominaria", "Rivals of Ixalan"]
characters:
  - gideon-jurgen
  - nissa-rev
artwork: artbook/ravnica/wotsp-cover.jpg
language: en
related_links:
  - type: plane, ref: ravnica
  - type: character, ref: gideon-jurgen
---

# War of the Spark

Garruk unleashes an invasion...
```

**Vorteile:**
- Die App kann Metadaten automatisch parsen (Titel, Datum, Verknüpfungen)
- Cross-Referencing zwischen Karten, Charakteren und Events möglich  
- Such-Indexierung über die YAML-Felder einfach machbar

---

## Woher bekommst du die Daten?

### 1. Offizielle Quellen (Fan Policy-konform)

| Quelle | Inhalt | Format | Zugriff |
|--------|--------|--------|---------|
| **[MagicStory.net](https://magicrology.com/)** | Alle Kurzgeschichten & Artikel | Online lesen + Screenshots | Free, aber kein API-Export |
| **[Uncharted Realms](https://unchartedrealms.wizards.com/)** | Lore-Artikel, Plane-Guides | Webartikel | Kein offizieller Export |
| **[MTG Wiki](https://magic.fandom.com/wiki/Magic:_The_Gathering_Wiki)** | Zusammenfassungen aller Arcs | Wikipedia-ähnlich | Kann gecrawlt werden (manuell) |

### 2. Community-Ressourcen  

| Quelle | Inhalt | Format | Zugriff |
|--------|--------|--------|---------|
| **MTG Wiki (Fandom)** | Chronologien, Plane-Liste, Character-Bios | Web | Manuell kopieren oder crawlen |
| **Reddit r/MagicLore** | Zusammenfassungen & Guides | Forum | Manuell |

### 3. Offline-Material

| Quelle | Inhalt | Format | Zugriff |
|--------|--------|--------|---------|
| **Harper Prism Romane** | Planeswalker-Saga, Block-Novels | E-Book/Print | Eigenes OCR oder Transkription nötig |
| **Comics (ARMADA, etc.)** | Visuelle Storys | Print/Digital | Screenshots + Manuskript |

---

## Workflow: Neue Story einpflegen

1. **Text beschaffen** → Von MagicStory.net kopieren ODER MTG Wiki zusammenfassen
2. **Markdown-Datei erstellen** mit YAML Frontmatter (siehe Template oben)  
3. **Bilder herunterladen** (Scryfall Artworks, MagicStory-Cover) in `assets/artbook/`
4. **Verknüpfungen pflegen** → Characters und Planes referenzieren
5. **Lokale DB indexieren lassen** (die Rust-Backend liest alle `.md` Files beim Start ein)

---

## Schätzung der Datenmenge

| Kategorie | Anzahl geschätzt | Größe pro Eintrag | Gesamtgröße |
|-----------|-----------------|------------------|-------------|
| Planes | ~30+ | 5–20 KB Text | ~400 KB |
| Characters | ~150+ | 2–10 KB | ~1 MB |  
| Sagas/Arcs | ~80+ | 10–50 KB | ~3 MB |
| Einzel-Artikel (MagicStory) | ~2.000+ | 5–30 KB | ~60 MB |
| Artbook-Bilder | ~500+ | 500 KB – 2 MB | **~500 MB** |

> ⚠️ Die reinen Texte bleiben unter **100 MB**, aber Bilder können schnell in den **GB-Bereich** gehen.  
> Empfehlung: Bilder lazy-loaden und nur bei Bedarf herunterladen (über Scryfall API).

---

## Alternative Ansätze

### Option A: Alles lokal als Markdown (wie jetzt)
- ✅ Volle Kontrolle, offline-fähig, einfach zu editieren
- ❌ Manuelle Pflegeaufwand groß (~2.000+ Artikel)

### Option B: Scryfall + API für Kartentexte, Lore manuell
- ✅ Karten-Texte automatisch aktuell  
- ❌ Lore-Daten gibt es nicht via API → muss trotzdem manuell rein

### Option C: Hybrid — Markdown-Templates + Sync-Script
- Script crawelt MagicStory.net/MTG Wiki automatisch
- Generiert Markdown-Dateien mit Frontmatter
- Manuelles Review für Qualität
- ✅ Beste Lösung — automatisiert den größten Teil der Arbeit
