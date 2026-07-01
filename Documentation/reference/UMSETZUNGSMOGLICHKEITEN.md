# MTG Multiverse Studio — Umsetzungsmöglichkeiten

> **Datenquellen, APIs, Frameworks und Design**

---

## Einleitung: Ziel und Kontext des Projekts

Die App **„MTG Multiverse Studio"** verfolgt das Ziel, eine umfassende, lokal laufende und datenschutzfreundliche Plattform für Magic: The Gathering (MTG) zu schaffen. Sie richtet sich an Sammlerinnen, Deckbauerinnen, Lore-Enthusiasten und Community-Mitglieder.

Im Zentrum stehen leistungsfähige Tools für **Sammlung**, **Deckbau**, **Lore-Entdeckung** und **Community-Interaktion**. Ein besonderes Alleinstellungsmerkmal ist der **Lore-Atlas**, der als Mischung aus Artbook und interaktiver Story-Collection das Storytelling-Erlebnis vertieft.

---

## 1. Datenquellen und APIs für Kartendaten, Bilder, Metadaten und Decklisten

### 1.1 Anforderungen an Kartendatenquellen

Für die Kernfunktionen der App sind hochwertige, aktuelle und strukturierte Kartendaten essenziell:

- Vollständige Kartendatensätze (Name, Typ, Oracle-Text, Edition, Seltenheit, etc.)
- Hochauflösende Kartenbilder und Artworks
- Metadaten (z.B. Legality, Preise, Community-Tags)
- Decklisten und Metagame-Daten
- Regelmäßige Updates und Synchronisation
- Rechtssichere Nutzung (Lizenzen, Fan Site Policy)

### 1.2 Vergleich zentraler Kartendatenquellen und APIs

| Quelle/API | Kartendaten | Bilder | Metadaten | Preise | Decklisten | API/Export | Lizenz/Policy | Besonderheiten |
|---|---|---|---|---|---|---|---|---|
| **Scryfall** | ✅ | ✅ | ✅ | ⚠️\* | ❌ | ✅ | Fan Policy | Bulk-Exports, Community-Tags, Artworks |
| **MTGJSON** | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | Fan Policy | Strukturierte JSON-Dumps, Editionen, Sets |
| **Gatherer** | ✅ | ✅ | ✅ | ❌ | ❌ | Eingeschränkt | Offiziell | Offizielle Texte, eingeschränkte API |
| **TCGPlayer** | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | Kommerziell | Preis- und Marktdaten, Affiliate-Programme |
| **Cardmarket** | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | Kommerziell | Europäischer Markt, Preisentwicklung |
| **EDHREC** | ❌ | ❌ | ✅ | ❌ | ✅ | Eingeschränkt | Fan Policy | Commander-Decklisten, Synergie-Daten |
| **MTGGoldfish** | ✅ | ✅ | ✅ | ✅ | ✅ | Eingeschränkt | Fan Policy | Metagame, Preisentwicklung, Turnierdecks |

> \* Scryfall: Preisangaben sind laut Dokumentation nur als grobe Orientierung geeignet und werden nicht in Echtzeit aktualisiert.

### Analyse und Empfehlungen

- **Scryfall** ist die zentrale Quelle für Kartendaten, Bilder und Metadaten. Die API bietet Bulk-Exports (Oracle Cards, Unique Artwork, Default Cards, All Cards) und Community-Tags.
- **MTGJSON** liefert strukturierte, maschinenlesbare Kartendaten — ideal für lokale Speicherung und Offline-Nutzung.
- **Gatherer** ist die offizielle Datenbank von Wizards of the Coast — wertvoll für offizielle Texte und Rulings.
- **TCGPlayer & Cardmarket** bieten Preis-/Marktdaten (kommerziell lizenziert).
- **EDHREC & MTGGoldfish** sind wichtige Quellen für Decklisten, Metagame-Analysen und Synergie-Daten.

### 1.3 Bild-Assets und Lizenzfragen

Die Nutzung von Kartenbildern und Artworks unterliegt den Rechten von Wizards of the Coast:

- Scryfall stellt Bilder bereit, die unter der **Wizards Fan Site Policy** genutzt werden dürfen (keine kommerzielle Nutzung, Quellen korrekt angeben).
- Fan-Artworks und Community-Inhalte können optional eingebunden werden (müssen moderiert und rechtlich geprüft werden).
- Für hochauflösende Artworks empfiehlt sich die Integration von **Artbook-Assets** und offiziellen Wizards-Quellen.

### 1.4 Integration und Synchronisation

- Datenquellen werden über APIs und regelmäßige Bulk-Exporte integriert.
- Lokale Caching-Strategien und Datenbanken (**SQLite**) ermöglichen Offline-Nutzung und schnelle Abfragen.
- Für die Synchronisation von Preis- und Deckdaten sind automatisierte Update-Prozesse erforderlich.

---

## 2. Tools und Plattformen für Deckbau und Analyse

### 2.1 Marktüberblick: Deckbau- und Analyse-Tools

| Tool/Plattform | Deckbau | Sammlung | Analyse | Playtesting | Import/Export | Community | Besonderheiten |
|---|---|---|---|---|---|---|---|
| **GrimDeck** | ✅ | ✅ (+Scanner) | ✅ | ❌ (Goldfish geplant) | ✅ | Wächst | KI-gestützte Empfehlungen |
| **Moxfield** | ✅ | Basic | ✅ | ❌ (Goldfish) | ✅ | Groß | Clean Editor, Packages, Sharing |
| **Archidekt** | ✅ | ❌ | ✅ | ❌ (Goldfish) | ✅ | Aktiv | Auto-Kategorisierung, Bracket-Scoring |
| **MTGGoldfish** | ✅ | Premium | ✅ | ❌ | ✅ | Groß | Metagame, Preisentwicklung, Turnierdecks |
| **Deckstats** | ✅ | ✅ (CSV) | ✅ | ❌ (Goldfish) | ✅ | Klein | Probability Analysis, CSV-Import |
| **Scryfall** | ✅ (Basic) | ❌ | ❌ | ❌ | ❌ | ✅ | Beste Kartensuche, Listenexport |
| **AetherHub** | ✅ | Arena | Basic | ❌ | ✅ | Klein | Arena-Integration, Metagame |
| **ManaStack** | ✅ | ❌ | Basic | ✅ (Multiplayer) | ✅ | Klein | Multiplayer-Playtesting im Browser |

### Analyse der Nutzerpräferenzen und Feature-Wünsche

- **Sammlung & Deckbau vereint:** Nutzer:innen schätzen Tools, die Sammlung und Deckbau in einem Workflow vereinen.
- **Analyse-Tools:** Mana-Kurve, Farbbalance, Synergie-Analysen und Format-Legalität.
- **Playtesting:** Simuliertes Ziehen von Starthänden (Goldfishing) und Playtesting gegen KI oder eigene Decks.
- **Import/Export:** Unterstützung gängiger Formate (CSV, Arena, MTGJSON, Scryfall).
- **Community & Sharing:** Deck-Sharing, Kommentare, Community-Features.
- **KI-gestützte Empfehlungen:** Synergien, Budget-Alternativen, Optimierungen.

### 2.2 Synergie-Analyse und KI-gestützte Empfehlungen

Moderne Tools nutzen KI-gestützte Algorithmen:

- **Synergie-Vorschläge:** Automatische Erkennung von Kartenkombinationen (z.B. Cabal Coffers + Urborg).
- **Deck-Beschreibungen:**Automatisch generierte Zusammenfassungen der Deckstrategie.
- **Interaktions-Scoring:** Bewertung der Synergie-Stärke und Relevanz für das Deck.
- **Import von Decklisten:** Unterstützung für Moxfield, Archidekt und andere Plattformen.
- **KI-Integration:** Nutzung von Embeddings und semantischer Suche für kontextbezogene Empfehlungen.

### 2.3 Technische Umsetzung: Import, Analyse und Testing

- **Import-Workflows:** CSV, MTGJSON, Scryfall-Listen, Arena-Formate.
- **Karten-Scanner:** Mobile Scanning (OCR, Barcode) zur schnellen Erfassung physischer Karten.
- **Analyse-Module:** Echtzeit-Berechnung von Mana-Kurve, Farbbalance, Synergien und Wahrscheinlichkeiten.
- **Testing-Modus:** Simuliertes Ziehen von Starthänden, Goldfishing, Playtesting gegen KI oder eigene Decks.
- **Export-Optionen:** Austausch mit anderen Plattformen und Tools.

---

## 3. Lore-Integration: Quellen, Storytelling und multimediale Storybooks

### 3.1 Offizielle und inoffizielle Lore-Quellen

| Quelle/Medium | Inhaltstyp | Umfang | Zugang | Lizenz/Policy |
|---|---|---|---|---|
| **Wizards Magic Story** | Kurzgeschichten, Lore | Sehr umfangreich | Online | Offiziell, Fan Policy |
| **MTG Wiki** | Zusammenfassungen, Analysen | Umfangreich | Online | Community, CC |
| **Romane/Novellen** | Vollständige Storys | Umfangreich | Print/E-Book | Kommerziell |
| **Comics/Manga** | Storylines, Charaktere | Umfangreich | Print/Digital | Lizenzabhängig |
| **Artbooks/Visual Guides** | Artworks, Welten, Story | Umfangreich | Print/Digital | Kommerziell |
| **D&D-Produkte** | Welten, Charaktere | Umfangreich | Print/Digital | Lizenzabhängig |

### 3.2 Integration in die App: Lore-Atlas und Storytelling

Der **Lore-Atlas** ist das Herzstück für Lore- und Story-Fans:

- **Artbook-Funktion:** Hochwertige Darstellung von Karten-Artworks, Konzeptzeichnungen und Illustrationen.
- **Story-Collection:** Sammlung offizieller Storys, Kurzgeschichten, Charakterprofile und Weltenbeschreibungen.
- **Interaktive Storybooks:** Entscheidungsbäume, alternative Handlungsstränge, multimediale Inhalte.
- **Verknüpfungen:** Direkte Links von Karten zu relevanten Storys, Charakteren und Welten.
- **Fan Content:** Optionale Einbindung von Fan-Geschichten, Artworks oder Analysen (abhängig von Lizenz/Moderation).

### 3.3 Technische Umsetzung: Datenstruktur und Asset-Management

- **Datenstruktur:** `/assets/artbook`, `/assets/stories` — klare Ordnerstrukturen für Texte und Bilder.
- **Verknüpfungen:** Karten, Charaktere und Storys werden über IDs und Relationen verknüpft.
- **Asset-Management:** Lokale Speicherung von Bildern und Texten, optional CDN-Synchronisation.
- **Multimedia-Integration:** Unterstützung für Bilder, Audio, Video und interaktive Elemente in Storybooks.
- **Lizenzmanagement:** Klare Prüfung der Nutzungsrechte; Moderation von Fan Content.

---

## 4. Frameworks und Technologien für Frontend, Backend und KI-Integration

### 4.1 Architekturüberblick

Die App wird als Desktop-Anwendung mit Fokus auf **Offline-Fähigkeit** und **Datenschutz** entwickelt:

| Komponente | Beschreibung |
|---|---|
| **Frontend** | Benutzeroberfläche, Interaktion, Rendering (2D/3D) |
| **Backend (lokal)** | Datenhaltung, Business-Logik, KI-Module |
| **Datenquellen** | Lokale Datenbanken, API-Schnittstellen, Asset-Management |

### 4.2 Vergleich der empfohlenen Frameworks und Technologien

| Komponente | Empfohlene Technologien | Begründung |
|---|---|---|
| **Frontend** | React (mit Electron), TypeScript, Zustand/Redux | Moderne UI, Desktop-Integration, Typisierung |
| **3D-Grafik** | Three.js, React-Three-Fiber, WebGL | Leistungsfähige 3D-Visualisierung |
| **Styling** | Tailwind CSS, Styled Components, Design-System | Konsistentes, anpassbares Styling |
| **Backend (lokal)** | Node.js (Electron), Python (KI) | Lokale Verarbeitung, KI-Integration |
| **Datenbank** | SQLite (lokal), IndexedDB, Realm | Offline-First, schnelle Abfragen |
| **Suche/Indexierung** | Lunr.js, Elasticlunr, FAISS | Schnelle Volltext- und semantische Suche |
| **KI-Assistent** | On-Device LLM (Llama.cpp, GGML), Sentence Transformers | Datenschutz, lokale Verarbeitung |
| **Asset-Management** | Lokale Verzeichnisse, optional CDN | Kontrolle über Daten, Offline-Nutzung |

### Analyse der Stack-Empfehlungen

- **React + Electron:** Moderne, performante Desktop-App mit Webtechnologien und nativer Integration.
- **Three.js & React-Three-Fiber:** Führend für 3D-Visualisierungen im Browser/Desktop — ideal für den 3D-Graphen.
- **Tailwind CSS & Styled Components:** Konsistentes, modulares und anpassbares UI-Design.
- **SQLite & IndexedDB:** Schnelle, lokale Datenhaltung; unterstützt die Offline-First-Strategie.
- **Lunr.js & FAISS:** Schnelle Volltext- und semantische Suche — essenziell für Navigation in großen Datenmengen.
- **On-Device LLMs** (Llama.cpp, GGML): Datenschutzfreundlicher, lokaler KI-Assistent.

### 4.3 Performance- und Architekturstrategien

- **Effizientes Rendering:** WebGL + Three.js mit dynamischem Level-of-Detail (LOD) und Culling.
- **Lazy Loading:** Nachladen von Assets nur bei Bedarf — spart Speicher.
- **Caching:** Lokale Speicherung häufig genutzter Daten und Assets.
- **Datenkompression:** Komprimierte Assets (z.B. WebP für Bilder).
- **Asynchrone Prozesse:** Hintergrund-Threads für aufwändige Berechnungen (KI, Suche).
- **Profiling & Optimierung:** Regelmäßige Performance-Analysen.

### 4.4 Beispiel: Moderne Webarchitektur (Rust Backend, React Frontend)

Ein modernes Architekturbeispiel kombiniert einen performanten **Rust-Backend-Server** (z.B. mit Axum) mit einem **React-Frontend**:

- Hohe Performance, Sicherheit, Skalierbarkeit
- Klare Trennung von UI und Business-Logik
- Datenbanken: PostgreSQL, SQLite
- Authentifizierung: JWT
- Containerisierung: Docker
- Caching-Strategien

---

## 5. UI/UX-Design: Modern, Minimalistisch, Typografisch stark, Storytelling-fähig

### 5.1 Design-Philosophie und Prinzipien

- **Klarheit und Übersicht:** Intuitive Navigation, klare Informationshierarchien.
- **Modularität:** Wiederverwendbare UI-Komponenten, konsistentes Styling.
- **Barrierefreiheit:** Screenreader-Support, Tastaturnavigation, ausreichende Kontraste.
- **Dark/Light Mode:** Anpassbare Themes für unterschiedliche Präferenzen.
- **Artbook-Ästhetik:** Hochwertige Darstellung von Artworks und Storys (insb. im Lore-Atlas).
- **Responsive Design:** Optimierung für verschiedene Bildschirmgrößen (primär Desktop, optional Tablet).

### 5.2 Design-Vorschläge für ein modernes, minimalistisches UI

Inspirationen finden sich auf Plattformen wie Dribbble — klare Linien, großzügige Weißräume, reduzierte Farbpaletten und starke Typografie:

- **Großzügige Weißräume:** Erhöhen die Lesbarkeit, lenken den Fokus auf Inhalte.
- **Starke Typografie:** Klare Schriftarten; Hierarchien durch Größe, Gewicht und Farbe.
- **Reduzierte Farbpalette:** Akzentfarben für Interaktionen; neutrale Grundfarben für Hintergrund und Flächen.
- **Kartenbasierte Layouts:** Übersichtliche Darstellung von Karten, Decks, Storys und Artworks.
- **Bild/Text-Kombinationen:** Storytelling durch die Kombination von Bildern, Illustrationen und begleitenden Texten.
- **Interaktive Elemente:** Hover-Effekte, Animationen und Microinteractions für ein modernes Nutzererlebnis.

### 5.3 Storytelling und multimediale Storybooks

- **Interaktive Storybooks:** Geschichten nicht nur lesend, sondern interaktiv erleben — Entscheidungsbäume, alternative Handlungsstränge, multimediale Inhalte (Bilder, Audio, Video).
- **Verknüpfung von Karten und Storys:** Direkte Links von Karten zu relevanten Storys, Charakteren und Welten.
- **Artbook-Elemente:** Hochwertige Präsentation von Artworks, Konzeptzeichnungen und Illustrationen — inspiriert von offiziellen Artbooks und Visual Guides.

---

## 6. Community-Funktionen, Moderation und Plugin-Architektur

### 6.1 Community-Features und Nutzerpräferenzen

Nutzer:innen schätzen folgende Community-Features:

- **Inhalte teilen und tauschen:** Decklisten, Sammlungen, Storys und Artworks können geteilt und diskutiert werden.
- **Gemeinsames Arbeiten:** Kollaborative Projekte (z.B. gemeinsames Deckbuilding oder Storywriting).
- **Moderation:** Klare Richtlinien und Moderationssysteme für Fan Content und Community-Beiträge.
- **Plugin-Architektur:** Möglichkeit, eigene Erweiterungen, Visualisierungen oder Datenquellen als Plugins zu integrieren.

### 6.2 Technische Umsetzung: Erweiterbarkeit und Sicherheit

- **Modulare Erweiterungen:** Neue Features, Visualisierungen oder Datenquellen können als Plugins integriert werden.
- **Schnittstellen:** Klare APIs für den Zugriff auf Kernfunktionen und Daten.
- **Sandboxing:** Plugins laufen isoliert — Sicherheit und Stabilität gewährleistet.
- **Community-Marktplatz:** Optionale Plattform für den Austausch von Plugins und Inhalten.

---

## 7. Rechtliche Risiken, IP und Datenschutz

### 7.1 Lizenzierung und Urheberrecht

- **Wizards IP:** Nutzung von Kartenbildern, Storys und Artworks unterliegt den Rechten von Wizards of the Coast. Die Fan Site Policy erlaubt die Nutzung für nicht-kommerzielle Projekte (Quellen korrekt angeben).
- **Fan Content:** Fan-Artworks und Community-Inhalte müssen moderiert und rechtlich geprüft werden.
- **Preis- und Marktdaten:** Kommerzielle APIs (TCGPlayer, Cardmarket) erfordern die Beachtung der jeweiligen Nutzungsbedingungen.

### 7.2 Datenschutz und Offline-First-Strategie

- **Lokale Datenhaltung:** Alle Nutzerdaten lokal gespeichert — keine Cloud-Pflicht.
- **GDPR-Konformität:** Keine Speicherung oder Verarbeitung personenbezogener Daten auf externen Servern.
- **On-Device KI:** Der KI-Assistent läuft lokal — Datenschutz und Offline-Nutzung gewährleistet.

---

## 8. Feature-Priorisierung, MVP-Roadmap und Erfolgskriterien

### 8.1 Prioritäten und Entwicklungsphasen

#### MVP (Minimum Viable Product)
- Collection Management (Basisfunktionen)
- Deck-Labor (Grundfunktionen)
- Integration von MTGJSON/Scryfall
- Basis-UI und Design-System
- Lokale Datenhaltung (SQLite/IndexedDB)

#### v1.0
- Erweiterte Deck-Analyse und Testing
- Lore-Atlas (Artbook, Story-Collection)
- 3D-Graph (Basisversion)
- Lokaler KI-Assistent (Basisfunktionen)
- Import/Export, Sharing
- Performance-Optimierungen

#### v2.0
- Interaktive Storybooks im Lore-Atlas
- Erweiterte 3D-Graph-Features (z.B. Zeitachsen)
- Community-Features (Tausch, Fan Content)
- Plugin-Architektur für Modding
- Internationalisierung, Lokalisierung
- Erweiterte KI-Funktionen (semantische Suche, Personalisierung)

### 8.2 Erfolgskriterien und KPIs

Der Erfolg der App wird anhand folgender Kriterien gemessen:

- **Nutzerzufriedenheit** (Feedback, Retention, Churn-Rate)
- **Performance** (Ladezeiten, Interaktionsgeschwindigkeit)
- **Feature-Nutzung** (Häufigkeit der Kernfunktionen)
- **Community-Wachstum** (aktive Nutzer:innen, geteilte Inhalte)
- **Datenschutz** (keine Datenschutzverletzungen)
- **Erweiterbarkeit** (Integration neuer Features, Plugins)
- **Fehlerquote** (kritische Bugs, stabile Releases)

---

## 9. Testing, Playtesting und probabilistische Analyse

- **Testing-Modus:** Simuliertes Ziehen von Starthänden, Goldfishing, Playtesting gegen KI oder eigene Decks.
- **Probabilistische Analyse:** Berechnung von Wahrscheinlichkeiten (z.B. Ziehchancen, Mana-Kurve) zur Optimierung von Decks.
- **Performance-Tests:** Regelmäßige Profiling- und Optimierungsmaßnahmen — flüssige Nutzung auch auf durchschnittlicher Hardware.

---

## 10. Dokumentation, Onboarding und Support

- **In-App-Tutorials:** Schritt-für-Schritt-Anleitungen für neue Nutzer:innen.
- **Hilfecenter:** FAQ, Troubleshooting, Kontaktmöglichkeiten.
- **Community-Support:** Foren, Discord-Server oder ähnliche Plattformen für Austausch und Hilfe.
- **Entwicklerdokumentation:** Ausführliche technische Dokumentation für Erweiterungen und Plugins.

---

## Fazit

**MTG Multiverse Studio** vereint leistungsfähige Tools für Sammlung, Deckbau und Lore-Entdeckung in einer datenschutzfreundlichen, lokal laufenden Desktop-App. Die Integration hochwertiger Datenquellen (Scryfall, MTGJSON, Gatherer, EDHREC, MTGGoldfish), moderner Frameworks (React, Electron/Tauri, Three.js, Tailwind CSS, SQLite, On-Device LLMs) und eines klaren, minimalistischen UI-Designs schafft eine innovative Plattform für MTG-Fans.

Der besondere Fokus auf den **Lore-Atlas** als interaktives Storytelling- und Artbook-Element hebt die App von bestehenden Lösungen ab. Risiken werden durch proaktive Maßnahmen adressiert, und der Erfolg wird anhand klar definierter Kriterien gemessen. Die geplante Plugin-Architektur und Community-Features sichern die langfristige Weiterentwicklung und Relevanz der App.

> **Die Umsetzungsmöglichkeiten sind vielfältig** und orientieren sich an den besten verfügbaren Datenquellen, Technologien und Designprinzipien. Die App ist sowohl für Einsteiger:innen als auch für erfahrene MTG-Fans attraktiv und bietet eine zukunftssichere, erweiterbare Plattform für die Verwaltung, Analyse und das Erleben des MTG-Multiversums.
