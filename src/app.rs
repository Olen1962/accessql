use eframe::egui;
use crate::{backstage::*, estado::{a_escrever, AccessApp}, menus::*, navegacao::*, ribbon::*, area_trabalho::*, status::*};

impl eframe::App for AccessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // sem os atalhos de zoom do egui (Ctrl +, Ctrl -, Ctrl 0): em teclados portugueses a tecla
        // do apóstrofo é fisicamente a do «menos» e o egui consumia o Ctrl+'
        ctx.options_mut(|o| o.zoom_with_keyboard = false);
        if std::env::var_os("ACCESSQL_DEBUG").is_some() {
            // diagnóstico: mostra na barra de estado as teclas e o texto que chegam ao egui
            let t: Vec<String> = ctx.input(|i| {
                i.events
                    .iter()
                    .filter_map(|e| match e {
                        egui::Event::Key { key, pressed: true, modifiers, .. } => {
                            Some(format!("{:?}{}", key, if modifiers.ctrl { "+Ctrl" } else { "" }))
                        }
                        egui::Event::Text(s) => Some(format!("texto«{}»", s.escape_debug())),
                        _ => None,
                    })
                    .collect()
            });
            if !t.is_empty() {
                self.msg = format!("[depuração] {}", t.join(" "));
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.backstage = false;
            self.dropdown = None;
            self.confirm_del = None;
            self.pedir_nome = None;
            self.renomear_obj = None;
            self.menu_col = None;
            self.importar = None;
            self.exportar_csv = None;
        }
        // o último clique decide a quem pertence o F2: painel de navegação ou área de trabalho
        if ctx.input(|i| i.pointer.primary_pressed()) {
            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                self.foco_nav = !self.backstage && self.nav_rect.map_or(false, |r| r.contains(pos));
            }
        }
        if self.foco_nav
            && self.renomear_obj.is_none()
            && !a_escrever(ctx)
            && ctx.input(|i| i.key_pressed(egui::Key::F2))
        {
            self.pedir_renomear_nav();
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
            self.dialogo_guardar_consulta(ctx);
            self.dialogo_renomear(ctx);
            self.desenhar_menu_col(ctx);
            self.dialogo_importar_csv(ctx);
            self.dialogo_exportar_csv(ctx);
        }
        self.show_dropdown(ctx);
        // X11/XIM: com o método de entrada (IME) ativo, o Backspace e as setas são
        // "engolidos" antes de chegarem ao egui. Retirar o IME no fim de cada frame
        // (depois de as caixas de texto o terem pedido) resolve-o.
        ctx.output_mut(|o| o.ime = None);
    }
}
