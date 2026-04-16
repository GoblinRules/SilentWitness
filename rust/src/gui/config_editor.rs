//! Graphical TOML config editor.

use std::path::PathBuf;

use anyhow::Result;
use eframe::{egui, App, CreationContext, NativeOptions};

use crate::config::{parse_hms, Config};

pub fn run(cfg: Config, path: PathBuf) -> Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([760.0, 620.0])
            .with_min_inner_size([640.0, 520.0])
            .with_title("SilentWitness Config"),
        ..Default::default()
    };
    eframe::run_native(
        "SilentWitness Config",
        options,
        Box::new(|cc| Ok(Box::new(ConfigApp::new(cc, cfg, path)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))
}

struct ConfigApp {
    cfg: Config,
    path: PathBuf,
    status: String,
}

impl ConfigApp {
    fn new(_cc: &CreationContext<'_>, cfg: Config, path: PathBuf) -> Self {
        Self {
            cfg,
            path,
            status: String::new(),
        }
    }
}

impl App for ConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("hdr").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("SilentWitness Config");
                ui.separator();
                ui.monospace(self.path.display().to_string());
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::CollapsingHeader::new("Paths")
                    .default_open(true)
                    .show(ui, |ui| {
                        labeled_edit(ui, "ffmpeg", &mut self.cfg.paths.ffmpeg);
                        labeled_edit(ui, "ffprobe", &mut self.cfg.paths.ffprobe);
                        labeled_edit(ui, "key_overlay", &mut self.cfg.paths.key_overlay);
                    });

                egui::CollapsingHeader::new("Recording")
                    .default_open(true)
                    .show(ui, |ui| {
                        labeled_edit(ui, "recording_dir", &mut self.cfg.recording.recording_dir);
                        labeled_edit(
                            ui,
                            "filename_template",
                            &mut self.cfg.recording.filename_template,
                        );
                        ui.checkbox(&mut self.cfg.recording.record_video, "record_video");
                        ui.checkbox(
                            &mut self.cfg.recording.enable_key_logging,
                            "enable_key_logging",
                        );
                        ui.checkbox(
                            &mut self.cfg.recording.enable_key_overlay,
                            "enable_key_overlay",
                        );

                        hms_edit(
                            ui,
                            "record_duration (HH:MM:SS)",
                            &mut self.cfg.recording.record_duration,
                        );
                        hms_edit(
                            ui,
                            "idle_threshold (HH:MM:SS)",
                            &mut self.cfg.recording.idle_threshold,
                        );

                        ui.horizontal(|ui| {
                            ui.label("check_interval (s):");
                            ui.add(egui::DragValue::new(&mut self.cfg.recording.check_interval).range(1..=600));
                        });
                        ui.horizontal(|ui| {
                            ui.label("framerate:");
                            ui.add(egui::DragValue::new(&mut self.cfg.recording.framerate).range(5..=120));
                        });
                        ui.horizontal(|ui| {
                            ui.label("crf (quality, lower = better):");
                            ui.add(egui::DragValue::new(&mut self.cfg.recording.crf).range(10..=40));
                        });
                        labeled_edit(ui, "preset", &mut self.cfg.recording.preset);
                    });

                egui::CollapsingHeader::new("Tray")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.checkbox(&mut self.cfg.tray.enabled, "enabled");
                        labeled_edit(ui, "tooltip_idle", &mut self.cfg.tray.tooltip_idle);
                        labeled_edit(ui, "tooltip_recording", &mut self.cfg.tray.tooltip_recording);
                    });

                egui::CollapsingHeader::new("Logging")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.checkbox(&mut self.cfg.logging.enabled, "enabled");
                        ui.checkbox(&mut self.cfg.logging.to_file, "to_file");
                        labeled_edit(ui, "log_dir", &mut self.cfg.logging.log_dir);
                        labeled_edit(ui, "level", &mut self.cfg.logging.level);
                    });

                egui::CollapsingHeader::new("Startup")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.checkbox(
                            &mut self.cfg.startup.enable_startup_menu,
                            "enable_startup_menu",
                        );
                        ui.checkbox(&mut self.cfg.startup.enable_auto_start, "enable_auto_start");
                        ui.checkbox(&mut self.cfg.startup.start_recorder, "start_recorder");
                        ui.checkbox(&mut self.cfg.startup.start_monitor, "start_monitor");
                        ui.horizontal(|ui| {
                            ui.label("startup_delay_secs:");
                            ui.add(
                                egui::DragValue::new(&mut self.cfg.startup.startup_delay_secs)
                                    .range(0..=600),
                            );
                        });
                    });
            });
        });

        egui::TopBottomPanel::bottom("actions").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    match self.cfg.save(&self.path) {
                        Ok(()) => self.status = format!("Saved to {}", self.path.display()),
                        Err(e) => self.status = format!("Save failed: {e}"),
                    }
                }
                if ui.button("Reload").clicked() {
                    match Config::load_or_create(Some(&self.path)) {
                        Ok((c, _)) => {
                            self.cfg = c;
                            self.status = "Reloaded".into();
                        }
                        Err(e) => self.status = format!("Reload failed: {e}"),
                    }
                }
                if ui.button("Reset to defaults").clicked() {
                    self.cfg = Config::default();
                    self.status = "Defaults loaded (not yet saved)".into();
                }
                ui.separator();
                ui.label(&self.status);
            });
        });
    }
}

fn labeled_edit(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));
    });
}

fn hms_edit(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        let valid = parse_hms(value).is_some();
        let mut te = egui::TextEdit::singleline(value).desired_width(140.0);
        if !valid {
            te = te.text_color(egui::Color32::from_rgb(220, 60, 60));
        }
        ui.add(te);
        if !valid {
            ui.label("⚠ expected HH:MM:SS");
        }
    });
}
