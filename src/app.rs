use eframe::egui;
use crate::{backstage::*, estado::AccessApp, menus::*, navegacao::*, ribbon::*, area_trabalho::*, status::*};

impl eframe::App for AccessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.backstage = false;
            self.dropdown = None;
            self.confirm_del = None;
        }
        self.qat(ctx);
        self.tab_strip(ctx);
        self.status_bar(ctx);
        if self.backstage {
            self.backstage_ui(ctx);
        } else {
            self.ribbon(ctx);
            self.nav_pane(ctx);
            self.workspace(ctx);
            self.dialogo_eliminar(ctx);
        }
        self.show_dropdown(ctx);
    }
}
