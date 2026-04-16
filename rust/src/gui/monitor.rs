//! Monitor window: shows the recorder status, a live list of recordings on
//! disk, and quick actions (open folder, play, view keylog, delete).

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::{DateTime, Local};
use eframe::{egui, App, CreationContext, NativeOptions};

use crate::config::Config;
use crate::ffmpeg;

pub fn run(cfg: Config) -> Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 650.0])
            .with_min_inner_size([720.0, 420.0])
            .with_title("SilentWitness Monitor"),
        ..Default::default()
    };
    eframe::run_native(
        "SilentWitness Monitor",
        options,
        Box::new(|cc| Ok(Box::new(MonitorApp::new(cc, cfg)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))
}

#[derive(Clone, Debug)]
struct Recording {
    path: PathBuf,
    filename: String,
    modified: DateTime<Local>,
    size_bytes: u64,
    user: String,
    duration_secs: Option<f64>,
}

struct MonitorApp {
    cfg: Config,
    recordings: Vec<Recording>,
    search: String,
    user_filter: String,
    limit: usize,
    last_refresh: Instant,
    ffprobe: Option<PathBuf>,
    log_lines: Vec<String>,
}

impl MonitorApp {
    fn new(_cc: &CreationContext<'_>, cfg: Config) -> Self {
        let ffprobe = ffmpeg::ffprobe_path(&cfg);
        let mut app = Self {
            cfg,
            recordings: Vec::new(),
            search: String::new(),
            user_filter: "All".to_string(),
            limit: 50,
            last_refresh: Instant::now() - Duration::from_secs(60),
            ffprobe,
            log_lines: Vec::new(),
        };
        app.refresh();
        app
    }

    fn log(&mut self, msg: impl Into<String>) {
        let stamped = format!("{} — {}", Local::now().format("%H:%M:%S"), msg.into());
        self.log_lines.push(stamped);
        if self.log_lines.len() > 500 {
            self.log_lines.drain(..self.log_lines.len() - 500);
        }
    }

    fn refresh(&mut self) {
        self.recordings.clear();
        let root = Path::new(&self.cfg.recording.recording_dir);
        if !root.exists() {
            self.last_refresh = Instant::now();
            return;
        }
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(read) = std::fs::read_dir(&dir) else { continue };
            for entry in read.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // skip recycle bins
                    if path.file_name().and_then(|s| s.to_str()) == Some("_RecycleBin") {
                        continue;
                    }
                    stack.push(path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("mkv") {
                    if let Ok(meta) = entry.metadata() {
                        let modified: DateTime<Local> = meta
                            .modified()
                            .ok()
                            .and_then(|t| t.elapsed().ok().map(|e| Local::now() - chrono::Duration::from_std(e).unwrap_or_default()))
                            .unwrap_or_else(Local::now);
                        let filename = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                        let user = filename.split('-').next().unwrap_or("Unknown").to_string();
                        let duration_secs = self
                            .ffprobe
                            .as_ref()
                            .and_then(|p| ffmpeg::probe_duration(p, &path).ok());
                        self.recordings.push(Recording {
                            path,
                            filename,
                            modified,
                            size_bytes: meta.len(),
                            user,
                            duration_secs,
                        });
                    }
                }
            }
        }
        self.recordings
            .sort_by(|a, b| b.modified.cmp(&a.modified));
        self.last_refresh = Instant::now();
    }

    fn delete_to_recycle(&mut self, path: &Path) {
        let Some(parent) = path.parent() else { return };
        let recycle = parent.join("_RecycleBin");
        if let Err(e) = std::fs::create_dir_all(&recycle) {
            self.log(format!("recycle dir error: {e}"));
            return;
        }
        let filename = path.file_name().unwrap_or_default();
        let mut target = recycle.join(filename);
        let mut i = 1;
        while target.exists() {
            let stem = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
            let ext = path.extension().unwrap_or_default().to_string_lossy().into_owned();
            target = recycle.join(format!("{stem}_{i}.{ext}"));
            i += 1;
        }
        if let Err(e) = std::fs::rename(path, &target) {
            self.log(format!("rename error: {e}"));
        } else {
            self.log(format!("recycled {:?}", path.file_name().unwrap_or_default()));
        }
    }
}

fn open_in_default_app(path: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

impl App for MonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_refresh.elapsed() > Duration::from_secs(15) {
            self.refresh();
        }

        egui::TopBottomPanel::top("title").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("SilentWitness Monitor");
                ui.separator();
                ui.label(format!(
                    "Recording dir: {}",
                    self.cfg.recording.recording_dir
                ));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::TopBottomPanel::top("controls")
                .resizable(false)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Search:");
                        ui.text_edit_singleline(&mut self.search);
                        ui.separator();
                        ui.label("Show last:");
                        egui::ComboBox::from_id_salt("limit")
                            .selected_text(format!("{}", self.limit))
                            .show_ui(ui, |ui| {
                                for &v in &[30usize, 50, 100, 500] {
                                    ui.selectable_value(&mut self.limit, v, v.to_string());
                                }
                            });
                        ui.separator();
                        let users: Vec<String> = {
                            let mut u: Vec<String> = self
                                .recordings
                                .iter()
                                .map(|r| r.user.clone())
                                .collect();
                            u.sort();
                            u.dedup();
                            u
                        };
                        egui::ComboBox::from_id_salt("user_filter")
                            .selected_text(&self.user_filter)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.user_filter,
                                    "All".to_string(),
                                    "All",
                                );
                                for u in users {
                                    ui.selectable_value(&mut self.user_filter, u.clone(), u);
                                }
                            });
                        ui.separator();
                        if ui.button("Refresh").clicked() {
                            self.refresh();
                        }
                    });
                });

            let search = self.search.to_lowercase();
            let mut visible: Vec<&Recording> = self
                .recordings
                .iter()
                .filter(|r| search.is_empty() || r.filename.to_lowercase().contains(&search))
                .filter(|r| self.user_filter == "All" || r.user == self.user_filter)
                .take(self.limit)
                .collect();
            visible.sort_by(|a, b| b.modified.cmp(&a.modified));

            let mut action: Option<(String, PathBuf)> = None;

            use egui_extras::{Column, TableBuilder};
            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(Column::auto().at_least(90.0))
                .column(Column::auto().at_least(70.0))
                .column(Column::auto().at_least(70.0))
                .column(Column::auto().at_least(80.0))
                .column(Column::auto().at_least(80.0))
                .column(Column::remainder().at_least(260.0))
                .column(Column::auto())
                .header(22.0, |mut header| {
                    for h in ["Date", "Time", "Duration", "Size", "User", "File", "Actions"] {
                        header.col(|ui| {
                            ui.strong(h);
                        });
                    }
                })
                .body(|mut body| {
                    for rec in visible {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(rec.modified.format("%Y-%m-%d").to_string());
                            });
                            row.col(|ui| {
                                ui.label(rec.modified.format("%H:%M:%S").to_string());
                            });
                            row.col(|ui| {
                                ui.label(match rec.duration_secs {
                                    Some(s) => format_secs(s),
                                    None => "—".to_string(),
                                });
                            });
                            row.col(|ui| {
                                ui.label(format!(
                                    "{:.1} MB",
                                    rec.size_bytes as f64 / 1_048_576.0
                                ));
                            });
                            row.col(|ui| {
                                ui.label(&rec.user);
                            });
                            row.col(|ui| {
                                ui.label(&rec.filename);
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    if ui.small_button("Folder").clicked() {
                                        if let Some(parent) = rec.path.parent() {
                                            action = Some(("open".to_string(), parent.to_path_buf()));
                                        }
                                    }
                                    if ui.small_button("Play").clicked() {
                                        action = Some(("open".to_string(), rec.path.clone()));
                                    }
                                    if ui.small_button("Keylog").clicked() {
                                        action = Some((
                                            "keylog".to_string(),
                                            rec.path.with_extension("txt"),
                                        ));
                                    }
                                    if ui.small_button("Delete").clicked() {
                                        action = Some(("delete".to_string(), rec.path.clone()));
                                    }
                                });
                            });
                        });
                    }
                });

            if let Some((op, path)) = action {
                match op.as_str() {
                    "open" => match open_in_default_app(&path) {
                        Ok(()) => self.log(format!("opened {:?}", path)),
                        Err(e) => self.log(format!("open error: {e}")),
                    },
                    "keylog" => {
                        if path.exists() {
                            let _ = open_in_default_app(&path);
                        } else {
                            self.log("no keylog file");
                        }
                    }
                    "delete" => self.delete_to_recycle(&path),
                    _ => {}
                }
            }
        });

        egui::TopBottomPanel::bottom("log").show(ctx, |ui| {
            ui.label("Activity log");
            egui::ScrollArea::vertical()
                .max_height(120.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &self.log_lines {
                        ui.monospace(line);
                    }
                });
        });

        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

fn format_secs(seconds: f64) -> String {
    let secs = seconds as u64;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}
