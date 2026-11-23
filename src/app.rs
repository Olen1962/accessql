use eframe::egui;
use crate::estado::{a_escrever, AccessApp};

impl eframe::App for AccessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.options_mut(|o| o.zoom_with_keyboard = false);
        let passos_zoom: f32 = ctx.input(|i| { i.events.iter().filter_map(|e| match e { egui::Event::MouseWheel { unit, delta, modifiers } if modifiers.ctrl => { let y = match unit { egui::MouseWheelUnit::Point => delta.y / 50.0, _ => delta.y }; Some(y) } _ => None }).sum() });
        if passos_zoom != 0.0 { let z = ctx.zoom_factor() * 1.1_f32.powf(passos_zoom); let z = z.clamp(0.5, 2.5); ctx.set_zoom_factor((z * 20.0).round() / 20.0); }
        if std::env::var_os("ACCESSQL_DEBUG").is_some() { let t: Vec<String> = ctx.input(|i| { i.events.iter().filter_map(|e| match e { egui::Event::Key { key, pressed: true, modifiers, .. } => Some(format!("{:?}{}", key, if modifiers.ctrl { "+Ctrl" } else { "" })), egui::Event::Text(s) => Some(format!("texto«{}»", s.escape_debug())), _ => None }).collect() }); if !t.is_empty() { self.msg = format!("[depuração] {}", t.join(" ")); } }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) { self.backstage = false; self.confirm_del = None; self.pedir_nome = None; self.renomear_obj = None; self.duplicar_obj = None; self.menu_col = None; self.importar = None; self.exportar_csv = None; self.totais_aberto = false; self.indices = None; self.relacoes = None; self.eliminar_obj = None; }
        if ctx.input(|i| i.pointer.primary_pressed()) { if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) { self.foco_nav = !self.backstage && self.nav_rect.map_or(false, |r| r.contains(pos)); } }
        if self.foco_nav && self.renomear_obj.is_none() && !a_escrever(ctx) && ctx.input(|i| i.key_pressed(egui::Key::F2)) { self.pedir_renomear_nav(); }
        if self.foco_nav && self.renomear_obj.is_none() && self.duplicar_obj.is_none() && self.eliminar_obj.is_none() && !a_escrever(ctx) && ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::D)) { self.pedir_duplicar_nav(); }
        if self.foco_nav && self.eliminar_obj.is_none() && self.renomear_obj.is_none() && !a_escrever(ctx) && ctx.input(|i| i.key_pressed(egui::Key::Delete)) { self.pedir_eliminar_nav(); }

        if self.qbe_add_table_aberto { self.qbe_dialogo_adicionar_tabela_inline(ctx); }

        self.qat(ctx); self.tab_strip(ctx); self.status_bar(ctx);
        if self.backstage { self.backstage_ui(ctx); } else { self.ribbon(ctx); self.nav_pane(ctx); self.workspace(ctx); self.dialogo_eliminar(ctx); self.dialogo_guardar_consulta(ctx); self.dialogo_renomear(ctx); self.dialogo_duplicar(ctx); self.desenhar_menu_col(ctx); self.dialogo_importar_csv(ctx); self.dialogo_exportar_csv(ctx); self.desenhar_totais(ctx); self.dialogo_indices(ctx); self.dialogo_relacoes(ctx); self.dialogo_eliminar_obj(ctx); }
        ctx.output_mut(|o| o.ime = None);
    }
}