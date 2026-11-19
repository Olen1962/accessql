use eframe::egui;
use crate::estado::{a_escrever, AccessApp};

impl eframe::App for AccessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // sem os atalhos de zoom do egui (Ctrl +, Ctrl -, Ctrl 0): em teclados portugueses a tecla
        // do apóstrofo é fisicamente a do «menos» e o egui consumia o Ctrl+'
        ctx.options_mut(|o| o.zoom_with_keyboard = false);
        // Ctrl + roda do rato aumenta/diminui a escala de toda a interface.
        // O egui 0.29 trata a roda com Ctrl como zoom interno, mas não altera
        // o zoom global; os atalhos de teclado estão desativados acima.
        let passos_zoom: f32 = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::MouseWheel { unit, delta, modifiers } if modifiers.ctrl => {
                        let y = match unit {
                            egui::MouseWheelUnit::Point => delta.y / 50.0,
                            _ => delta.y,
                        };
                        Some(y)
                    }
                    _ => None,
                })
                .sum()
        });
        if passos_zoom != 0.0 {
            // Rola para baixo = diminuir; para cima = aumentar.
            let z = ctx.zoom_factor() / 1.1_f32.powf(passos_zoom);
            let z = z.clamp(0.5, 2.5);
            ctx.set_zoom_factor((z * 20.0).round() / 20.0);
        }
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
            self.confirm_del = None;
            self.pedir_nome = None;
            self.renomear_obj = None;
            self.menu_col = None;
            self.importar = None;
            self.exportar_csv = None;
            self.totais_aberto = false;
            self.indices = None;
            self.relacoes = None;
            self.eliminar_obj = None;
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
        // Delete no painel de navegação: eliminar a tabela ou consulta selecionada (com confirmação)
        if self.foco_nav
            && self.eliminar_obj.is_none()
            && self.renomear_obj.is_none()
            && !a_escrever(ctx)
            && ctx.input(|i| i.key_pressed(egui::Key::Delete))
        {
            self.pedir_eliminar_nav();
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
            self.desenhar_totais(ctx);
            self.dialogo_indices(ctx);
            self.dialogo_relacoes(ctx);
            self.dialogo_eliminar_obj(ctx);
        }
        // X11/XIM: com o método de entrada (IME) ativo, o Backspace e as setas são
        // "engolidos" antes de chegarem ao egui. Retirar o IME no fim de cada frame
        // (depois de as caixas de texto o terem pedido) resolve-o.
        ctx.output_mut(|o| o.ime = None);
    }
}
