//! =====================================================================================
//! RUST-DESKTOP-GUI: DAS INTERAKTIVE SPRACH- & WIDGET-LERNLABOR
//! =====================================================================================
//!
//! Diese Anwendung demonstriert die Kernkonzepte der Programmiersprache Rust anhand
//! einer vollständigen Desktop-GUI auf Basis des Frameworks `eframe` / `egui`.
//!
//! SCHNELLSUCHE IN IHRER IDE:
//! - Suchen Sie nach `[RUST-KONZEPT:` um gezielt alle Rust-Konzepte im Code aufzurufen.
//! - Suchen Sie nach `[GUI-ELEMENT:` um alle Desktop-GUI-Elemente zu finden.
//!
//! -------------------------------------------------------------------------------------
//! ANLEITUNG ZUM STARTEN:
//! 1. Erstellen Sie ein neues Projekt: `cargo new rust_gui_lab --bin`
//! 2. Fügen Sie folgende Abhängigkeiten in Ihre `Cargo.toml` ein:
//!
//!    [package]
//!    name = "rust_gui_lab"
//!    version = "0.1.0"
//!    edition = "2021"
//!
//!    [dependencies]
//!    eframe = "0.29"
//!
//! 3. Ersetzen Sie den Inhalt von `src/main.rs` mit diesem Code.
//! 4. Führen Sie `cargo run` aus!
//! =====================================================================================

// Verhindert das schwarze Terminal-Fenster im Release-Modus auf Windows:
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Color32, Pos2, RichText, Stroke, Vec2};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// [RUST-KONZEPT: 10_MAKROS_DECLARATIVE]
// Deklarative Makros (`macro_rules!`) erzeugen zur Compile-Zeit Quellcode (Metaprogrammierung).
// Anders als Funktionen arbeiten Makros auf AST-Token-Ebene, können variable Argumentlisten
// verarbeiten und Typinformationen abstrahieren, ohne Laufzeitkosten zu verursachen.
macro_rules! audit_log {
    ($target:expr, $($arg:tt)*) => {
        $target.push(format!("[AUDIT {}] {}", chrono_dummy_timestamp(), format_args!($($arg)*)))
    };
}

/// Dummy-Zeitstempel-Funktion für Makro-Demonstration
fn chrono_dummy_timestamp() -> &'static str {
    "JETZT"
}

// [RUST-KONZEPT: 05_TRAITS_GENERICS - Trait-Definition]
// Traits definieren gemeinsame Schnittstellen und Fähigkeiten (vergleichbar mit Interfaces,
// aber mächtiger, da sie Default-Implementierungen besitzen und nachträglich für bestehende
// Typen via "Extension Traits" implementiert werden können).
pub trait ComputableMetric {
    fn compute_score(&self) -> f64;
    fn descriptor(&self) -> String;

    // Default-Implementierung einer Trait-Methode:
    fn is_critical(&self) -> bool {
        self.compute_score() > 80.0
    }
}

// [RUST-KONZEPT: 04_STRUCTS_ENUMS_MATCH - Structs]
// Normale Named-Field Structs bündeln zusammengehörige Zustände.
#[derive(Debug, Clone)]
pub struct ServerNode {
    pub name: String,
    pub cpu_load_pct: f64,
    pub memory_gb: f64,
}

// Implementierung des Traits für `ServerNode`
impl ComputableMetric for ServerNode {
    fn compute_score(&self) -> f64 {
        self.cpu_load_pct * 0.7 + (self.memory_gb / 64.0 * 100.0) * 0.3
    }

    fn descriptor(&self) -> String {
        format!("Server '{}' (CPU: {:.1}%, RAM: {:.1} GB)", self.name, self.cpu_load_pct, self.memory_gb)
    }
}

// [RUST-KONZEPT: 05_TRAITS_GENERICS - Statischer vs. Dynamischer Dispatch]
// 1. Statischer Dispatch via Generics und Trait Bounds (`impl Trait` oder `<T: Trait>`):
// Der Compiler führt "Monomorphisierung" durch: Für jeden konkreten Typ wird zur Compile-Zeit
// eine eigene, hochgradig inlinbare Maschinenfunktion erzeugt. Null Overhead zur Laufzeit!
fn format_statically<T: ComputableMetric>(item: &T) -> String {
    format!("STATIC DISPATCH -> {} => Score: {:.1}", item.descriptor(), item.compute_score())
}

// 2. Dynamischer Dispatch via Trait Objects (`&dyn Trait` oder `Box<dyn Trait>`):
// Wenn der Typ erst zur Laufzeit feststeht (z. B. eine heterogene Liste verschiedener Typen).
// Dies nutzt einen Fat Pointer (Datenzeiger + Zeiger auf die vtable).
fn format_dynamically(item: &dyn ComputableMetric) -> String {
    format!("DYN DISPATCH (vtable) -> {} => Kritisch? {}", item.descriptor(), item.is_critical())
}

// [RUST-KONZEPT: 04_STRUCTS_ENUMS_MATCH - Algebraic Data Types (Enums mit Nutzlast)]
// Rust-Enums sind vollwertige Summentypen: Jede Variante kann unterschiedliche Werte kapseln!
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineStatus {
    Uninitialized,
    Pending { queue_position: usize },
    Processing { current_step: String, progress: f32 },
    Success { hash: String, bytes_written: usize },
    Failed { error_code: u16, reason: String },
}

// [RUST-KONZEPT: 03_LIFETIMES - Lebenszeit-Annotationen]
// Rust garantiert Speicherintegrität ohne Garbage Collector über statische Lebenszeiten.
// Wenn eine Datenstruktur eine Referenz hält, MUSS der Entwickler mit einer Lebenszeit `'a`
// deklarieren, dass die Referenz niemals länger leben darf als das referenzierte Original!
pub struct TextSnippetView<'a> {
    pub header: &'a str,
    pub body: &'a str,
}

impl<'a> TextSnippetView<'a> {
    pub fn format_preview(&self) -> String {
        format!("{}: {}", self.header, self.body)
    }
}

// [RUST-KONZEPT: 08_CONCURRENCY_CHANNELS - Thread-Sichere Kommunikation]
// Rust verbietet Data Races zur Compile-Zeit durch das `Send`- und `Sync`-Trait.
// Hier definieren wir Nachrichten, die über einen `std::sync::mpsc::channel`
// vom Hintergrundthread an die GUI geschickt werden.
pub enum BackgroundJobMessage {
    StepProgress(f32, String),
    Finished(String),
}

// [GUI-ELEMENT: WINDOW_APP_SHELL]
// Haupt-Zustandsstruktur unserer Desktop-Anwendung.
// Hält den gesamten UI- und Verarbeitungszustand im Speicher.
pub struct RustMasterApp {
    // Navigations-Zustand (Tabs)
    selected_tab: TabPage,

    // [GUI-ELEMENT: ZOOM_CONTROLS - Desktop-Skalierung von 25% bis 200%]
    zoom_percent: i32,

    // [RUST-KONZEPT: 01_OWNERSHIP_MOVE - Interaktives Anschauungsbeispiel]
    owned_token: Option<String>,
    token_history: Vec<String>,

    // [RUST-KONZEPT: 02_BORROWING_MUTABILITY]
    counter_value: i32,
    borrow_log: Vec<String>,

    // [RUST-KONZEPT: 04_STRUCTS_ENUMS_MATCH]
    pipeline_state: PipelineStatus,

    // [RUST-KONZEPT: 06_OPTION_RESULT_ERRORS]
    input_dividend: String,
    input_divisor: String,
    calculation_result: String,

    // [RUST-KONZEPT: 08_CONCURRENCY_CHANNELS & 07_SMART_POINTERS_ARC_MUTEX]
    job_receiver: Receiver<BackgroundJobMessage>,
    job_sender: Sender<BackgroundJobMessage>,
    is_job_running: Arc<Mutex<bool>>,
    worker_progress: f32,
    worker_status_text: String,

    // [RUST-KONZEPT: 09_CLOSURES_ITERATORS]
    raw_numbers: Vec<i32>,
    filter_even_only: bool,
    multiplier: i32,

    // [GUI-ELEMENT: INTERACTIVE_WIDGET_STATES]
    demo_text_single: String,
    demo_text_multi: String,
    demo_checkbox: bool,
    demo_slider_val: f32,
    demo_selected_option: String,
    demo_color: Color32,
    demo_show_modal: bool,
    demo_toggle_switch: bool,

    // Log-Ausgabe für das UI-Audit
    system_logs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabPage {
    Overview,
    OwnershipAndBorrowing,
    EnumsAndMatching,
    TraitsAndGenerics,
    OptionResultAndErrors,
    ConcurrencyAndSmartPointers,
    IteratorsAndClosures,
    LifetimesAndUnsafe,
    GuiElementGallery,
}

impl Default for RustMasterApp {
    fn default() -> Self {
        let (tx, rx) = channel();
        let mut initial_logs = Vec::new();
        audit_log!(initial_logs, "Desktop-Anwendung erfolgreich initialisiert.");

        Self {
            selected_tab: TabPage::Overview,
            zoom_percent: 100,

            owned_token: Some(String::from("Geheimes_Besitz_Token_#42")),
            token_history: vec![String::from("Ursprünglicher Speicherbesitzer: `main_app`")],

            counter_value: 10,
            borrow_log: vec![String::from("Initialwert: 10")],

            pipeline_state: PipelineStatus::Uninitialized,

            input_dividend: "100".to_string(),
            input_divisor: "4".to_string(),
            calculation_result: "Noch keine Berechnung".to_string(),

            job_receiver: rx,
            job_sender: tx,
            is_job_running: Arc::new(Mutex::new(false)),
            worker_progress: 0.0,
            worker_status_text: "Bereit für Hintergrundaufgaben".to_string(),

            raw_numbers: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20],
            filter_even_only: false,
            multiplier: 2,

            demo_text_single: "Eingabefeld Text".to_string(),
            demo_text_multi: "Hier ist ein mehrzeiliger Textbereich.\nRust garantiert Speichersicherheit!\nKeine Nullzeiger, keine Data Races.".to_string(),
            demo_checkbox: true,
            demo_slider_val: 45.0,
            demo_selected_option: "Rust Edition 2021".to_string(),
            demo_color: Color32::from_rgb(46, 125, 246),
            demo_show_modal: false,
            demo_toggle_switch: true,

            system_logs: initial_logs,
        }
    }
}

impl RustMasterApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // [GUI-ELEMENT: STYLING_AND_FONTS]
        // Hier können Themes, Schriftarten und Visuals konfiguriert werden.
        Self::default()
    }

    /// [RUST-KONZEPT: 06_OPTION_RESULT_ERRORS - Robuste Fehlerbehandlung mit Result]
    /// Diese Hilfsfunktion demonstriert die Kombination von `Result`, Parsing und dem `?`-Operator.
    fn calculate_division_safe(div_a: &str, div_b: &str) -> Result<f64, String> {
        let a: f64 = div_a
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Ungültiger Dividend '{}': {}", div_a, e))?;

        let b: f64 = div_b
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Ungültiger Divisor '{}': {}", div_b, e))?;

        if b.abs() < f64::EPSILON {
            return Err("Division durch 0 ist mathematisch nicht definiert!".to_string());
        }

        Ok(a / b)
    }

    /// [RUST-KONZEPT: 08_CONCURRENCY_CHANNELS - Starten eines Worker-Threads]
    fn spawn_background_task(&mut self) {
        let mut is_running = self.is_job_running.lock().unwrap();
        if *is_running {
            return;
        }
        *is_running = true;
        self.worker_progress = 0.0;
        self.worker_status_text = "Hintergrund-Thread rechnet...".to_string();

        let tx = self.job_sender.clone();
        let running_lock = Arc::clone(&self.is_job_running);

        // [RUST-KONZEPT: 08_CONCURRENCY_CHANNELS - thread::spawn mit move-Closure]
        // Das Schlüsselwort `move` überträgt den Besitz von `tx` und `running_lock`
        // direkt in den neu erstellten Betriebssystem-Thread.
        thread::spawn(move || {
            for step in 1..=5 {
                thread::sleep(Duration::from_millis(350));
                let progress = step as f32 / 5.0;
                let text = format!("Schritt {} von 5 abgeschlossen...", step);
                let _ = tx.send(BackgroundJobMessage::StepProgress(progress, text));
            }

            thread::sleep(Duration::from_millis(200));
            let _ = tx.send(BackgroundJobMessage::Finished(
                "Hintergrundberechnung erfolgreich abgeschlossen! (Data-Race frei)".to_string(),
            ));

            // Mutex freigeben:
            let mut flag = running_lock.lock().unwrap();
            *flag = false;
        });
    }
}

impl eframe::App for RustMasterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // [GUI-ELEMENT: ZOOM_CONTROLS - Globale UI-Skalierung anwenden]
        // Stellt die Pixels-per-Point Skalierung stufenlos auf 25% bis 200% ein.
        ctx.set_zoom_factor(self.zoom_percent as f32 / 100.0);

        // [RUST-KONZEPT: 08_CONCURRENCY_CHANNELS - Empfang von Thread-Nachrichten ohne UI-Freeze]
        // `try_recv()` ist non-blocking! Verhindert, dass der UI-Rendering-Thread blockiert.
        while let Ok(msg) = self.job_receiver.try_recv() {
            match msg {
                BackgroundJobMessage::StepProgress(pct, status) => {
                    self.worker_progress = pct;
                    self.worker_status_text = status;
                }
                BackgroundJobMessage::Finished(summary) => {
                    self.worker_progress = 1.0;
                    self.worker_status_text = summary.clone();
                    audit_log!(self.system_logs, "Worker-Thread beendet: {}", summary);
                }
            }
        }

        // [GUI-ELEMENT: TOP_MENU_STATUS_BAR - Obere Leiste]
        egui::TopBottomPanel::top("header_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Rust Desktop GUI- & Sprach-Lernlabor").strong().color(Color32::from_rgb(230, 90, 40)));
                ui.separator();
                ui.label(RichText::new("Interaktive Exploration von Sprachkonzepten & Desktop-Widgets").italics());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Info / Über").clicked() {
                        self.demo_show_modal = true;
                    }
                    ui.separator();

                    // [GUI-ELEMENT: ZOOM_CONTROLS - Hereinzoomen (max. 200%)]
                    let can_zoom_in = self.zoom_percent < 200;
                    if ui.add_enabled(can_zoom_in, egui::Button::new(" + "))
                        .on_hover_text("Hereinzoomen (+25%, bis max. 200%)")
                        .clicked()
                    {
                        self.zoom_percent = (self.zoom_percent + 25).min(200);
                        audit_log!(self.system_logs, "Zoom vergrößert auf {}%", self.zoom_percent);
                    }

                    // [GUI-ELEMENT: ZOOM_CONTROLS - Zoom zurücksetzen (100%)]
                    if ui.button("100%")
                        .on_hover_text("Zoom auf Standardgröße (100%) zurücksetzen")
                        .clicked()
                    {
                        self.zoom_percent = 100;
                        audit_log!(self.system_logs, "Zoom zurückgesetzt auf 100%");
                    }

                    // [GUI-ELEMENT: ZOOM_CONTROLS - Herauszoomen (min. 25%)]
                    let can_zoom_out = self.zoom_percent > 25;
                    if ui.add_enabled(can_zoom_out, egui::Button::new(" - "))
                        .on_hover_text("Herauszoomen (-25%, bis min. 25%)")
                        .clicked()
                    {
                        self.zoom_percent = (self.zoom_percent - 25).max(25);
                        audit_log!(self.system_logs, "Zoom verkleinert auf {}%", self.zoom_percent);
                    }

                    // Anzeige der aktuellen Zoomstufe
                    ui.monospace(RichText::new(format!("{}%", self.zoom_percent)).strong());
                    ui.label(RichText::new("Zoom:").small());
                    ui.separator();

                    ui.label(RichText::new("eframe 0.29 | Safe Rust").small());
                });
            });
            ui.add_space(4.0);
        });

        // [GUI-ELEMENT: TOP_MENU_STATUS_BAR - Untere Statusleiste]
        egui::TopBottomPanel::bottom("footer_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Status:").strong());
                let running = *self.is_job_running.lock().unwrap();
                if running {
                    ui.spinner();
                    ui.colored_label(Color32::from_rgb(250, 180, 0), "Hintergrund-Thread arbeitet aktiv");
                } else {
                    ui.colored_label(Color32::from_rgb(60, 200, 120), "Bereit (UI läuft ruckelfrei)");
                }

                ui.separator();
                ui.label(format!("Letzter Log-Eintrag: {}", self.system_logs.last().cloned().unwrap_or_default()));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Drücken Sie Strg+F innerhalb der Codedatei und suchen Sie nach `[RUST-KONZEPT:` oder `[GUI-ELEMENT:`");
                });
            });
        });

        // [GUI-ELEMENT: SCROLL_AREA - Linke Seitennavigation für Tabs]
        egui::SidePanel::left("navigation_sidebar")
            .resizable(false)
            .default_width(220.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(RichText::new("KAPITEL-NAVIGATION").strong().color(Color32::GRAY));
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let nav_items = [
                        (TabPage::Overview, "01. Übersicht & Guide"),
                        (TabPage::OwnershipAndBorrowing, "02. Ownership & Borrowing"),
                        (TabPage::EnumsAndMatching, "03. Enums, Structs & Match"),
                        (TabPage::TraitsAndGenerics, "04. Traits & Generics"),
                        (TabPage::OptionResultAndErrors, "05. Option, Result & Fehler"),
                        (TabPage::ConcurrencyAndSmartPointers, "06. Concurrency & Pointer"),
                        (TabPage::IteratorsAndClosures, "07. Iteratoren & Closures"),
                        (TabPage::LifetimesAndUnsafe, "08. Lifetimes & Unsafe"),
                        (TabPage::GuiElementGallery, "09. Alle UI-Elemente & Canvas"),
                    ];

                    for (page, label) in nav_items {
                        // [GUI-ELEMENT: BUTTONS - Navigation Toggle Buttons]
                        let is_selected = self.selected_tab == page;
                        if ui.selectable_label(is_selected, RichText::new(label).size(13.0)).clicked() {
                            self.selected_tab = page;
                        }
                        ui.add_space(2.0);
                    }
                });
            });

        // [GUI-ELEMENT: CENTRAL_PANEL - Hauptinhalt dynamisch je nach Tab]
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(6.0);
                match self.selected_tab {
                    TabPage::Overview => self.render_tab_overview(ui),
                    TabPage::OwnershipAndBorrowing => self.render_tab_ownership(ui),
                    TabPage::EnumsAndMatching => self.render_tab_enums(ui),
                    TabPage::TraitsAndGenerics => self.render_tab_traits(ui),
                    TabPage::OptionResultAndErrors => self.render_tab_errors(ui),
                    TabPage::ConcurrencyAndSmartPointers => self.render_tab_concurrency(ui),
                    TabPage::IteratorsAndClosures => self.render_tab_iterators(ui),
                    TabPage::LifetimesAndUnsafe => self.render_tab_lifetimes_unsafe(ui),
                    TabPage::GuiElementGallery => self.render_tab_gui_gallery(ui),
                }
                ui.add_space(20.0);
            });
        });

        // [GUI-ELEMENT: MODAL_WINDOW_DIALOG - Modaler Pop-Up-Dialog]
        if self.demo_show_modal {
            let mut is_open = true;
            let mut close_requested = false;

            egui::Window::new("Rust Sprach- & Architektur-Übersicht")
                .open(&mut is_open)
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_width(450.0);
                    ui.heading("Warum Rust für Desktop-Apps?");
                    ui.add_space(6.0);
                    ui.label("• Keine Garbage-Collection-Pausen (reproduzierbare Frame-Rates)");
                    ui.label("• Vollständige Typsicherheit und Thread-Sicherheit zur Compile-Zeit");
                    ui.label("• Zero-Cost Abstractions: Komfort wie in Hochsprachen, Geschwindigkeit wie C/C++");
                    ui.label("• Immediate-Mode-GUI (egui): UI ist eine direkte Funktion des aktuellen Status!");
                    ui.add_space(10.0);
                    if ui.button("Verstanden & Schließen").clicked() {
                        close_requested = true;
                    }
                });

            self.demo_show_modal = is_open && !close_requested;
        }
    }
}

impl RustMasterApp {
    fn render_tab_overview(&mut self, ui: &mut egui::Ui) {
        ui.heading("Willkommen im Rust Sprach- & GUI-Lernlabor!");
        ui.label(
            "Dieses interaktive Labor verbindet alle zentralen theoretischen Säulen von Rust \
            mit handfester praktischer GUI-Entwicklung. Jedes Konzept kann direkt im Interface \
            ausprobiert und über Such-Tags im Quellcode nachvollzogen werden.",
        );
        ui.add_space(12.0);

        // [GUI-ELEMENT: COLLAPSING_HEADER_ACCORDION]
        ui.collapsing("So navigieren Sie durch den Quellcode", |ui| {
            ui.label(RichText::new("Suchen Sie global in Ihrer IDE nach folgendem Präfix:").strong());
            ui.monospace("[RUST-KONZEPT:  -> Springt zu allen 11 Sprachkonzepten");
            ui.monospace("[GUI-ELEMENT:   -> Springt zu allen verwendeten Desktop-Widgets");
            ui.label("Jede Sektion enthält ausführliche Dokumentationen zu Theorie, Mechanik und Speicherorganisation.");
        });

        ui.add_space(12.0);
        // [GUI-ELEMENT: LABELS_HEADINGS_MARKDOWN - Hervorgehobene Info-Box]
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading("Die drei Grundpfeiler von Rust:");
            ui.horizontal(|ui| {
                ui.label(RichText::new("1. Speichersicherheit:").strong().color(Color32::LIGHT_BLUE));
                ui.label("Kein freier Zugriff auf ungültigen Speicher, keine Nullzeiger, kein Use-After-Free.");
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("2. Furchtlose Nebenläufigkeit:").strong().color(Color32::LIGHT_GREEN));
                ui.label("Data Races werden durch das Typsystem (Send/Sync) zur Compile-Zeit ausgeschlossen.");
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("3. Zero-Cost Abstractions:").strong().color(Color32::LIGHT_YELLOW));
                ui.label("Generics, Iteratoren und Pattern Matching kompilieren zu extrem schnellem Maschinencode.");
            });
        });

        ui.add_space(12.0);
        ui.label("Wählen Sie links ein Kapitel aus, um Konzepte interaktiv zu testen.");
    }

    fn render_tab_ownership(&mut self, ui: &mut egui::Ui) {
        ui.heading("1. Ownership & 2. Borrowing (Besitz & Ausleihe)");
        ui.label(
            "Rust verzichtet auf einen Garbage Collector. Speicher wird statisch über das \
            Ownership-Modell verwaltet. Jeder Wert hat genau EINEN Besitzer (Owner). Wenn der \
            Besitzer den Gültigkeitsbereich (Scope) verlässt, wird der Speicher automatisch freigegeben (RAII).",
        );
        ui.add_space(10.0);

        // [RUST-KONZEPT: 01_OWNERSHIP_MOVE - Interaktive Demonstration]
        ui.group(|ui| {
            ui.heading("Interaktive Ownership-Demonstration (Move-Semantik)");
            ui.label(
                "Wenn ein Typ nicht das `Copy`-Trait implementiert (wie `String` oder `Vec`), \
                bewirkt eine Zuweisung einen 'Move'. Das ursprüngliche Binding ist danach ungültig.",
            );
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Aktueller Wert in `self.owned_token`:");
                match &self.owned_token {
                    Some(val) => {
                        ui.colored_label(Color32::GREEN, format!("VORHANDEN ('{}')", val));
                    }
                    None => {
                        ui.colored_label(Color32::RED, "VERLOREN / GELEERT (Wurde verschoben!)");
                    }
                }
            });

            ui.horizontal(|ui| {
                // [GUI-ELEMENT: BUTTONS]
                if ui.button("Ownership transferieren (Move via .take())").clicked() {
                    // [RUST-KONZEPT: 01_OWNERSHIP_MOVE - Option::take() entzieht den Besitz]
                    if let Some(token) = self.owned_token.take() {
                        let transferred_msg = format!("Besitz übertragen auf lokalen Scope: '{}'", token);
                        self.token_history.push(transferred_msg.clone());
                        audit_log!(self.system_logs, "{}", transferred_msg);
                    }
                }

                if ui.button("Neuen Besitz instanziieren (Heap-Allokation)").clicked() {
                    let new_token = format!("Neuer_Besitz_Wert_#{}", self.token_history.len() + 1);
                    self.owned_token = Some(new_token);
                    self.token_history.push("Neuer Wert im Heap allokiert und Besitzer zugewiesen.".to_string());
                }
            });

            ui.collapsing("Protokoll der Besitzwechsel", |ui| {
                for entry in &self.token_history {
                    ui.label(format!("• {}", entry));
                }
            });
        });

        ui.add_space(14.0);

        // [RUST-KONZEPT: 02_BORROWING_MUTABILITY - Ausleihe und Aliasing-Regel]
        ui.group(|ui| {
            ui.heading("Borrowing: Referenzen & Mutability (& vs &mut)");
            ui.label(
                "Die goldene Aliasing-Regel von Rust lautet: \
                Zu jedem Zeitpunkt darf es ENTWEDER beliebig viele unveränderliche Referenzen (&T) \
                ODER genau EINE veränderliche Referenz (&mut T) geben – niemals beides gleichzeitig!",
            );
            ui.add_space(6.0);

            ui.label(format!("Aktueller Zählerstand: {}", self.counter_value));

            ui.horizontal(|ui| {
                if ui.button("&mut self.counter_value mutieren (+5)").clicked() {
                    // [RUST-KONZEPT: 02_BORROWING_MUTABILITY - Exklusive veränderliche Referenz]
                    fn mutate_counter(val_ref: &mut i32) {
                        *val_ref += 5; // Dereferenzierung und Mutation
                    }
                    mutate_counter(&mut self.counter_value);
                    self.borrow_log.push(format!("Mutiert via &mut: Neuer Wert = {}", self.counter_value));
                }

                if ui.button("&self.counter_value unveränderlich lesen").clicked() {
                    // [RUST-KONZEPT: 02_BORROWING_MUTABILITY - Geteilte Lese-Referenz]
                    fn read_counter(val_ref: &i32) -> String {
                        format!("Gelesen via &i32: Wert beträgt {}", *val_ref)
                    }
                    let res = read_counter(&self.counter_value);
                    self.borrow_log.push(res);
                }
            });

            ui.collapsing("Borrow-Logbuch", |ui| {
                for item in self.borrow_log.iter().rev().take(5) {
                    ui.label(format!("• {}", item));
                }
            });
        });
    }

    fn render_tab_enums(&mut self, ui: &mut egui::Ui) {
        ui.heading("4. Enums, Structs & Pattern Matching");
        ui.label(
            "Rust-Enums sind mächtige algebraische Datentypen (Tagged Unions). \
            Das `match`-Konstrukt erzwingt Vollständigkeit (Exhaustiveness): Jeder mögliche \
            Zustand MUSS behandelt werden, wodurch unhandled edge-cases unmöglich werden.",
        );
        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("Zustandsautomat mit Enums steuern:");
            ui.horizontal(|ui| {
                ui.label("Status wechseln zu:");
                if ui.button("Uninitialized").clicked() {
                    self.pipeline_state = PipelineStatus::Uninitialized;
                }
                if ui.button("Pending (Warteschlange)").clicked() {
                    self.pipeline_state = PipelineStatus::Pending { queue_position: 3 };
                }
                if ui.button("Processing (Berechnung)").clicked() {
                    self.pipeline_state = PipelineStatus::Processing {
                        current_step: "Kompiliere Bytecode".to_string(),
                        progress: 0.65,
                    };
                }
                if ui.button("Success").clicked() {
                    self.pipeline_state = PipelineStatus::Success {
                        hash: "sha256:e3b0c44298fc1c149afbf4c8996fb924".to_string(),
                        bytes_written: 4096,
                    };
                }
                if ui.button("Failed").clicked() {
                    self.pipeline_state = PipelineStatus::Failed {
                        error_code: 503,
                        reason: "Service vorübergehend nicht erreichbar".to_string(),
                    };
                }
            });

            ui.separator();
            ui.label(RichText::new("Pattern Matching Rendering (`match &self.pipeline_state`):").strong());

            // [RUST-KONZEPT: 04_STRUCTS_ENUMS_MATCH - Exhaustive Match Demonstration]
            match &self.pipeline_state {
                PipelineStatus::Uninitialized => {
                    ui.colored_label(Color32::GRAY, "• Pipeline ist nicht initialisiert.");
                }
                PipelineStatus::Pending { queue_position } => {
                    ui.colored_label(Color32::LIGHT_YELLOW, format!("• In Warteschlange an Position {}", queue_position));
                }
                PipelineStatus::Processing { current_step, progress } => {
                    ui.colored_label(Color32::LIGHT_BLUE, format!("• Verarbeite: {} ({:.0}%)", current_step, progress * 100.0));
                    // [GUI-ELEMENT: SLIDER_PROGRESSBAR]
                    ui.add(egui::ProgressBar::new(*progress).show_percentage());
                }
                PipelineStatus::Success { hash, bytes_written } => {
                    ui.colored_label(Color32::GREEN, format!("• Erfolgreich! {} Bytes geschrieben. Prüfsumme: {}", bytes_written, hash));
                }
                PipelineStatus::Failed { error_code, reason } => {
                    ui.colored_label(Color32::RED, format!("• Fehler [Code {}]: {}", error_code, reason));
                }
            }
        });
    }

    fn render_tab_traits(&mut self, ui: &mut egui::Ui) {
        ui.heading("5. Traits, Generics & Polymorphie");
        ui.label(
            "Rust implementiert Polymorphie über Traits anstelle von klassischer Klassen-Vererbung. \
            Man unterscheidet zwischen statischem Dispatch (`impl Trait`, zur Compile-Zeit monomorphisiert) \
            und dynamischem Dispatch (`dyn Trait`, zur Laufzeit über vtables aufgelöst).",
        );
        ui.add_space(10.0);

        // Instanziierung von Demonstrations-Objekten
        let server_a = ServerNode {
            name: "Alpha-Web-01".to_string(),
            cpu_load_pct: 42.5,
            memory_gb: 32.0,
        };
        let server_b = ServerNode {
            name: "Compute-Cluster-99".to_string(),
            cpu_load_pct: 94.2,
            memory_gb: 64.0,
        };

        ui.group(|ui| {
            ui.heading("1. Statischer Dispatch via Generics (`<T: ComputableMetric>`)");
            ui.label("Der Compiler generiert für jeden Typen spezialisierten Maschinencode. Maximale Performance!");
            ui.label(RichText::new(format_statically(&server_a)).monospace());
            ui.label(RichText::new(format_statically(&server_b)).monospace());
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("2. Dynamischer Dispatch via Trait Objects (`&dyn ComputableMetric`)");
            ui.label("Ermöglicht heterogene Sammlungen zur Laufzeit über Fat Pointer:");

            // [RUST-KONZEPT: 07_SMART_POINTERS_ARC_MUTEX - Box<dyn Trait>]
            let heterogeneous_list: Vec<Box<dyn ComputableMetric>> = vec![
                Box::new(server_a),
                Box::new(server_b),
            ];

            for (idx, obj) in heterogeneous_list.iter().enumerate() {
                ui.label(RichText::new(format!("{}: {}", idx + 1, format_dynamically(obj.as_ref()))).monospace());
            }
        });
    }

    fn render_tab_errors(&mut self, ui: &mut egui::Ui) {
        ui.heading("6. Option<T>, Result<T, E> & Der ?-Operator");
        ui.label(
            "In Rust gibt es weder Nullpointer noch ungeprüfte Exceptions (Exceptions). \
            Das Fehlen eines Wertes wird typisiert über `Option<T>` (`Some(T)` oder `None`) abgebildet. \
            Fehleranfällige Operationen geben `Result<T, E>` (`Ok(T)` oder `Err(E)`) zurück.",
        );
        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("Interaktiver fehlersicherer Rechner:");
            ui.label("Geben Sie Werte ein. Ungültige Eingaben (Buchstaben oder Division durch 0) führen zu keinem Absturz!");

            ui.horizontal(|ui| {
                ui.label("Dividend:");
                // [GUI-ELEMENT: TEXT_INPUT_SINGLE_MULTI - Einzeiliges Textfeld]
                ui.text_edit_singleline(&mut self.input_dividend);

                ui.label("Divisor:");
                ui.text_edit_singleline(&mut self.input_divisor);
            });

            if ui.button("Berechnen via Result & ?-Operator").clicked() {
                // Ausführung der sicheren Funktion
                match Self::calculate_division_safe(&self.input_dividend, &self.input_divisor) {
                    Ok(result) => {
                        self.calculation_result = format!("Ergebnis: {:.4}", result);
                        audit_log!(self.system_logs, "Rechnung erfolgreich: {}", self.calculation_result);
                    }
                    Err(err_msg) => {
                        self.calculation_result = format!("Fehler abgefangen: {}", err_msg);
                        audit_log!(self.system_logs, "Rechnungsfehler: {}", err_msg);
                    }
                }
            }

            ui.separator();
            ui.label(RichText::new(&self.calculation_result).strong().color(Color32::LIGHT_YELLOW));
        });

        ui.add_space(10.0);
        ui.collapsing("Wie der ?-Operator funktioniert", |ui| {
            ui.label("Der `?`-Operator prüft das Ergebnis eines `Result`:");
            ui.label("• Bei `Ok(val)`: Entpackt den Wert und führt die Funktion weiter.");
            ui.label("• Bei `Err(e)`: Beendet die aktuelle Funktion vorzeitig und gibt den Fehler zurück (`early return`).");
        });
    }

    fn render_tab_concurrency(&mut self, ui: &mut egui::Ui) {
        ui.heading("7. Smart Pointer & 8. Concurrency (Arc, Mutex, Box)");
        ui.label(
            "Rust garantiert 'Fearless Concurrency': Wenn Ihr Code kompiliert, \
            haben Sie garantiert keine Data Races. Für geteilten, veränderlichen Zustand \
            über Thread-Grenzen hinweg nutzt man `Arc<Mutex<T>>` (Atomic Reference Counted Mutex).",
        );
        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("Hintergrund-Worker (Thread & Channel Kommunikation)");
            ui.label(
                "Dieser Task läuft in einem separaten nativen Betriebssystem-Thread. \
                Die Desktop-GUI bleibt währenddessen 100% flüssig und ansprechbar!",
            );
            ui.add_space(6.0);

            let is_running = *self.is_job_running.lock().unwrap();

            ui.horizontal(|ui| {
                if is_running {
                    ui.add_enabled(false, egui::Button::new("Arbeit läuft bereits..."));
                } else if ui.button("Hintergrund-Thread starten").clicked() {
                    self.spawn_background_task();
                }

                ui.label(format!("Status: {}", self.worker_status_text));
            });

            ui.add_space(6.0);
            // [GUI-ELEMENT: SLIDER_PROGRESSBAR - Fortschrittsanzeige]
            ui.add(egui::ProgressBar::new(self.worker_progress).show_percentage());
        });

        ui.add_space(10.0);
        ui.group(|ui| {
            ui.heading("Smart Pointer Übersicht:");
            ui.label("• `Box<T>`: Allokiert Daten auf dem Heap mit exklusivem Besitz (z.B. für rekursive Strukturen).");
            ui.label("• `Rc<T>`: Referenzzähler für geteilten Besitz innerhalb EINES Threads (nicht thread-safe).");
            ui.label("• `Arc<T>`: Atomarer Referenzzähler für geteilten Besitz ÜBER THREADS HINWEG.");
            ui.label("• `Mutex<T>`: Garantiert gegenseitigen Ausschluss bei Datenmutationen zur Laufzeit.");
        });
    }

    fn render_tab_iterators(&mut self, ui: &mut egui::Ui) {
        ui.heading("9. Closures & Iteratoren");
        ui.label(
            "Iteratoren in Rust sind 'lazy' (träge): Sie berechnen erst dann Werte, \
            wenn sie konsumiert werden (z. B. via `.collect()` oder `.fold()`). \
            Der Compiler optimiert Iterator-Pipelines oft zu schnelleren Schleifen als handgeschriebenes C.",
        );
        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("Interaktive funktionale Daten-Pipeline:");

            ui.horizontal(|ui| {
                // [GUI-ELEMENT: CHECKBOX_RADIO - Checkbox]
                ui.checkbox(&mut self.filter_even_only, "Nur gerade Zahlen filtern (.filter(|x| x % 2 == 0))");
            });

            ui.horizontal(|ui| {
                ui.label("Multiplikator (.map(|x| x * n)):");
                // [GUI-ELEMENT: SLIDER_PROGRESSBAR - Slider]
                ui.add(egui::Slider::new(&mut self.multiplier, 1..=10));
            });

            ui.separator();

            // [RUST-KONZEPT: 09_CLOSURES_ITERATORS - Pipeline-Ausführung mit Closures]
            let pipeline_result: Vec<i32> = self
                .raw_numbers
                .iter()
                .filter(|&&x| !self.filter_even_only || x % 2 == 0) // Closure mit Umgebung
                .map(|&x| x * self.multiplier)                     // Closure mit Mutator
                .collect();

            let sum: i32 = pipeline_result.iter().sum();

            ui.label(format!("Ursprüngliche Liste: {:?}", self.raw_numbers));
            ui.label(RichText::new(format!("Ergebnis nach Transformation: {:?}", pipeline_result)).strong().color(Color32::LIGHT_GREEN));
            ui.label(RichText::new(format!("Summe via .sum(): {}", sum)).italics());
        });
    }

    fn render_tab_lifetimes_unsafe(&mut self, ui: &mut egui::Ui) {
        ui.heading("3. Lifetimes & 11. Unsafe Rust");
        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("Explizite Lebenszeiten (`'a`)");
            ui.label(
                "Lebenszeit-Parameter ändern nicht, wie lange ein Wert existiert. \
                Sie sind Verträge für den Borrow-Checker, um zu beweisen, dass keine Referenz \
                länger lebt als das Original (Verhinderung von Dangling References).",
            );

            let static_text = "Dieser String lebt im statischen Speicher";
            let view = TextSnippetView {
                header: "METADATEN",
                body: static_text,
            };
            ui.label(format!("Instanziiertes Struct mit Lebenszeit: '{}'", view.format_preview()));
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("Unsafe Rust: Die kontrollierte Ausnahme");
            ui.label(
                "`unsafe` schaltet weder den Type-Checker aus noch deaktiviert es den Borrow-Checker. \
                Es schaltet lediglich 5 'Superkräfte' frei (z.B. Rohzeiger dereferenzieren, FFI aufrufen). \
                In idiomatischem Rust kapselt man unsicheren Code immer hinter einer 100% sicheren Abstraktion!",
            );
            ui.add_space(6.0);

            // [RUST-KONZEPT: 11_UNSAFE_RUST - Sichere Kapselung eines Rohzeigers]
            let val: u64 = 0xDEADBEEF;
            let val_ptr: *const u64 = &val as *const u64;

            let inspected_val = unsafe {
                // Das Dereferenzieren eines Rohzeigers benötigt zwingend einen unsafe-Block.
                // Wir als Entwickler garantieren hier manuell, dass `val_ptr` gültig und nicht null ist.
                *val_ptr
            };

            ui.label(format!("Wert sicher inspiziert über Rohzeiger *const u64: 0x{:X}", inspected_val));
        });
    }

    fn render_tab_gui_gallery(&mut self, ui: &mut egui::Ui) {
        ui.heading("Galerie aller gängigen UI-Elemente");
        ui.label("Hier finden Sie alle klassischen Desktop-GUI-Elemente interaktiv eingebunden.");
        ui.add_space(10.0);

        ui.columns(2, |columns| {
            // SPALTE 1: Formularelemente
            columns[0].group(|ui| {
                ui.heading("Eingaben & Selektoren");

                // [GUI-ELEMENT: TEXT_INPUT_SINGLE_MULTI]
                ui.label("Einzeilige Texteingabe:");
                ui.text_edit_singleline(&mut self.demo_text_single);

                ui.label("Mehrzeilige Texteingabe:");
                ui.text_edit_multiline(&mut self.demo_text_multi);

                ui.separator();
                // [GUI-ELEMENT: CHECKBOX_RADIO - Radio Buttons]
                ui.label("Radio-Button Auswahl:");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.demo_selected_option, "Rust Edition 2018".to_string(), "2018");
                    ui.radio_value(&mut self.demo_selected_option, "Rust Edition 2021".to_string(), "2021");
                    ui.radio_value(&mut self.demo_selected_option, "Rust Edition 2024".to_string(), "2024");
                });

                ui.separator();
                // [GUI-ELEMENT: COMBOBOX_DROPDOWN - Dropdown / Auswahlmenü]
                ui.label("ComboBox / Dropdown Menü:");
                egui::ComboBox::from_label("Aktive Edition")
                    .selected_text(&self.demo_selected_option)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.demo_selected_option, "Rust Edition 2018".to_string(), "Rust Edition 2018");
                        ui.selectable_value(&mut self.demo_selected_option, "Rust Edition 2021".to_string(), "Rust Edition 2021");
                        ui.selectable_value(&mut self.demo_selected_option, "Rust Edition 2024".to_string(), "Rust Edition 2024");
                    });

                ui.separator();
                // [GUI-ELEMENT: CHECKBOX_TOGGLE - Checkbox & Kippschalter]
                ui.label("Checkbox & Schalter:");
                let checkbox_label = if self.demo_checkbox {
                    "Checkbox (Status: Aktiviert)".to_owned()
                } else {
                    "Checkbox (Status: Deaktiviert)".to_owned()
                };
                ui.checkbox(&mut self.demo_checkbox, checkbox_label);
                let toggle_label = if self.demo_toggle_switch {
                    "Kippschalter: AN".to_owned()
                } else {
                    "Kippschalter: AUS".to_owned()
                };
                ui.toggle_value(&mut self.demo_toggle_switch, toggle_label);

                ui.separator();
                // [GUI-ELEMENT: COLOR_PICKER]
                ui.horizontal(|ui| {
                    ui.label("Color Picker (Farbwähler):");
                    ui.color_edit_button_srgba(&mut self.demo_color);
                });
            });

            // SPALTE 2: Feedback & 2D Canvas
            columns[1].group(|ui| {
                ui.heading("Visuelles Feedback & 2D Canvas");

                // [GUI-ELEMENT: SLIDER_PROGRESSBAR]
                ui.label("Interaktiver Schieberegler:");
                ui.add(egui::Slider::new(&mut self.demo_slider_val, 0.0..=100.0).text("Intensität"));

                ui.label("Progress Bar:");
                ui.add(egui::ProgressBar::new(self.demo_slider_val / 100.0).show_percentage());

                ui.separator();

                // [GUI-ELEMENT: PAINTER_CANVAS - Benutzerdefinierte 2D-Zeichenfläche]
                ui.label("2D Zeichenfläche (Custom Painter Canvas):");
                let canvas_size = Vec2::new(ui.available_width(), 160.0);
                let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::hover());

                let rect = response.rect;
                // Hintergrund zeichnen
                painter.rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 35));
                painter.rect_stroke(rect, 4.0_f32, Stroke::new(1.0_f32, Color32::DARK_GRAY));

                // Dynamische Kreise und Formen basierend auf Slider und Farbwähler
                let center = Pos2::new(rect.center().x, rect.center().y);
                let radius = 10.0 + (self.demo_slider_val * 0.5);

                painter.circle_filled(center, radius, self.demo_color);
                painter.circle_stroke(center, radius + 5.0, Stroke::new(2.0_f32, Color32::WHITE));

                let text_pos = Pos2::new(rect.left() + 10.0, rect.bottom() - 20.0);
                painter.text(
                    text_pos,
                    egui::Align2::LEFT_BOTTOM,
                    "Live gerendertes 2D Canvas",
                    egui::FontId::proportional(12.0),
                    Color32::LIGHT_GRAY,
                );
            });
        });
    }
}

// [RUST-KONZEPT: 01_OWNERSHIP_MOVE - Der Einstiegspunkt main()]
fn main() -> eframe::Result<()> {
    // Konfiguration des Desktop-Anwendungsfensters
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1150.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("Rust Desktop GUI- & Sprach-Lernlabor"),
        ..Default::default()
    };

    // [GUI-ELEMENT: WINDOW_APP_SHELL - Starten der nativen Desktop-Schleife]
    eframe::run_native(
        "Rust Desktop GUI- & Sprach-Lernlabor",
        native_options,
        Box::new(|cc| Ok(Box::new(RustMasterApp::new(cc)))),
    )
}