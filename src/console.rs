use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui::{self, ScrollArea};
use flexi_logger::{Logger, writers::LogWriter};
use log::Level;

const MAX_LOGS: usize = 500;

pub struct ConsoleLogWriter {
    sender: Sender<(Level, String)>,
}

impl ConsoleLogWriter {
    pub fn new(sender: Sender<(Level, String)>) -> Self {
        Self { sender }
    }
}

impl LogWriter for ConsoleLogWriter {
    fn write(
        &self,
        _now: &mut flexi_logger::DeferredNow,
        record: &log::Record,
    ) -> std::io::Result<()> {
        let _ = self
            .sender
            .send((record.level(), record.args().to_string()));
        Ok(())
    }
    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}
#[derive(Debug)]
pub struct Console {
    recv: Receiver<(Level, String)>,
    logs: Vec<(Level, String)>,
}

impl Console {
    pub fn new() -> Self {
        let (sender, recv) = channel();
        Logger::try_with_env()
            .unwrap()
            .log_to_writer(Box::new(ConsoleLogWriter::new(sender)))
            .start()
            .unwrap();
        Self {
            recv,
            logs: Default::default(),
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui) {
        while let Ok(log) = self.recv.try_recv() {
            self.logs.push(log);
            if self.logs.len() > MAX_LOGS {
                self.logs.remove(0);
            }
        }

        ScrollArea::new([true, true]).show(ui, |ui| {
            ui.allocate_space((ui.available_width(), 1.0).into());
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            for (level, log) in &self.logs {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let level_color = Self::level_color(*level);
                    ui.label(egui::RichText::new(level.to_string()).color(level_color));
                    ui.label(format!(": {}", log));
                });
            }
            ui.allocate_space((ui.available_width(), ui.available_height()).into());
        });
    }

    fn level_color(level: Level) -> egui::Color32 {
        match level {
            Level::Error => egui::Color32::from_rgb(255, 51, 102),
            Level::Warn => egui::Color32::from_rgb(255, 204, 85),
            Level::Info => egui::Color32::from_rgb(68, 170, 238),
            _ => egui::Color32::GRAY,
        }
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}
