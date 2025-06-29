use eframe::CreationContext;
use egui_snarl::{Snarl, ui::SnarlStyle};

use crate::node::{Node, NodeViewer};

pub struct App {
    snarl: Snarl<Node>,
    style: SnarlStyle,
}

const fn default_snarl_style() -> SnarlStyle {
    SnarlStyle {
        ..SnarlStyle::new()
    }
}

impl App {
    pub fn new(cx: &CreationContext) -> Self {
        let snarl = cx.storage.map_or_else(Snarl::new, |storage| {
            storage
                .get_string("snarl")
                .and_then(|snarl| serde_json::from_str(&snarl).ok())
                .unwrap_or_default()
        });

        let style = cx.storage.map_or_else(SnarlStyle::new, |storage| {
            storage
                .get_string("style")
                .and_then(|style| serde_json::from_str(&style).ok())
                .unwrap_or_else(default_snarl_style)
        });

        Self { snarl, style }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.snarl.show(&mut NodeViewer, &self.style, "snarl", ui);
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let snarl = serde_json::to_string(&self.snarl).unwrap();
        storage.set_string("snarl", snarl);

        let style = serde_json::to_string(&self.style).unwrap();
        storage.set_string("style", style);
    }
}
