# MTG Multiverse Studio — Dokumentation

Willkommen im `Documentation/`-Verzeichnis des MTG Multiverse Studio.  
Dieser Ordner enthält alle Projekt-Dokumentationen, organisiert nach thematischen Unterordnern.

## Ordnerstruktur

```
Documentation/
├── README.md                    ← Diese Datei
├── guides/                      ← Anleitungen & Integration Guides
│   └── FRONTEND_INTEGRATION.md  ← Rust → React Frontend-Anbindung
├── plans/                       ← Technische Architektur- & Umsetzungspläne
│   ├── IMPORT_STRATEGY.md       ← 3-Stufen-Bulk-Import-Plan
│   └── RUST_BACKEND_PLAN.md     ← Rust-Backend-Architektur
├── reference/                   ← Allgemeine Recherchen & Feature-Analysen
│   ├── FEATURE_RESEARCH.md      ← Feature-Analyse von MTG-Plattformen
│   └── UMSETZUNGSMOGLICHKEITEN.md ← MTG-Ökosystem-Analyse
├── roadmap/                     ← Projekt-Roadmap & Meilensteine
│   └── ROADMAP.md               ← Roadmap mit aktuellem Status
└── reviews/                     ← Code-Reviews (wird von Sub-Agents befüllt)
```

## Beschreibung der Unterordner

| Ordner | Inhalt |
|---|---|
| `guides/` | How-to-Anleitungen, Setup-Guides, Integrationsdokumente |
| `plans/` | Technische Architekturpläne, Migrationsstrategien, Design-Dokumente |
| `reference/` | Allgemeine Recherchen (Feature-Analysen, Ökosystem-Übersichten) |
| `roadmap/` | Meilensteine, Prioritäten, Status-Tracking |
| `reviews/` | (vorbereitet) Code-Review-Dokumentation |

## Hinweise

- Alle Dateiinhalte bleiben unverändert — organisiert wird nur die Ordnerstruktur.
- Relative Links zwischen Dokumenten innerhalb dieses Verzeichnisses sind nach Möglichkeit auf dem neuesten Stand.
- Bei Fragen zur Architektur: Siehe `plans/RUST_BACKEND_PLAN.md`.
- Für den aktuellen Projektstatus: Siehe `roadmap/ROADMAP.md`.
