use eframe::egui::{self, Align, Align2, Color32, FontId, Frame, Layout, Margin, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use crate::{dados_exemplo::*, estado::*, icones::*, tema::*, widgets::*};

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

                        p.rect_stroke(rect, top, Stroke::new(1.0, RIBBON_LINE));

                    } else {

                        p.rect_filled(rect.shrink2(Vec2::new(0.0, 2.0)), top, if resp.hovered() { rgb(0xE6EEF9) } else { rgb(0xD3DFF0) });

                    }

                    draw_icon(&p, Rect::from_center_size(Pos2::new(rect.left() + 14.0, rect.center().y), Vec2::splat(14.0)), d.kind.icon());

                    p.text(Pos2::new(rect.left() + 26.0, rect.center().y), Align2::LEFT_CENTER, &d.name, FontId::proportional(12.5), Color32::BLACK);

                    if sel {

                        let cr = Rect::from_center_size(Pos2::new(rect.right() - 13.0, rect.center().y), Vec2::splat(14.0));

                        let cresp = ui.interact(cr, ui.id().with(("close", i)), Sense::click());

                        let col = if cresp.hovered() { ACCENT } else { rgb(0x6A7A94) };

                        p.line_segment([cr.center() + Vec2::new(-3.5, -3.5), cr.center() + Vec2::new(3.5, 3.5)], Stroke::new(1.5, col));

                        p.line_segment([cr.center() + Vec2::new(-3.5, 3.5), cr.center() + Vec2::new(3.5, -3.5)], Stroke::new(1.5, col));

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

                self.active = self.active.saturating_sub(1);

                return;

            }

            if let Some(i) = switch {

                self.active = i;

                self.sel = (0, 1);

            }

            let area = ui.available_rect_before_wrap();

            ui.painter().rect_filled(area, 0.0, Color32::WHITE);

            ui.painter().rect_stroke(area, 0.0, Stroke::new(1.0, RIBBON_LINE));

            let kind = self.docs[self.active].kind;

            let name = self.docs[self.active].name.clone();

            match kind {

                ObjKind::Table | ObjKind::Query => self.datasheet(ui, &name),

                ObjKind::Form => self.form_view(ui, &name),

                ObjKind::Report => self.report_view(ui, &name),

            }

        });

    }



    pub(crate) fn datasheet(&mut self, ui: &mut Ui, name: &str) {

        let (cols, rows) = dataset(name);

        let (rh, hh, sw) = (21.0_f32, 24.0_f32, 22.0_f32);

        let extra = 150.0_f32;

        let data_w = sw + cols.iter().map(|c| c.1).sum::<f32>() + extra;

        self.sel.0 = self.sel.0.min(rows.len());

        self.sel.1 = self.sel.1.min(cols.len() - 1);

        let h = (ui.available_height() - 27.0).max(60.0);

        egui::ScrollArea::both().auto_shrink([false, false]).max_height(h).show(ui, |ui| {

            let width = data_w.max(ui.available_width());

            let grid_bottom_rel = hh + rh * (rows.len() as f32 + 1.0);

            let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, grid_bottom_rel.max(h)), Sense::click());

            let p = ui.painter().clone();

            let o = rect.min;

            let line = Stroke::new(1.0, GRID);

            let gb = o.y + grid_bottom_rel;

            p.rect_filled(rect, 0.0, Color32::WHITE);

            // cabeçalho

            p.rect_filled(Rect::from_min_size(o, Vec2::new(data_w, hh)), 0.0, rgb(0xF2F2F2));

            p.rect_filled(Rect::from_min_size(o, Vec2::new(sw, gb - o.y)), 0.0, rgb(0xE9E9E9));

            let mut x = o.x + sw;

            for &(label, w) in &cols {

                p.text(Pos2::new(x + 6.0, o.y + hh / 2.0), Align2::LEFT_CENTER, label, FontId::proportional(13.0), Color32::BLACK);

                down_arrow(&p, Pos2::new(x + w - 11.0, o.y + hh / 2.0), rgb(0x6A6A6A));

                x += w;

                p.line_segment([Pos2::new(x, o.y), Pos2::new(x, gb)], line);

            }

            p.text(Pos2::new(x + 6.0, o.y + hh / 2.0), Align2::LEFT_CENTER, "Clique para Adicionar", FontId::proportional(13.0), rgb(0x5A7FB5));

            down_arrow(&p, Pos2::new(x + extra - 11.0, o.y + hh / 2.0), rgb(0x5A7FB5));

            p.line_segment([Pos2::new(o.x + sw, o.y), Pos2::new(o.x + sw, gb)], Stroke::new(1.0, rgb(0xB4B4B4)));

            p.line_segment([Pos2::new(o.x, o.y + hh), Pos2::new(o.x + data_w, o.y + hh)], Stroke::new(1.0, rgb(0xB4B4B4)));

            // linhas

            for ri in 0..=rows.len() {

                let y = o.y + hh + rh * ri as f32;

                if ri == self.sel.0 {

                    p.rect_filled(Rect::from_min_size(Pos2::new(o.x, y), Vec2::new(sw, rh)), 0.0, rgb(0xC9D7EA));

                    tri(&p, [Pos2::new(o.x + 7.0, y + 5.0), Pos2::new(o.x + 7.0, y + rh - 5.0), Pos2::new(o.x + 14.0, y + rh / 2.0)], rgb(0x3A3A3A));

                }

                p.line_segment([Pos2::new(o.x, y + rh), Pos2::new(o.x + data_w, y + rh)], line);

                if ri == rows.len() {

                    p.text(Pos2::new(o.x + sw / 2.0, y + rh / 2.0 + 2.0), Align2::CENTER_CENTER, "*", FontId::proportional(14.0), rgb(0x555555));

                    p.text(Pos2::new(o.x + sw + 6.0, y + rh / 2.0), Align2::LEFT_CENTER, "(Novo)", FontId::proportional(13.0), rgb(0x8A8A8A));

                    continue;

                }

                let mut cx = o.x + sw;

                for (ci, &(_, w)) in cols.iter().enumerate() {

                    p.text(Pos2::new(cx + 6.0, y + rh / 2.0), Align2::LEFT_CENTER, rows[ri][ci], FontId::proportional(13.0), Color32::BLACK);

                    cx += w;

                }

            }

            // célula ativa

            let mut cx = o.x + sw;

            for (ci, &(_, w)) in cols.iter().enumerate() {

                if ci == self.sel.1 {

                    let y = o.y + hh + rh * self.sel.0 as f32;

                    p.rect_stroke(Rect::from_min_size(Pos2::new(cx, y), Vec2::new(w, rh)).shrink(0.5), 0.0, Stroke::new(2.0, Color32::BLACK));

                }

                cx += w;

            }

            if resp.clicked() {

                if let Some(pos) = resp.interact_pointer_pos() {

                    let rel = pos - o;

                    if rel.y > hh && rel.x > sw {

                        let ri = ((rel.y - hh) / rh) as usize;

                        if ri <= rows.len() {

                            self.sel.0 = ri;

                            let mut acc = sw;

                            for (ci, &(_, w)) in cols.iter().enumerate() {

                                if rel.x < acc + w {

                                    self.sel.1 = ci;

                                    break;

                                }

                                acc += w;

                            }

                        }

                    }

                }

            }

        });

        self.record_bar(ui, rows.len());

    }



    pub(crate) fn record_bar(&mut self, ui: &mut Ui, total: usize) {

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

                    .stroke(Stroke::new(1.0, rgb(0xA0A0A0)))

                    .inner_margin(Margin::symmetric(8.0, 1.0))

                    .show(ui, |ui| {

                        ui.label(RichText::new(format!("{} de {}", self.sel.0.min(total - 1) + 1, total)).size(12.0));

                    });

                if nav_arrow(ui, 2).clicked() {

                    self.sel.0 = (self.sel.0 + 1).min(total);

                }

                if nav_arrow(ui, 3).clicked() {

                    self.sel.0 = total - 1;

                }

                if nav_arrow(ui, 4).clicked() {

                    self.sel.0 = total;

                }

                ui.add_space(8.0);

                Frame::none().stroke(Stroke::new(1.0, rgb(0xC0C0C0))).inner_margin(Margin::symmetric(8.0, 1.0)).show(ui, |ui| {

                    ui.label(RichText::new("Sem Filtro").size(12.0).color(rgb(0x555555)));

                });

                ui.add_space(4.0);

                ui.add(egui::TextEdit::singleline(&mut self.rec_search).hint_text("Procurar").desired_width(170.0));

            });

        });

    }



    pub(crate) fn form_view(&mut self, ui: &mut Ui, name: &str) {

        let (cols, rows) = dataset(name);

        let h = (ui.available_height() - 27.0).max(60.0);

        egui::ScrollArea::vertical().auto_shrink([false, false]).max_height(h).show(ui, |ui| {

            Frame::none().fill(rgb(0xDCE6F4)).inner_margin(Margin::symmetric(16.0, 10.0)).show(ui, |ui| {

                ui.set_width(ui.available_width());

                ui.label(RichText::new(name).size(22.0).color(BLUE_TITLE));

            });

            let r = self.sel.0.min(rows.len() - 1);

            Frame::none().inner_margin(Margin::same(24.0)).show(ui, |ui| {

                egui::Grid::new("formgrid").num_columns(2).spacing([14.0, 10.0]).show(ui, |ui| {

                    for (i, &(label, _)) in cols.iter().enumerate() {

                        ui.label(RichText::new(label).size(13.0));

                        let mut s = rows[r][i].to_string();

                        ui.add(egui::TextEdit::singleline(&mut s).desired_width(260.0));

                        ui.end_row();

                    }

                });

            });

        });

        self.record_bar(ui, rows.len());

    }



    pub(crate) fn report_view(&self, ui: &mut Ui, name: &str) {

        let (cols, rows) = dataset("Clientes");

        let rect = ui.available_rect_before_wrap();

        let p = ui.painter().clone();

        p.rect_filled(rect, 0.0, rgb(0x8C98A8));

        let pw = 600.0_f32.min(rect.width() - 40.0);

        let page = Rect::from_min_size(Pos2::new(rect.center().x - pw / 2.0, rect.top() + 16.0), Vec2::new(pw, rect.height() - 32.0));

        p.rect_filled(page.translate(Vec2::new(3.0, 3.0)), 0.0, rgb(0x6E7A8A));

        p.rect_filled(page, 0.0, Color32::WHITE);

        p.text(page.min + Vec2::new(30.0, 28.0), Align2::LEFT_TOP, name, FontId::proportional(22.0), BLUE_TITLE);

        let n = cols.len().min(4);

        let colw = (pw - 60.0) / n as f32;

        let mut y = page.top() + 84.0;

        for (ci, &(label, _)) in cols.iter().take(n).enumerate() {

            p.text(Pos2::new(page.left() + 30.0 + colw * ci as f32, y), Align2::LEFT_TOP, label, FontId::proportional(13.0), BLUE_TITLE);

        }

        y += 22.0;

        p.line_segment([Pos2::new(page.left() + 30.0, y), Pos2::new(page.right() - 30.0, y)], Stroke::new(1.5, BLUE_TITLE));

        y += 6.0;

        for row in &rows {

            for ci in 0..n {

                p.text(Pos2::new(page.left() + 30.0 + colw * ci as f32, y), Align2::LEFT_TOP, row[ci], FontId::proportional(12.5), Color32::BLACK);

            }

            y += 20.0;

        }

        ui.allocate_rect(rect, Sense::hover());

    }

}
