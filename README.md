# Rust Desktop GUI- & Sprach-Lernlabor

Ein interaktives Desktop-Lernlabor, das die Kernkonzepte der Programmiersprache **Rust** mit moderner, nativer GUI-Entwicklung auf Basis von **`eframe` / `egui`** verbindet.

Die Anwendung ermöglicht es, theoretische Rust-Konzepte (wie Ownership, Borrowing, Concurrency, Lifetimes und Enums) direkt über eine grafische Oberfläche auszuprobieren und den dazugehörigen Code anhand eindeutiger Tags im Quelltext nachzuvollziehen.

---

## Highlights & Features

- **11 Interaktive Rust-Konzepte:** Jedes Konzept ist mit theoretischen Erklärungen, interaktiven Beispielen in der GUI und kommentiertem Quellcode hinterlegt.
- **Umfassende Widget-Galerie:** Buttons, Checkboxen, Radio-Buttons, ComboBoxen/Dropdowns, Schieberegler, Fortschrittsbalken, Text-Inputs (ein- und mehrzeilig), modale Dialogfenster sowie ein 2D-Vektor-Canvas.
- **Integrierte Zoom-Steuerung:** Stufenlose Skalierung der Benutzeroberfläche von **25 % bis 300 %** in 25%-Schritten – ideal für hochauflösende 4K-Bildschirme.
- **Schnellnavigation im Code:** Einheitliche Such-Tags (`[RUST-KONZEPT:` und `[GUI-ELEMENT:`) ermöglichen das direkte Anspringen relevanter Codeblöcke in jeder IDE.
- **Multithreading & Fearless Concurrency:** Ein Hintergrund-Worker-Thread führt asynchrone Berechnungen aus und kommuniziert über thread-sichere Channels (`mpsc`), ohne die GUI einzufrieren.
- **Standalone EXE-Support:** Kann als eigenständige Windows-Desktop-App ohne Terminalfenster und ohne externe Abhängigkeiten verteilt werden.

---

## Schnellsuche im Quellcode (`src/main.rs`)

Nutzen Sie die globale Suchfunktion Ihrer IDE (`Strg + F` bzw. `Strg + Umschalt + F`), um gezielt zu den Erklärungen zu springen:

| Suchbegriff | Thema |
| :--- | :--- |
| `[RUST-KONZEPT: 01_OWNERSHIP_MOVE]` | Ownership, Heap-Allokation & `.take()` |
| `[RUST-KONZEPT: 02_BORROWING_MUTABILITY]` | Aliasing-Regel: `&T` vs. `&mut T` |
| `[RUST-KONZEPT: 03_LIFETIMES]` | Lebenszeit-Annotationen (`'a`) bei Structs |
| `[RUST-KONZEPT: 04_STRUCTS_ENUMS_MATCH]` | Tagged Unions & Exhaustive Pattern Matching |
| `[RUST-KONZEPT: 05_TRAITS_GENERICS]` | Statischer (`impl Trait`) vs. dynamischer (`dyn Trait`) Dispatch |
| `[RUST-KONZEPT: 06_OPTION_RESULT_ERRORS]` | Typsichere Fehlerbehandlung & der `?`-Operator |
| `[RUST-KONZEPT: 07_SMART_POINTERS_ARC_MUTEX]` | `Box<T>`, `Rc<T>`, `Arc<T>` und `Mutex<T>` |
| `[RUST-KONZEPT: 08_CONCURRENCY_CHANNELS]` | Native Threads & message passing via `mpsc::channel` |
| `[RUST-KONZEPT: 09_CLOSURES_ITERATORS]` | Lazy Iterators, `.filter()`, `.map()`, `.collect()` |
| `[RUST-KONZEPT: 10_MAKROS_DECLARATIVE]` | Metaprogrammierung via `macro_rules!` |
| `[RUST-KONZEPT: 11_UNSAFE_RUST]` | Sichere Kapselung von Rohzeigern (`*const T`) |
| `[GUI-ELEMENT:` | Alle Widgets, Layouts, Panel und Zeichenflächen |

---

## Voraussetzungen

### Für Entwickler (Kompilieren & Quellcode bearbeiten)

- [Rust & Cargo](https://www.rust-lang.org/tools/install) (aktuelle Stable-Version)
- Visual Studio C++ Build Tools (für Windows MSVC)

### Für Endanwender (Ihre Mitarbeiter)

- **Keine Installationen erforderlich!** Die erzeugte `.exe` läuft direkt per Doppelklick ohne Vorbedingungen oder Administratorrechte.

---

## Installation & Entwicklungsstart

1. **Repository klonen oder Projektverzeichnis öffnen:**

   ```powershell
   cd C:\Pfad\zu\Ihrem\Projekt\rust_gui_lab
   ```

2. **Abhängigkeiten in `Cargo.toml` prüfen:**

   ```toml
   [package]
   name = "rust_gui_lab"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   eframe = "0.29"
   ```

3. **Anwendung im Entwicklungsmodus starten:**

   ```powershell
   cargo run
   ```

---

## Standalone `.exe` für Mitarbeiter erstellen

Um eine optimierte, eigenständige Programmdatei ohne begleitendes Konsolenfenster zu kompilieren:

```powershell
cargo build --release
```

Die fertige Datei befindet sich anschließend unter:

```text
target/release/rust_gui_lab.exe
```

> **Hinweis zur Dateigröße & DLL-Unabhängigkeit:**  
> Eine detaillierte Anleitung zur Reduzierung der Dateigröße (auf 5–9 MB via LTO und Strip) sowie zur statischen Einbindung der C-Runtime finden Sie in der Datei [`STANDALONE_ANLEITUNG.md`](STANDALONE_ANLEITUNG.md).

---

## Projektstruktur

```text
rust_gui_lab/
├── .cargo/
│   └── config.toml             # (Optional) Statisches CRT-Linking
├── src/
│   └── main.rs                 # Gesamter Anwendungs- und Lerncode
├── Cargo.toml                  # Projektkonfiguration und Abhängigkeiten
├── README.md                   # Projektdokumentation (diese Datei)
└── README.md     # Leitfaden für Release-Build & Verteilung
```

---

## Verwendete Technologien

- **Programmiersprache:** [Rust](https://www.rust-lang.org/) (Edition 2021)
- **GUI-Framework:** [egui / eframe](https://github.com/emilk/egui) (Immediate-Mode-GUI)
- **Grafik-Backend:** WGPU / Glow (hardwarebeschleunigtes Rendering)
