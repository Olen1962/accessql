use eframe::egui::{self, Align, Align2, Color32, FontId, Frame, Layout, Margin, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use crate::{db::TableData, estado::*, icones::*, tema::*, widgets::*};
#[derive(Default)]
struct Teclas {
    dx: i32,
    dy: i32,
    ctrl: bool,
    home: bool,
    end: bool,
    pgup: bool,
    pgdn: bool,
    f2: bool,
    delete: bool,
    backspace: bool,
    typed: String,
}

impl AccessApp {
// ───────────── Área de trabalho (documentos em separadores) ─────────────
    pub(crate) fn workspace(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().frame(Frame::none().fill(WORK_BG)).show(ctx, |ui| {
            if self.docs.is_empty() {
                return;
            }
            self.active = self.active.min(self.docs.len() - 1);
            let mut close: Option<usize> = None;
            let mut switch: Option<usize> = None;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(1.0, 0.0);
                ui.add_space(4.0);
                for (i, d) in self.docs.iter().enumerate() {
                    let sel = i == self.active;
                    let w = tw(ui, &d.name, 12.5) + if sel { 52.0 } else { 36.0 };
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 24.0), Sense::click());
                    let top = Rounding { nw: 3.0, ne: 3.0, sw: 0.0, se: 0.0 };
                    let p = ui.painter().clone();
                    if sel {
                        p.rect_filled(rect, top, Color32::WHITE);
                        p.rect_stroke(rect, top, Stroke::new(1.0_f32, RIBBON_LINE));
                    } else {
                        p.rect_filled(rect.shrink2(Vec2::new(0.0, 2.0)), top, if resp.hovered() { rgb(0xE6EEF9) } else { rgb(0xD3DFF0) });
                    }
                    draw_icon(&p, Rect::from_center_size(Pos2::new(rect.left() + 14.0, rect.center().y), Vec2::splat(14.0)), d.kind.icon());
                    p.text(Pos2::new(rect.left() + 26.0, rect.center().y), Align2::LEFT_CENTER, &d.name, FontId::proportional(12.5), Color32::BLACK);
                    if sel {
                        let cr = Rect::from_center_size(Pos2::new(rect.right() - 13.0, rect.center().y), Vec2::splat(14.0));
                        let cresp = ui.interact(cr, ui.id().with(("close", i)), Sense::click());
                        let col = if cresp.hovered() { ACCENT } else { rgb(0x6A7A94) };
                        p.line_segment([cr.center() + Vec2::new(-3.5, -3.5), cr.center() + Vec2::new(3.5, 3.5)], Stroke::new(1.5_f32, col));
                        p.line_segment([cr.center() + Vec2::new(-3.5, 3.5), cr.center() + Vec2::new(3.5, -3.5)], Stroke::new(1.5_f32, col));
                        if cresp.clicked() {
                            close = Some(i);
                            continue;
                        }
                    }
                    if resp.clicked() {
                        switch = Some(i);
                    }
                }
            });
            if let Some(i) = close {
                self.docs.remove(i);
                self.edit = None;
                self.row_sel = false;
                self.active = self.active.saturating_sub(1);
                return;
            }
            if let Some(i) = switch {
                self.edit = None;
                self.row_sel = false;
                self.active = i;
                self.sel = (0, 1);
            }
            let area = ui.available_rect_before_wrap();
            ui.painter().rect_filled(area, 0.0, Color32::WHITE);
            ui.painter().rect_stroke(area, 0.0, Stroke::new(1.0_f32, RIBBON_LINE));
            let kind = self.docs[self.active].kind;
            let name = self.docs[self.active].name.clone();
            match kind {
                ObjKind::Table | ObjKind::Query => self.datasheet(ui, &name),
                ObjKind::Sql => self.sql_view(ui),
                ObjKind::Design => self.estrutura_view(ui),
            }
        });
    }
    pub(crate) fn datasheet(&mut self, ui: &mut Ui, _name: &str) {
        let mut data = std::mem::take(&mut self.docs[self.active].data);
        let tabela = self.docs[self.active].name.clone();
        let (rh, hh, sw) = (21.0_f32, 24.0_f32, 22.0_f32);
        let extra = 150.0_f32;
        let ncols = data.cols.len();
        let nrows = data.rows.len();
        // só as tabelas têm a linha «(Novo)»; consultas e resultados SQL são só de leitura
        let novo = data.editable as usize;
        let nlin = nrows + novo;
        let data_w = sw + data.cols.iter().map(|c| c.1).sum::<f32>() + extra;
        self.sel.0 = self.sel.0.min(nlin.saturating_sub(1));
        self.sel.1 = self.sel.1.min(ncols.saturating_sub(1));
        let editable = data.editable && ncols > 0;
        let h = (ui.available_height() - 27.0).max(60.0);

        // ── teclado (quando não se está a editar) ──
        let mut scroll_to_sel = std::mem::take(&mut self.scroll_sel);
        let digitando = ui.ctx().memory(|m| {
            matches!(m.focused(), Some(id) if id == egui::Id::new("nav_filter") || id == egui::Id::new("rec_search") || id == egui::Id::new("sql_editor") || id == egui::Id::new("nome_consulta"))
        });
        if self.edit.is_some() {
            self.row_sel = false;
        }
        let mut commit: Option<(usize, usize, String, bool)> = None;
        // ações que precisam dos dados do documento: executadas só depois de os devolver
        let mut menu_act: Option<u8> = None;
        if self.edit.is_none() && ncols > 0 && !digitando && self.confirm_del.is_none() {
            let t = ui.input(|i| {
                let mut t = Teclas::default();
                t.ctrl = i.modifiers.command;
                if i.key_pressed(egui::Key::ArrowRight) {
                    t.dx += 1;
                }
                if i.key_pressed(egui::Key::ArrowLeft) {
                    t.dx -= 1;
                }
                if i.key_pressed(egui::Key::ArrowDown) {
                    t.dy += 1;
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    t.dy -= 1;
                }
                t.home = i.key_pressed(egui::Key::Home);
                t.end = i.key_pressed(egui::Key::End);
                t.pgup = i.key_pressed(egui::Key::PageUp);
                t.pgdn = i.key_pressed(egui::Key::PageDown);
                t.f2 = i.key_pressed(egui::Key::F2) || i.key_pressed(egui::Key::Enter);
                t.delete = i.key_pressed(egui::Key::Delete);
                t.backspace = i.key_pressed(egui::Key::Backspace);
                if !i.modifiers.command {
                    for e in &i.events {
                        if let egui::Event::Text(s) = e {
                            if !s.chars().any(|c| c.is_control()) {
                                t.typed.push_str(s);
                            }
                        }
                    }
                }
                t
            });
            let navega = t.dx != 0 || t.dy != 0 || t.home || t.end || t.pgup || t.pgdn;
            if navega {
                let pagina = (((h - hh) / rh).floor().max(1.0)) as i32;
                let (mut r, mut c) = (self.sel.0 as i32, self.sel.1 as i32);
                let (ult_r, ult_c) = ((nlin as i32 - 1).max(0), ncols as i32 - 1);
                if t.ctrl {
                    // Ctrl+setas: primeiro/último registo e campo; Ctrl+Home/End: cantos
                    if t.dy < 0 {
                        r = 0;
                    }
                    if t.dy > 0 {
                        r = ult_r;
                    }
                    if t.dx < 0 {
                        c = 0;
                    }
                    if t.dx > 0 {
                        c = ult_c;
                    }
                    if t.home {
                        r = 0;
                        c = 0;
                    }
                    if t.end {
                        r = ult_r;
                        c = ult_c;
                    }
                } else {
                    r += t.dy;
                    c += t.dx;
                    if t.home {
                        c = 0;
                    }
                    if t.end {
                        c = ult_c;
                    }
                    if t.pgup {
                        r -= pagina;
                    }
                    if t.pgdn {
                        r += pagina;
                    }
                }
                self.sel = (r.clamp(0, ult_r) as usize, c.clamp(0, ult_c) as usize);
                if t.dx != 0 || t.home || t.end {
                    self.row_sel = false;
                }
                scroll_to_sel = true;
            } else if t.delete {
                if self.row_sel {
                    menu_act = Some(2);
                } else if editable && self.sel.0 < nrows {
                    commit = Some((self.sel.0, self.sel.1, String::new(), false));
                }
            } else if editable && (t.f2 || t.backspace || !t.typed.is_empty()) {
                let (r, c) = self.sel;
                let ini = if !t.typed.is_empty() {
                    t.typed
                } else if t.backspace {
                    String::new()
                } else if r < nrows {
                    data.rows[r][c].clone()
                } else {
                    String::new()
                };
                self.row_sel = false;
                self.edit = Some(Edicao { row: r, col: c, texto: ini, foco: true });
                scroll_to_sel = true;
            }
        }

        let sel = self.sel;
        let row_sel = self.row_sel;
        let mut hit: Option<(usize, usize, bool)> = None;
        let mut gutter_hit: Option<usize> = None;
        {
            let cols = &data.cols;
            let rows = &data.rows;
            egui::ScrollArea::both().auto_shrink([false, false]).max_height(h).show_viewport(ui, |ui, vp| {
                let width = data_w.max(ui.available_width());
                let grid_h = hh + rh * nlin as f32;
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, grid_h.max(h)), Sense { click: true, drag: false, focusable: false });
                let p = ui.painter().clone();
                let o = rect.min;
                let ty = o.y + vp.min.y; // topo visível (cabeçalho fixo)
                let line = Stroke::new(1.0_f32, GRID);
                let gb = o.y + grid_h;
                p.rect_filled(rect, 0.0, Color32::WHITE);
                p.rect_filled(Rect::from_min_size(o, Vec2::new(sw, gb - o.y)), 0.0, rgb(0xE9E9E9));
                // linhas verticais das colunas
                let mut x = o.x + sw;
                for (_, w) in cols.iter() {
                    x += *w;
                    p.line_segment([Pos2::new(x, o.y), Pos2::new(x, gb)], line);
                }
                // só as linhas visíveis
                let first = (((vp.min.y - hh) / rh).floor().max(0.0)) as usize;
                let last = ((((vp.max.y - hh) / rh).ceil().max(0.0)) as usize + 1).min(nlin);
                for ri in first..last {
                    let y = o.y + hh + rh * ri as f32;
                    if ri == sel.0 {
                        let cor = if row_sel { rgb(0xF2B84B) } else { rgb(0xC9D7EA) };
                        p.rect_filled(Rect::from_min_size(Pos2::new(o.x, y), Vec2::new(sw, rh)), 0.0, cor);
                        if row_sel {
                            p.rect_filled(Rect::from_min_size(Pos2::new(o.x + sw, y), Vec2::new(data_w - sw - extra, rh)), 0.0, rgb(0xDCE8F7));
                        }
                        tri(&p, [Pos2::new(o.x + 7.0, y + 5.0), Pos2::new(o.x + 7.0, y + rh - 5.0), Pos2::new(o.x + 14.0, y + rh / 2.0)], rgb(0x3A3A3A));
                    }
                    p.line_segment([Pos2::new(o.x, y + rh), Pos2::new(o.x + data_w, y + rh)], line);
                    if ri == nrows {
                        p.text(Pos2::new(o.x + sw / 2.0, y + rh / 2.0 + 2.0), Align2::CENTER_CENTER, "*", FontId::proportional(14.0), rgb(0x555555));
                        p.text(Pos2::new(o.x + sw + 6.0, y + rh / 2.0), Align2::LEFT_CENTER, "(Novo)", FontId::proportional(13.0), rgb(0x8A8A8A));
                        continue;
                    }
                    let mut cx = o.x + sw;
                    for (ci, (_, w)) in cols.iter().enumerate() {
                        let cell = Rect::from_min_size(Pos2::new(cx, y), Vec2::new(*w, rh));
                        p.with_clip_rect(cell.intersect(p.clip_rect())).text(
                            Pos2::new(cx + 6.0, y + rh / 2.0),
                            Align2::LEFT_CENTER,
                            &rows[ri][ci],
                            FontId::proportional(13.0),
                            Color32::BLACK,
                        );
                        cx += *w;
                    }
                }
                // célula ativa
                if ncols > 0 && nlin > 0 {
                    let cx = o.x + sw + cols.iter().take(sel.1).map(|c| c.1).sum::<f32>();
                    let cy = o.y + hh + rh * sel.0 as f32;
                    let cell = Rect::from_min_size(Pos2::new(cx, cy), Vec2::new(cols[sel.1].1, rh));
                    if scroll_to_sel {
                        ui.scroll_to_rect(Rect::from_min_max(cell.min - Vec2::new(0.0, hh), cell.max), None);
                    }
                    if !row_sel {
                        p.rect_stroke(cell.shrink(0.5), 0.0, Stroke::new(2.0_f32, Color32::BLACK));
                    }
                }
                // editor da célula
                let mut fin: Option<(usize, usize, String, bool, bool)> = None;
                if let Some(ed) = self.edit.as_mut() {
                    if ed.row <= nrows && ed.col < ncols {
                        let ex = o.x + sw + cols.iter().take(ed.col).map(|c| c.1).sum::<f32>();
                        let ecell = Rect::from_min_size(Pos2::new(ex, o.y + hh + rh * ed.row as f32), Vec2::new(cols[ed.col].1.max(90.0), rh));
                        p.rect_filled(ecell, 0.0, Color32::WHITE);
                        let te = egui::TextEdit::singleline(&mut ed.texto)
                            .id(egui::Id::new("editor_celula"))
                            .frame(false)
                            .font(FontId::proportional(13.0));
                        let r = ui.put(ecell, te);
                        if std::env::var_os("ACCESSQL_DEBUG").is_some() {
                            // diagnóstico: que teclas chegam ao egui e se o editor tem o foco
                            let teclas = ui.input(|i| {
                                i.events
                                    .iter()
                                    .filter_map(|e| match e {
                                        egui::Event::Key { key, pressed: true, .. } => Some(format!("{:?}", key)),
                                        _ => None,
                                    })
                                    .collect::<Vec<_>>()
                                    .join(",")
                            });
                            if !teclas.is_empty() {
                                self.msg = format!("[depuração] teclas={} foco={} texto={:?}", teclas, r.has_focus(), ed.texto);
                            }
                        }
                        if ed.foco {
                            r.request_focus();
                            ed.foco = false;
                        }
                        p.rect_stroke(ecell.shrink(0.5), 0.0, Stroke::new(2.0_f32, Color32::BLACK));
                        let (enter, tab, esc) = ui.input(|i| {
                            (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Tab), i.key_pressed(egui::Key::Escape))
                        });
                        if esc {
                            fin = Some((ed.row, ed.col, String::new(), false, true));
                        } else if r.lost_focus() {
                            fin = Some((ed.row, ed.col, ed.texto.clone(), enter || tab, false));
                        }
                    } else {
                        fin = Some((0, 0, String::new(), false, true));
                    }
                }
                if let Some((r, c, t, adv, esc)) = fin {
                    self.edit = None;
                    if !esc {
                        commit = Some((r, c, t, adv));
                    }
                }
                // cabeçalho (fixo no topo da área visível)
                p.rect_filled(Rect::from_min_size(Pos2::new(o.x, ty), Vec2::new(data_w, hh)), 0.0, rgb(0xF2F2F2));
                p.rect_filled(Rect::from_min_size(Pos2::new(o.x, ty), Vec2::new(sw, hh)), 0.0, rgb(0xE9E9E9));
                let mut x = o.x + sw;
                for (label, w) in cols.iter() {
                    let w = *w;
                    p.text(Pos2::new(x + 6.0, ty + hh / 2.0), Align2::LEFT_CENTER, label, FontId::proportional(13.0), Color32::BLACK);
                    down_arrow(&p, Pos2::new(x + w - 11.0, ty + hh / 2.0), rgb(0x6A6A6A));
                    x += w;
                    p.line_segment([Pos2::new(x, ty), Pos2::new(x, ty + hh)], line);
                }
                if editable {
                    p.text(Pos2::new(x + 6.0, ty + hh / 2.0), Align2::LEFT_CENTER, "Clique para Adicionar", FontId::proportional(13.0), rgb(0x5A7FB5));
                    down_arrow(&p, Pos2::new(x + extra - 11.0, ty + hh / 2.0), rgb(0x5A7FB5));
                }
                p.line_segment([Pos2::new(o.x + sw, ty), Pos2::new(o.x + sw, ty + hh)], Stroke::new(1.0_f32, rgb(0xB4B4B4)));
                p.line_segment([Pos2::new(o.x, ty + hh), Pos2::new(o.x + data_w, ty + hh)], Stroke::new(1.0_f32, rgb(0xB4B4B4)));
                // cliques: células e seletor de registo (coluna à esquerda)
                let localizar = |pos: Pos2| -> Option<(usize, Option<usize>)> {
                    if pos.y <= ty + hh {
                        return None;
                    }
                    let rel = pos - o;
                    let ri = ((rel.y - hh) / rh) as usize;
                    if ri >= nlin {
                        return None;
                    }
                    if rel.x <= sw {
                        return Some((ri, None));
                    }
                    let mut acc = sw;
                    for (ci, (_, w)) in cols.iter().enumerate() {
                        if rel.x < acc + *w {
                            return Some((ri, Some(ci)));
                        }
                        acc += *w;
                    }
                    None
                };
                if resp.clicked() || resp.double_clicked() || resp.secondary_clicked() {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        match localizar(pos) {
                            Some((ri, Some(ci))) => hit = Some((ri, ci, resp.double_clicked())),
                            Some((ri, None)) => gutter_hit = Some(ri),
                            None => {}
                        }
                    }
                }
                if editable {
                    resp.context_menu(|ui| {
                        if ui.button("Novo Registo").clicked() {
                            menu_act = Some(1);
                            ui.close_menu();
                        }
                        if ui.button("Eliminar Registo").clicked() {
                            menu_act = Some(2);
                            ui.close_menu();
                        }
                    });
                }
            });
        }
        if let Some(ri) = gutter_hit {
            self.sel.0 = ri;
            if ri < nrows {
                self.row_sel = true;
                self.msg = "Registo selecionado: prima Delete ou use Registos → Eliminar".into();
            } else {
                self.row_sel = false;
            }
        }
        if let Some((ri, ci, dbl)) = hit {
            self.row_sel = false;
            self.sel = (ri, ci);
            if dbl && editable {
                let ini = if ri < nrows { data.rows[ri][ci].clone() } else { String::new() };
                self.edit = Some(Edicao { row: ri, col: ci, texto: ini, foco: true });
            }
        }
        if let Some((r, c, t, adv)) = commit {
            self.gravar_celula(&mut data, &tabela, r, c, t);
            let n = data.rows.len();
            if adv {
                self.sel = if c + 1 < ncols { (r, c + 1) } else { ((r + 1).min(n), 0) };
                self.scroll_sel = true;
            }
        }
        let total = data.rows.len();
        self.docs[self.active].data = data;
        // só agora os dados voltaram ao documento
        match menu_act {
            Some(1) => self.novo_registo(),
            Some(2) => self.pedir_eliminar(),
            _ => {}
        }
        self.record_bar(ui, total, novo == 1);
    }

    /// Grava uma célula (UPDATE) ou cria um registo novo (INSERT) na linha `(Novo)`.
    fn gravar_celula(&mut self, data: &mut TableData, tabela: &str, r: usize, c: usize, texto: String) {
        let Some(db) = self.db.as_ref() else { return };
        let col = data.cols[c].0.clone();
        if r < data.rows.len() {
            if data.rows[r][c] == texto {
                return;
            }
            let rid = data.rowids[r];
            if let Err(e) = db.update_cell(tabela, &col, rid, &texto) {
                self.msg = format!("Não foi possível gravar: {}", e);
                return;
            }
            match db.fetch_row(tabela, rid) {
                Ok(Some(row)) => data.rows[r] = row,
                _ => {
                    // a chave mudou (por exemplo, o campo ID): voltar a ler a tabela
                    if let Ok(d) = db.load(tabela, true) {
                        *data = d;
                    }
                }
            }
        } else {
            if texto.is_empty() {
                return;
            }
            match db.insert_cell(tabela, &col, &texto) {
                Ok(rid) => match db.fetch_row(tabela, rid) {
                    Ok(Some(row)) => {
                        data.rows.push(row);
                        data.rowids.push(rid);
                    }
                    _ => {
                        if let Ok(d) = db.load(tabela, true) {
                            *data = d;
                        }
                    }
                },
                Err(e) => self.msg = format!("Não foi possível criar o registo: {}", e),
            }
        }
    }

    /// Vai para a linha `(Novo)` da folha de dados ativa.
    pub(crate) fn novo_registo(&mut self) {
        match self.docs.get(self.active) {
            Some(d) if d.data.editable => {
                self.sel = (d.data.rows.len(), 0);
                self.scroll_sel = true;
            }
            _ => self.msg = "Esta vista não permite criar registos".into(),
        }
    }

    /// Pede confirmação antes de eliminar o registo selecionado.
    pub(crate) fn pedir_eliminar(&mut self) {
        match self.docs.get(self.active) {
            Some(d) if d.data.editable && self.sel.0 < d.data.rows.len() => self.confirm_del = Some(self.sel.0),
            Some(d) if d.data.editable => self.msg = "Selecione um registo para eliminar".into(),
            _ => self.msg = "Esta vista não permite eliminar registos".into(),
        }
    }

    pub(crate) fn dialogo_eliminar(&mut self, ctx: &egui::Context) {
        let Some(row) = self.confirm_del else { return };
        let mut resp: Option<bool> = None;
        egui::Window::new("AccesSQL")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Vai eliminar 1 registo.");
                ui.label("Não poderá anular esta operação. Quer eliminar o registo?");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Sim").clicked() {
                        resp = Some(true);
                    }
                    if ui.button("Não").clicked() {
                        resp = Some(false);
                    }
                });
            });
        if resp.is_none() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            resp = Some(true);
        }
        match resp {
            Some(true) => {
                self.confirm_del = None;
                self.eliminar_registo(row);
            }
            Some(false) => self.confirm_del = None,
            None => {}
        }
    }

    fn eliminar_registo(&mut self, row: usize) {
        let Some(doc) = self.docs.get(self.active) else { return };
        if row >= doc.data.rows.len() {
            return;
        }
        let nome = doc.name.clone();
        let rid = doc.data.rowids[row];
        match self.db.as_ref().map(|d| d.delete_row(&nome, rid)) {
            Some(Ok(())) => {
                let d = &mut self.docs[self.active].data;
                d.rows.remove(row);
                d.rowids.remove(row);
                self.sel.0 = row.min(d.rows.len());
                self.msg = "Registo eliminado".into();
            }
            Some(Err(e)) => self.msg = format!("Não foi possível eliminar: {}", e),
            None => {}
        }
    }

    /// Volta a ler a tabela ou consulta ativa (e o painel de navegação).
    pub(crate) fn recarregar_doc(&mut self) {
        self.refresh_nav();
        let Some(doc) = self.docs.get(self.active) else { return };
        let (nome, kind) = (doc.name.clone(), doc.kind);
        if kind == ObjKind::Sql || kind == ObjKind::Design {
            return;
        }
        match self.db.as_ref().map(|d| d.load(&nome, kind == ObjKind::Table)) {
            Some(Ok(d)) => {
                self.docs[self.active].data = d;
                self.edit = None;
                self.msg = format!("«{}» atualizada", nome);
            }
            Some(Err(e)) => self.msg = format!("Erro ao atualizar «{}»: {}", nome, e),
            None => {}
        }
    }
    pub(crate) fn record_bar(&mut self, ui: &mut Ui, total: usize, novo: bool) {
        // última linha selecionável: a linha «(Novo)» só existe nas tabelas
        let max = if novo { total } else { total.saturating_sub(1) };
        Frame::none().fill(rgb(0xF0F0F0)).inner_margin(Margin::symmetric(6.0, 3.0)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                ui.label(RichText::new("Registo:").size(12.0));
                if nav_arrow(ui, 0).clicked() {
                    self.sel.0 = 0;
                }
                if nav_arrow(ui, 1).clicked() {
                    self.sel.0 = self.sel.0.saturating_sub(1);
                }
                Frame::none()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.0_f32, rgb(0xA0A0A0)))
                    .inner_margin(Margin::symmetric(8.0, 1.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(format!("{} de {}", if total == 0 { 0 } else { self.sel.0.min(total - 1) + 1 }, total)).size(12.0));
                    });
                if nav_arrow(ui, 2).clicked() {
                    self.sel.0 = (self.sel.0 + 1).min(max);
                }
                if nav_arrow(ui, 3).clicked() {
                    self.sel.0 = total.saturating_sub(1);
                }
                if nav_arrow(ui, 4).clicked() && novo {
                    self.sel.0 = total;
                }
                ui.add_space(8.0);
                Frame::none().stroke(Stroke::new(1.0_f32, rgb(0xC0C0C0))).inner_margin(Margin::symmetric(8.0, 1.0)).show(ui, |ui| {
                    ui.label(RichText::new("Sem Filtro").size(12.0).color(rgb(0x555555)));
                });
                ui.add_space(4.0);
                ui.add(egui::TextEdit::singleline(&mut self.rec_search).id(egui::Id::new("rec_search")).hint_text("Procurar").desired_width(170.0));
            });
        });
    }
}
