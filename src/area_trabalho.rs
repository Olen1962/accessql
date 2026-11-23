use eframe::egui::{self, Align2, Color32, FontId, Frame, Margin, Pos2, Rect, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use rusqlite::types::Value;
use crate::{desfazer::Acao, db::{Campo, Condicao, Op, TableData}, estado::*, icones::*, tema::*, widgets::*};

#[derive(Default)]
struct Teclas {
    dx: i32, dy: i32, ctrl: bool, home: bool, end: bool, pgup: bool, pgdn: bool,
    f2: bool, delete: bool, backspace: bool, copiar: bool, cortar: bool, colar: Option<String>,
    acima: bool, desfazer: bool, refazer: bool, typed: String,
}

impl AccessApp {
    // ───────────── workspace ─────────────
    pub(crate) fn workspace(&mut self, ctx: &egui::Context) {
        if self.docs.is_empty() { return; }
        self.active = self.active.min(self.docs.len() - 1);

        let flutuantes: Vec<usize> = self.docs.iter().enumerate().filter(|(_, d)| d.flutuante()).map(|(i, _)| i).collect();
        for i in flutuantes {
            if self.docs[i].window.minimizada { continue; }
            let d = &self.docs[i];
            let mut titulo = format!("{} - {}", d.name, if self.db_path.is_empty() { "AccesSQL" } else { &self.db_path });
            let mut rect = if d.window.maximizada { ctx.available_rect() } else { d.window.rect.unwrap_or_else(|| egui::Rect::from_min_size(Pos2::new(50.0 + i as f32 * 20.0, 50.0 + i as f32 * 20.0), Vec2::new(640.0, 480.0))) };
            let id = egui::Id::new(format!("flutuante_{}", i));
            let mut open = true;
            let mut fechar = false;
            let kind = d.kind;
            let name = d.name.clone();
            let maximizada = d.window.maximizada;
            egui::Window::new(&titulo).id(id).vscroll(true).resizable(!maximizada).collapsible(false).open(&mut open).default_rect(rect).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.small_button("—").clicked() { if let Some(doc) = self.docs.get_mut(i) { doc.window.minimizada = true; doc.window.maximizada = false; } }
                    if maximizada {
                        if ui.small_button("❐").clicked() { if let Some(doc) = self.docs.get_mut(i) { doc.window.maximizada = false; if let Some(r) = doc.window.rect_anterior.take() { doc.window.rect = Some(r); } } }
                    } else {
                        if ui.small_button("□").clicked() { if let Some(doc) = self.docs.get_mut(i) { doc.window.rect_anterior = doc.window.rect; doc.window.maximizada = true; } }
                    }
                    if ui.small_button("✕").clicked() { fechar = true; }
                });
                match kind {
                    ObjKind::Table | ObjKind::Query => { let mut data = std::mem::take(&mut self.docs[i].data); self.datasheet_float(ui, &name, &mut data); self.docs[i].data = data; }
                    ObjKind::Sql => self.sql_view(ui),
                    ObjKind::Design => self.estrutura_view(ui),
                    ObjKind::QueryDesign => { let mut qbe = std::mem::take(&mut self.docs[i].qbe); if let Some(mut q) = qbe { self.qbe_view(ui, &mut q); self.docs[i].qbe = Some(q); } }
                }
                let rr = ui.max_rect(); if let Some(doc) = self.docs.get_mut(i) { if !doc.window.maximizada { doc.window.rect = Some(rr); } }
            });
            if fechar || !open { self.docs[i].window.flutuante = false; self.docs[i].window.maximizada = false; }
        }
        // barra de status: mostrar minimizadas
        let minimizadas: Vec<usize> = self.docs.iter().enumerate().filter(|(_, d)| d.flutuante() && d.window.minimizada).map(|(i, _)| i).collect();
        if !minimizadas.is_empty() {
            egui::TopBottomPanel::bottom("minimizadas").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for i in minimizadas {
                        let nome = self.docs[i].name.clone();
                        if ui.button(&nome).clicked() { if let Some(doc) = self.docs.get_mut(i) { doc.window.minimizada = false; } }
                    }
                });
            });
        }

        egui::CentralPanel::default().frame(Frame::none().fill(WORK_BG)).show(ctx, |ui| {
            let ancorados: Vec<usize> = self.docs.iter().enumerate().filter(|(_, d)| !d.flutuante()).map(|(i, _)| i).collect();
            if ancorados.is_empty() { return; }
            self.active = self.active.min(ancorados.len() - 1);

            let mut close: Option<usize> = None; let mut switch: Option<usize> = None;
            ui.horizontal(|ui| { ui.spacing_mut().item_spacing = Vec2::new(1.0, 0.0); ui.add_space(4.0); for (j, &i) in ancorados.iter().enumerate() { let d = &self.docs[i]; let sel = j == self.active; let w = tw(ui, &d.name, 12.5) + if sel { 52.0 } else { 36.0 }; let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 24.0), Sense::click()); let top = Rounding { nw: 3.0, ne: 3.0, sw: 0.0, se: 0.0 }; let p = ui.painter().clone(); if sel { p.rect_filled(rect, top, Color32::WHITE); p.rect_stroke(rect, top, Stroke::new(1.0_f32, RIBBON_LINE)); } else { p.rect_filled(rect.shrink2(Vec2::new(0.0, 2.0)), top, if resp.hovered() { rgb(0xE6EEF9) } else { rgb(0xD3DFF0) }); } draw_icon(&p, Rect::from_center_size(Pos2::new(rect.left() + 14.0, rect.center().y), Vec2::splat(14.0)), d.kind.icon()); p.text(Pos2::new(rect.left() + 26.0, rect.center().y), Align2::LEFT_CENTER, &d.name, FontId::proportional(12.5), Color32::BLACK); if sel { let cr = Rect::from_center_size(Pos2::new(rect.right() - 13.0, rect.center().y), Vec2::splat(14.0)); let cresp = ui.interact(cr, ui.id().with(("close", i)), Sense::click()); let col = if cresp.hovered() { ACCENT } else { rgb(0x6A7A94) }; p.line_segment([cr.center() + Vec2::new(-3.5, -3.5), cr.center() + Vec2::new(3.5, 3.5)], Stroke::new(1.5_f32, col)); p.line_segment([cr.center() + Vec2::new(-3.5, 3.5), cr.center() + Vec2::new(3.5, -3.5)], Stroke::new(1.5_f32, col)); if cresp.clicked() { close = Some(i); continue; } } if resp.clicked() { switch = Some(i); } } });
            if let Some(i) = close { self.docs.remove(i); self.edit = None; self.row_sel = false; self.active = self.active.saturating_sub(1); return; }
            if let Some(i) = switch { self.edit = None; self.row_sel = false; self.active = i; self.sel = (0, 1); }
            let area = ui.available_rect_before_wrap(); ui.painter().rect_filled(area, 0.0, Color32::WHITE); ui.painter().rect_stroke(area, 0.0, Stroke::new(1.0_f32, RIBBON_LINE));
            let kind = self.docs[self.active].kind; let name = self.docs[self.active].name.clone();
            match kind {
                ObjKind::Table | ObjKind::Query => self.datasheet(ui, &name, self.active),
                ObjKind::Sql => self.sql_view(ui),
                ObjKind::Design => self.estrutura_view(ui),
                ObjKind::QueryDesign => { let mut qbe = std::mem::take(&mut self.docs[self.active].qbe); if let Some(mut q) = qbe { self.qbe_view(ui, &mut q); self.docs[self.active].qbe = Some(q); } }
            }
        });
    }

    // ───────────── datasheet ─────────────
    pub(crate) fn datasheet(&mut self, ui: &mut Ui, name: &str, index: usize) {
        let mut data = std::mem::take(&mut self.docs[index].data);
        let tabela = self.docs[index].name.clone();
        let ordem_col: Option<(usize, bool)> = self.docs[index].ordem.as_ref().and_then(|(c, a)| data.cols.iter().position(|x| &x.0 == c).map(|i| (i, *a)));
        let cols_filtradas: Vec<bool> = { let d = &self.docs[index]; data.cols.iter().map(|(n, _)| d.filtro_ativo && d.filtros.iter().any(|f| &f.col == n)).collect() };
        let termo = self.rec_search.trim().to_lowercase();
        let (rh, hh, sw) = (21.0_f32, 24.0_f32, 22.0_f32);
        let extra = 150.0_f32;
        let ncols = data.cols.len(); let nrows = data.rows.len();
        let novo = data.editable as usize; let nlin = nrows + novo;
        let data_w = sw + data.cols.iter().map(|c| c.1).sum::<f32>() + extra;
        self.sel.0 = self.sel.0.min(nlin.saturating_sub(1)); self.sel.1 = self.sel.1.min(ncols.saturating_sub(1));
        let editable = data.editable && ncols > 0;
        let h = (ui.available_height() - 27.0).max(60.0);

        let mut scroll_to_sel = std::mem::take(&mut self.scroll_sel);
        let digitando = a_escrever(ui.ctx());
        if self.edit.is_some() { self.row_sel = false; }
        let mut commit: Option<(usize, usize, String, bool)> = None;
        let mut paste: Option<(usize, usize, String, bool)> = None;
        let mut menu_act: Option<u8> = None;
        if self.edit.is_none() && ncols > 0 && !digitando && self.confirm_del.is_none() && !self.foco_nav {
            let mut t = ui.input(|i| {
                let mut t = Teclas::default(); t.ctrl = i.modifiers.command;
                if i.key_pressed(egui::Key::ArrowRight) { t.dx += 1; }
                if i.key_pressed(egui::Key::ArrowLeft) { t.dx -= 1; }
                if i.key_pressed(egui::Key::ArrowDown) { t.dy += 1; }
                if i.key_pressed(egui::Key::ArrowUp) { t.dy -= 1; }
                t.home = i.key_pressed(egui::Key::Home); t.end = i.key_pressed(egui::Key::End);
                t.pgup = i.key_pressed(egui::Key::PageUp); t.pgdn = i.key_pressed(egui::Key::PageDown);
                t.f2 = i.key_pressed(egui::Key::F2) || i.key_pressed(egui::Key::Enter);
                t.delete = i.key_pressed(egui::Key::Delete); t.backspace = i.key_pressed(egui::Key::Backspace);
                t.acima = i.modifiers.command && (i.key_pressed(egui::Key::Quote) || i.key_pressed(egui::Key::Minus));
                t.desfazer = i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::Z);
                t.refazer = i.modifiers.command && (i.key_pressed(egui::Key::Y) || (i.modifiers.shift && i.key_pressed(egui::Key::Z)));
                for e in &i.events { match e { egui::Event::Copy => t.copiar = true, egui::Event::Cut => t.cortar = true, egui::Event::Paste(s) => t.colar = Some(s.clone()), _ => {} } }
                if !i.modifiers.command { for e in &i.events { if let egui::Event::Text(s) = e { if !s.chars().any(|c| c.is_control()) { t.typed.push_str(s); } } } }
                t
            });
            if let Some(a) = self.clip_acao.take() { if a == 0 { t.copiar = true; } else { t.cortar = true; } }
            let navega = t.dx != 0 || t.dy != 0 || t.home || t.end || t.pgup || t.pgdn;
            if navega {
                let pagina = (((h - hh) / rh).floor().max(1.0)) as i32;
                let (mut r, mut c) = (self.sel.0 as i32, self.sel.1 as i32);
                let (ult_r, ult_c) = ((nlin as i32 - 1).max(0), ncols as i32 - 1);
                if t.ctrl { if t.dy < 0 { r = 0; } if t.dy > 0 { r = ult_r; } if t.dx < 0 { c = 0; } if t.dx > 0 { c = ult_c; } if t.home { r = 0; c = 0; } if t.end { r = ult_r; c = ult_c; } } else { r += t.dy; c += t.dx; if t.home { c = 0; } if t.end { c = ult_c; } if t.pgup { r -= pagina; } if t.pgdn { r += pagina; } }
                self.sel = (r.clamp(0, ult_r) as usize, c.clamp(0, ult_c) as usize);
                if t.dx != 0 || t.home || t.end { self.row_sel = false; }
                scroll_to_sel = true;
            } else if t.copiar || t.cortar {
                let (r, c) = self.sel; if r < nrows { let texto = if self.row_sel { data.rows[r].join("\t") } else { data.rows[r][c.min(ncols - 1)].clone() }; ui.ctx().copy_text(texto); if t.cortar && editable && !self.row_sel { commit = Some((r, c, String::new(), false)); } }
            } else if let Some(txt) = &t.colar { if editable { paste = Some((self.sel.0, self.sel.1, txt.clone(), self.row_sel)); } }
            else if t.acima { let (r, c) = self.sel; if !editable || self.row_sel { self.msg = "Selecione uma célula para copiar o valor de cima".into(); } else if r == 0 { self.msg = "Não há nenhum registo acima".into(); } else if let Some(v) = data.rows.get(r - 1).and_then(|row| row.get(c)).cloned() { commit = Some((r, c, v, false)); } }
            else if t.desfazer { menu_act = Some(5); } else if t.refazer { menu_act = Some(6); }
            else if t.delete { if self.row_sel { menu_act = Some(2); } else if editable && self.sel.0 < nrows { commit = Some((self.sel.0, self.sel.1, String::new(), false)); } }
            else if editable && (t.f2 || t.backspace || !t.typed.is_empty()) { let (r, c) = self.sel; let ini = if !t.typed.is_empty() { t.typed } else if t.backspace { String::new() } else if r < nrows { data.rows[r][c].clone() } else { String::new() }; self.row_sel = false; self.edit = Some(Edicao { row: r, col: c, texto: ini, foco: true }); scroll_to_sel = true; }
        }

        self.clip_acao = None;
        let sel = self.sel; let row_sel = self.row_sel;
        let mut hit: Option<(usize, usize, bool)> = None; let mut gutter_hit: Option<usize> = None; let mut header_hit: Option<(usize, bool, Pos2)> = None;
        { let cols = &data.cols; let rows = &data.rows; egui::ScrollArea::both().auto_shrink([false, false]).max_height(h).show_viewport(ui, |ui, vp| { let width = data_w.max(ui.available_width()); let grid_h = hh + rh * nlin as f32; let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, grid_h.max(h)), Sense { click: true, drag: false, focusable: false }); let p = ui.painter().clone(); let o = rect.min; let ty = o.y + vp.min.y; let line = Stroke::new(1.0_f32, GRID); let gb = o.y + grid_h; p.rect_filled(rect, 0.0, Color32::WHITE); p.rect_filled(Rect::from_min_size(o, Vec2::new(sw, gb - o.y)), 0.0, rgb(0xE9E9E9)); let mut x = o.x + sw; for (_, w) in cols.iter() { x += *w; p.line_segment([Pos2::new(x, o.y), Pos2::new(x, gb)], line); } let first = (((vp.min.y - hh) / rh).floor().max(0.0)) as usize; let last = ((((vp.max.y - hh) / rh).ceil().max(0.0)) as usize + 1).min(nlin); for ri in first..last { let y = o.y + hh + rh * ri as f32; if ri == sel.0 { let cor = if row_sel { rgb(0xF2B84B) } else { rgb(0xC9D7EA) }; p.rect_filled(Rect::from_min_size(Pos2::new(o.x, y), Vec2::new(sw, rh)), 0.0, cor); if row_sel { p.rect_filled(Rect::from_min_size(Pos2::new(o.x + sw, y), Vec2::new(data_w - sw - extra, rh)), 0.0, rgb(0xDCE8F7)); } tri(&p, [Pos2::new(o.x + 7.0, y + 5.0), Pos2::new(o.x + 7.0, y + rh - 5.0), Pos2::new(o.x + 14.0, y + rh / 2.0)], rgb(0x3A3A3A)); } p.line_segment([Pos2::new(o.x, y + rh), Pos2::new(o.x + data_w, y + rh)], line); if ri == nrows { p.text(Pos2::new(o.x + sw / 2.0, y + rh / 2.0 + 2.0), Align2::CENTER_CENTER, "*", FontId::proportional(14.0), rgb(0x555555)); p.text(Pos2::new(o.x + sw + 6.0, y + rh / 2.0), Align2::LEFT_CENTER, "(Novo)", FontId::proportional(13.0), rgb(0x8A8A8A)); continue; } let mut cx = o.x + sw; for (ci, (_, w)) in cols.iter().enumerate() { let cell = Rect::from_min_size(Pos2::new(cx, y), Vec2::new(*w, rh)); if !termo.is_empty() && rows[ri][ci].to_lowercase().contains(&termo) { p.rect_filled(cell.shrink(1.0), 0.0, rgb(0xFFF2A8)); } p.with_clip_rect(cell.intersect(p.clip_rect())).text(Pos2::new(cx + 6.0, y + rh / 2.0), Align2::LEFT_CENTER, &rows[ri][ci], FontId::proportional(13.0), Color32::BLACK); cx += *w; } } if ncols > 0 && nlin > 0 { let cx = o.x + sw + cols.iter().take(sel.1).map(|c| c.1).sum::<f32>(); let cy = o.y + hh + rh * sel.0 as f32; let cell = Rect::from_min_size(Pos2::new(cx, cy), Vec2::new(cols[sel.1].1, rh)); if scroll_to_sel { ui.scroll_to_rect(Rect::from_min_max(cell.min - Vec2::new(0.0, hh), cell.max), None); } if !row_sel { p.rect_stroke(cell.shrink(0.5), 0.0, Stroke::new(2.0_f32, Color32::BLACK)); } } let mut fin: Option<(usize, usize, String, bool, bool)> = None; if let Some(ed) = self.edit.as_mut() { if ed.row <= nrows && ed.col < ncols { let ex = o.x + sw + cols.iter().take(ed.col).map(|c| c.1).sum::<f32>(); let ecell = Rect::from_min_size(Pos2::new(ex, o.y + hh + rh * ed.row as f32), Vec2::new(cols[ed.col].1.max(90.0), rh)); p.rect_filled(ecell, 0.0, Color32::WHITE); let te = egui::TextEdit::singleline(&mut ed.texto).id(egui::Id::new("editor_celula")).frame(false).font(FontId::proportional(13.0)); let r = ui.put(ecell, te); if ui.input(|i| i.modifiers.command && (i.key_pressed(egui::Key::Quote) || i.key_pressed(egui::Key::Minus))) && ed.row > 0 { if let Some(v) = rows.get(ed.row - 1).and_then(|row| row.get(ed.col)).cloned() { ed.texto = v; } } if std::env::var_os("ACCESSQL_DEBUG").is_some() { let teclas = ui.input(|i| { i.events.iter().filter_map(|e| match e { egui::Event::Key { key, pressed: true, .. } => Some(format!("{:?}", key)), _ => None }).collect::<Vec<_>>().join(",") }); if !teclas.is_empty() { self.msg = format!("[depuração] teclas={} foco={} texto={:?}", teclas, r.has_focus(), ed.texto); } } if ed.foco { r.request_focus(); ed.foco = false; } p.rect_stroke(ecell.shrink(0.5), 0.0, Stroke::new(2.0_f32, Color32::BLACK)); let (enter, tab, esc) = ui.input(|i| { (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Tab), i.key_pressed(egui::Key::Escape)) }); if esc { fin = Some((ed.row, ed.col, String::new(), false, true)); } else if r.lost_focus() { fin = Some((ed.row, ed.col, ed.texto.clone(), enter || tab, false)); } } else { fin = Some((0, 0, String::new(), false, true)); } } if let Some((r, c, t, adv, esc)) = fin { self.edit = None; if !esc { commit = Some((r, c, t, adv)); } } p.rect_filled(Rect::from_min_size(Pos2::new(o.x, ty), Vec2::new(data_w, hh)), 0.0, rgb(0xF2F2F2)); p.rect_filled(Rect::from_min_size(Pos2::new(o.x, ty), Vec2::new(sw, hh)), 0.0, rgb(0xE9E9E9)); let mut x = o.x + sw; for (hi, (label, w)) in cols.iter().enumerate() { let w = *w; if cols_filtradas.get(hi).copied().unwrap_or(false) { p.rect_filled(Rect::from_min_size(Pos2::new(x, ty), Vec2::new(w, hh)), 0.0, rgb(0xFBE3B0)); } p.text(Pos2::new(x + 6.0, ty + hh / 2.0), Align2::LEFT_CENTER, label, FontId::proportional(13.0), Color32::BLACK); if let Some((oc, asc)) = ordem_col { if oc == hi { let c = Pos2::new(x + w - 25.0, ty + hh / 2.0); let pts = if asc { [Pos2::new(c.x - 3.5, c.y + 2.0), Pos2::new(c.x + 3.5, c.y + 2.0), Pos2::new(c.x, c.y - 3.0)] } else { [Pos2::new(c.x - 3.5, c.y - 2.0), Pos2::new(c.x + 3.5, c.y - 2.0), Pos2::new(c.x, c.y + 3.0)] }; tri(&p, pts, rgb(0x2B5A9E)); } } down_arrow(&p, Pos2::new(x + w - 11.0, ty + hh / 2.0), rgb(0x6A6A6A)); x += w; p.line_segment([Pos2::new(x, ty), Pos2::new(x, ty + hh)], line); } if editable { p.text(Pos2::new(x + 6.0, ty + hh / 2.0), Align2::LEFT_CENTER, "Nova coluna", FontId::proportional(13.0), rgb(0x5A7FB5)); } p.line_segment([Pos2::new(o.x + sw, ty), Pos2::new(o.x + sw, ty + hh)], Stroke::new(1.0_f32, rgb(0xB4B4B4))); p.line_segment([Pos2::new(o.x, ty + hh), Pos2::new(o.x + data_w, ty + hh)], Stroke::new(1.0_f32, rgb(0xB4B4B4))); let localizar = |pos: Pos2| -> Option<(usize, Option<usize>)> { if pos.y <= ty + hh { return None; } let rel = pos - o; let ri = ((rel.y - hh) / rh) as usize; if ri >= nlin { return None; } if rel.x <= sw { return Some((ri, None)); } let mut acc = sw; for (ci, (_, w)) in cols.iter().enumerate() { if rel.x < acc + *w { return Some((ri, Some(ci))); } acc += *w; } None }; if resp.clicked() || resp.double_clicked() || resp.secondary_clicked() { if let Some(pos) = resp.interact_pointer_pos() { if pos.y >= ty && pos.y <= ty + hh { if pos.x > o.x + sw { let mut acc = o.x + sw; for (ci, (_, w)) in cols.iter().enumerate() { if pos.x < acc + *w { header_hit = Some((ci, resp.secondary_clicked() || pos.x > acc + *w - 22.0, pos)); break; } acc += *w; } } } else { match localizar(pos) { Some((ri, Some(ci))) => hit = Some((ri, ci, resp.double_clicked())), Some((ri, None)) => gutter_hit = Some(ri), None => {} } } } } if editable { resp.context_menu(|ui| { if ui.button("Novo Registo").clicked() { menu_act = Some(1); ui.close_menu(); } if ui.button("Eliminar Registo").clicked() { menu_act = Some(2); ui.close_menu(); } ui.separator(); if ui.button("Filtrar por Seleção").clicked() { menu_act = Some(3); ui.close_menu(); } if ui.button("Excluir Seleção").clicked() { menu_act = Some(4); ui.close_menu(); } }); } }); }
        if let Some(ri) = gutter_hit { self.sel.0 = ri; if ri < nrows { self.row_sel = true; self.msg = "Registo selecionado: prima Delete ou use Registos → Eliminar".into(); } else { self.row_sel = false; } }
        if let Some((ri, ci, dbl)) = hit { self.row_sel = false; self.sel = (ri, ci); if dbl && editable { let ini = if ri < nrows { data.rows[ri][ci].clone() } else { String::new() }; self.edit = Some(Edicao { row: ri, col: ci, texto: ini, foco: true }); } }
        if let Some((r, c, t, adv)) = commit { self.gravar_celula(&mut data, &tabela, r, c, t); self.fechar_passo(); let n = data.rows.len(); if adv { self.sel = if c + 1 < ncols { (r, c + 1) } else { ((r + 1).min(n), 0) }; self.scroll_sel = true; } }
        if let Some((r, c, txt, por_registo)) = paste { self.colar_texto(&mut data, &tabela, r, c, &txt, por_registo); self.fechar_passo(); }
        let total = data.rows.len(); self.docs[index].data = data;
        if let Some((ci, abrir, pos)) = header_hit { self.sel.1 = ci; self.row_sel = false; if abrir { self.abrir_menu_coluna(ci, Pos2::new(pos.x, pos.y + 8.0)); } }
        match menu_act { Some(1) => self.novo_registo(), Some(2) => self.pedir_eliminar(), Some(3) => self.filtrar_selecao(Op::Igual), Some(4) => self.filtrar_selecao(Op::Diferente), Some(5) => self.desfazer(), Some(6) => self.refazer(), _ => {} }
        self.record_bar(ui, total, novo == 1);
    }

    // ───────────── Helpers de Dados ─────────────
    fn recarregar_local(&mut self, data: &mut TableData, tabela: &str, index: usize) -> bool {
        let (ordem_f, filtros_f): (Option<(String, bool)>, Vec<Condicao>) = { let d = &self.docs[index]; (d.ordem.clone(), if d.filtro_ativo { d.filtros.clone() } else { Vec::new() }) };
        let r = self.db.as_ref().map(|db| db.load_filtrado(tabela, true, ordem_f.as_ref().map(|(c, a)| (c.as_str(), *a)), &filtros_f));
        match r { Some(Ok(d)) => { *data = d; true } Some(Err(e)) => { self.msg = format!("Erro ao atualizar a tabela: {}", e); false } None => false }
    }

    fn colar_texto(&mut self, data: &mut TableData, tabela: &str, r: usize, c: usize, txt: &str, por_registo: bool) {
        let linhas: Vec<Vec<String>> = txt.lines().map(|l| l.split('\t').map(str::to_string).collect()).collect();
        let ncols = data.cols.len(); let nrows = data.rows.len();
        if linhas.is_empty() || ncols == 0 { return; }
        let auto: Option<usize> = self.db.as_ref().and_then(|db| db.info_tabela(tabela).ok()).and_then(|(campos, _)| { let pks: Vec<&Campo> = campos.iter().filter(|f| f.pk).collect(); if pks.len() == 1 && pks[0].tipo.eq_ignore_ascii_case("INTEGER") { data.cols.iter().position(|x| x.0 == pks[0].nome) } else { None } });
        let completo = linhas[0].len() >= ncols;
        if r >= nrows { let c0 = if completo { 0 } else { c }; let saltar = completo && c0 == 0; let largura = linhas.iter().map(|l| l.len()).max().unwrap_or(0); let mut mapa: Vec<(usize, String, bool)> = Vec::new(); for j in 0..largura { let col = c0 + j; if col >= ncols { break; } if saltar && Some(col) == auto { continue; } mapa.push((j, data.cols[col].0.clone(), false)); } if let Some(db) = self.db.as_ref() { if let Ok((campos, _)) = db.info_tabela(tabela) { for &(j, ref nome, _) in &mapa { let Some(campo) = campos.iter().find(|f| f.nome.eq_ignore_ascii_case(nome)) else { continue }; for l in &linhas { let v = l.get(j).map(String::as_str).unwrap_or(""); if let Err(m) = crate::validacao::validar(&campo.tipo, campo.obrigatorio, v, nome) { self.msg = m; return; } } } } } let resultado = match self.db.as_ref() { Some(db) => db.inserir_linhas_ids(tabela, &mapa, &linhas, false).map(|ids| { let acoes: Vec<Acao> = ids.iter().filter_map(|rid| { db.registo_bruto(tabela, *rid).ok().map(|(colunas, valores)| Acao::Inserido { tabela: tabela.to_string(), rowid: *rid, colunas, valores }) }).collect(); (ids.len(), acoes) }), None => return }; match resultado { Ok((n, acoes)) => { self.hist_pend.extend(acoes); if self.recarregar_local(data, tabela, self.active) { self.sel = (nrows.min(data.rows.len().saturating_sub(1)), c0.min(ncols - 1)); self.scroll_sel = true; } self.msg = format!("Colado(s) {} registo(s)", n); } Err(e) => self.msg = format!("Não foi possível colar: {}", e), } } else { let c = if por_registo { 0 } else { c }; let saltar = completo && c == 0; for (i, l) in linhas.iter().enumerate() { let ri = r + i; if ri >= nrows { continue; } for (j, v) in l.iter().enumerate() { let ci = c + j; if ci >= ncols { break; } if saltar && Some(ci) == auto { continue; } let tipo = data.tipos.get(ci).map(String::as_str).unwrap_or(""); let obrig = data.obrig.get(ci).copied().unwrap_or(false); if let Err(m) = crate::validacao::validar(tipo, obrig, v, &data.cols[ci].0) { self.msg = m; return; } } } let antes = self.msg.clone(); let (mut n, mut fora_linhas, mut fora_cols) = (0usize, 0usize, false); for (i, l) in linhas.iter().enumerate() { let ri = r + i; if ri >= nrows { fora_linhas += 1; continue; } for (j, v) in l.iter().enumerate() { let ci = c + j; if ci >= ncols { fora_cols = true; break; } if saltar && Some(ci) == auto { continue; } self.gravar_celula(data, tabela, ri, ci, v.clone()); n += 1; } } if self.msg == antes { let mut m = format!("Colada(s) {} célula(s)", n); if fora_linhas > 0 { m.push_str(&format!("; {} linha(s) para além do fim da tabela foram ignoradas (use a linha (Novo) para acrescentar registos)", fora_linhas)); } if fora_cols { m.push_str("; algumas colunas ficaram de fora"); } self.msg = m; } }
    }

    fn gravar_celula(&mut self, data: &mut TableData, tabela: &str, r: usize, c: usize, texto: String) {
        let Some(db) = self.db.as_ref() else { return };
        let (ordem_f, filtros_f): (Option<(String, bool)>, Vec<Condicao>) = { let d = &self.docs[self.active]; (d.ordem.clone(), if d.filtro_ativo { d.filtros.clone() } else { Vec::new() }) };
        let col = data.cols[c].0.clone(); let tipo = data.tipos.get(c).map(String::as_str).unwrap_or(""); let obrig = data.obrig.get(c).copied().unwrap_or(false);
        if let Err(m) = crate::validacao::validar(tipo, obrig, &texto, &col) { self.msg = m; return; }
        if r < data.rows.len() {
            if data.rows[r][c] == texto { return; }
            let rid = data.rowids[r]; let antigo = db.valor_bruto(tabela, &col, rid).unwrap_or(Value::Null);
            if let Err(e) = db.update_cell(tabela, &col, rid, &texto) { self.msg = format!("Não foi possível gravar: {}", crate::db::traduz(e)); return; }
            let linha = db.fetch_row(tabela, rid); let rowid_depois = if matches!(linha, Ok(None)) { texto.trim().parse::<i64>().unwrap_or(rid) } else { rid };
            self.hist_pend.push(Acao::Celula { tabela: tabela.to_string(), rowid: rid, rowid_depois, col: col.clone(), antigo, novo: crate::db::valor(&texto) });
            match linha { Ok(Some(row)) => data.rows[r] = row, _ => { if let Ok(d) = db.load_filtrado(tabela, true, ordem_f.as_ref().map(|(c, a)| (c.as_str(), *a)), &filtros_f) { *data = d; } } }
        } else {
            if texto.is_empty() { return; }
            match db.insert_cell(tabela, &col, &texto) {
                Ok(rid) => { if let Ok((colunas, valores)) = db.registo_bruto(tabela, rid) { self.hist_pend.push(Acao::Inserido { tabela: tabela.to_string(), rowid: rid, colunas, valores }); } match db.fetch_row(tabela, rid) { Ok(Some(row)) => { data.rows.push(row); data.rowids.push(rid); } _ => { if let Ok(d) = db.load_filtrado(tabela, true, ordem_f.as_ref().map(|(c, a)| (c.as_str(), *a)), &filtros_f) { *data = d; } } } }
                Err(e) => self.msg = format!("Não foi possível criar o registo: {}", crate::db::traduz(e)),
            }
        }
    }

    pub(crate) fn novo_registo(&mut self) { match self.docs.get(self.active) { Some(d) if d.data.editable => { self.sel = (d.data.rows.len(), 0); self.scroll_sel = true; } _ => self.msg = "Esta vista não permite criar registos".into(), } }

    pub(crate) fn pedir_eliminar(&mut self) { match self.docs.get(self.active) { Some(d) if d.data.editable && self.sel.0 < d.data.rows.len() => self.confirm_del = Some(self.sel.0), Some(d) if d.data.editable => self.msg = "Selecione um registo para eliminar".into(), _ => self.msg = "Esta vista não permite eliminar registos".into(), } }

    pub(crate) fn dialogo_eliminar(&mut self, ctx: &egui::Context) { let Some(row) = self.confirm_del else { return }; let mut resp: Option<bool> = None; egui::Window::new("AccesSQL").collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| { ui.label("Vai eliminar 1 registo."); ui.label("Não poderá anular esta operação. Quer eliminar o registo?"); ui.add_space(8.0); ui.horizontal(|ui| { if ui.button("Sim").clicked() { resp = Some(true); } if ui.button("Não").clicked() { resp = Some(false); } }); }); if resp.is_none() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) { resp = Some(true); } match resp { Some(true) => { self.confirm_del = None; self.eliminar_registo(row); } Some(false) => self.confirm_del = None, None => {} } }

    fn eliminar_registo(&mut self, row: usize) { let Some(doc) = self.docs.get(self.active) else { return }; if row >= doc.data.rows.len() { return; } let nome = doc.name.clone(); let rid = doc.data.rowids[row]; let anterior = self.db.as_ref().and_then(|d| d.registo_bruto(&nome, rid).ok()); match self.db.as_ref().map(|d| d.delete_row(&nome, rid)) { Some(Ok(())) => { if let Some((colunas, valores)) = anterior { self.hist_pend.push(Acao::Apagado { tabela: nome.clone(), rowid: rid, colunas, valores }); self.fechar_passo(); } let d = &mut self.docs[self.active].data; d.rows.remove(row); d.rowids.remove(row); self.sel.0 = row.min(d.rows.len()); self.msg = "Registo eliminado".into(); } Some(Err(e)) => self.msg = format!("Não foi possível eliminar: {}", crate::db::traduz(e)), None => {} } }

    pub(crate) fn recarregar_doc(&mut self) { self.refresh_nav(); let Some(doc) = self.docs.get(self.active) else { return }; let (nome, kind) = (doc.name.clone(), doc.kind); if kind == ObjKind::Sql || kind == ObjKind::Design { return; } if self.carregar_doc(self.active) { self.edit = None; self.msg = format!("«{}» atualizada", nome); } }
    
    pub(crate) fn record_bar(&mut self, ui: &mut Ui, total: usize, novo: bool) { let max = if novo { total } else { total.saturating_sub(1) }; Frame::none().fill(rgb(0xF0F0F0)).inner_margin(Margin::symmetric(6.0, 3.0)).show(ui, |ui| { ui.set_width(ui.available_width()); ui.horizontal(|ui| { ui.spacing_mut().item_spacing.x = 2.0; ui.label(RichText::new("Registo:").size(12.0)); if nav_arrow(ui, 0).clicked() { self.sel.0 = 0; } if nav_arrow(ui, 1).clicked() { self.sel.0 = self.sel.0.saturating_sub(1); } Frame::none().fill(Color32::WHITE).stroke(Stroke::new(1.0_f32, rgb(0xA0A0A0))).inner_margin(Margin::symmetric(8.0, 1.0)).show(ui, |ui| { ui.label(RichText::new(format!("{} de {}", if total == 0 { 0 } else { self.sel.0.min(total - 1) + 1 }, total)).size(12.0)); }); if nav_arrow(ui, 2).clicked() { self.sel.0 = (self.sel.0 + 1).min(max); } if nav_arrow(ui, 3).clicked() { self.sel.0 = total.saturating_sub(1); } if nav_arrow(ui, 4).clicked() && novo { self.sel.0 = total; } ui.add_space(8.0); let (n_filtros, ligado) = self.docs.get(self.active).map_or((0, false), |d| (d.filtros.len(), d.filtro_ativo)); let (rotulo, cor) = if n_filtros > 0 && ligado { ("Filtrado", rgb(0xB45F06)) } else { ("Sem Filtro", rgb(0x555555)) }; if ui.add_enabled(n_filtros > 0, egui::Button::new(RichText::new(rotulo).size(12.0).color(cor))).on_hover_text("Liga ou desliga os filtros").clicked() { self.alternar_filtro(); } ui.add_space(4.0); let r = ui.add(egui::TextEdit::singleline(&mut self.rec_search).id(egui::Id::new("rec_search")).hint_text("Procurar").desired_width(170.0)); if std::mem::take(&mut self.focar_busca) { r.request_focus(); } if r.changed() { self.buscar_texto(self.sel.0); } if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { self.buscar_texto(self.sel.0 + 1); r.request_focus(); } }); }); }
    
    // Placeholder para janelas flutuantes (pode copiar lógica de datasheet se precisar)
    pub(crate) fn datasheet_float(&mut self, ui: &mut Ui, name: &str, data: &mut TableData) {
        ui.label(format!("Janela flutuante: {} (datasheet_float não implementado totalmente)", name));
    }
} // FIM DO IMPL ACCESSAPP