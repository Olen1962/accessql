use eframe::egui::{self, Align, Align2, Color32, FontId, Frame, Layout, Margin, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use crate::{estado::*, icones::*, tema::*, widgets::*};
impl AccessApp {
// ───────────── Painel de navegação ─────────────
    pub(crate) fn nav_pane(&mut self, ctx: &egui::Context) {
        let mut to_open: Option<(ObjKind, String)> = None;
        let mut to_sql: Option<String> = None;
        egui::SidePanel::left("nav")
            .resizable(true)
            .default_width(230.0)
            .width_range(170.0..=420.0)
            .frame(Frame::none().fill(NAV_BG))
            .show(ctx, |ui| {
                let (r, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 27.0), Sense::click());
                let p = ui.painter().clone();
                p.rect_filled(r, 0.0, NAV_HEAD);
                p.text(Pos2::new(r.left() + 8.0, r.center().y), Align2::LEFT_CENTER, "Todos os Objetos", FontId::proportional(13.0), TEXT);
                down_arrow(&p, Pos2::new(r.right() - 14.0, r.center().y), TEXT);
                self.act(&resp, "Categoria", Some("navcat"));
                Frame::none().inner_margin(Margin::same(4.0)).show(ui, |ui| {
                    let w = ui.available_width();
                    ui.add(egui::TextEdit::singleline(&mut self.nav_filter).id(egui::Id::new("nav_filter")).hint_text("Procurar...").desired_width(w));
                });
                let nav = self.nav.clone();
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    if self.db.is_none() {
                        ui.add_space(8.0);
                        ui.label(RichText::new("  Nenhuma base de dados aberta").size(12.0).color(rgb(0x5A6B85)));
                    }
                    for (gi, grp) in nav.iter().enumerate() {
                        let (gname, kind, items) = (grp.name, grp.kind, &grp.items);
                        let (hr, hresp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 22.0), Sense::click());
                        let p = ui.painter().clone();
                        p.rect_filled(hr, 0.0, if hresp.hovered() { rgb(0x8FADD8) } else { NAV_GROUP });
                        let c = Pos2::new(hr.left() + 12.0, hr.center().y);
                        if self.nav_open[gi] {
                            tri(&p, [Pos2::new(c.x - 4.0, c.y - 2.0), Pos2::new(c.x + 4.0, c.y - 2.0), Pos2::new(c.x, c.y + 3.0)], Color32::WHITE);
                        } else {
                            tri(&p, [Pos2::new(c.x - 2.0, c.y - 4.0), Pos2::new(c.x - 2.0, c.y + 4.0), Pos2::new(c.x + 3.0, c.y)], Color32::WHITE);
                        }
                        p.text(Pos2::new(hr.left() + 24.0, hr.center().y), Align2::LEFT_CENTER, gname, FontId::proportional(13.0), Color32::WHITE);
                        if hresp.clicked() {
                            self.nav_open[gi] = !self.nav_open[gi];
                        }
                        if !self.nav_open[gi] {
                            continue;
                        }
                        for (ii, name) in items.iter().enumerate() {
                            let f = self.nav_filter.to_lowercase();
                            if !f.is_empty() && !name.to_lowercase().contains(&f) {
                                continue;
                            }
                            let (rr, rresp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 21.0), Sense::click());
                            let p = ui.painter().clone();
                            if self.nav_sel == Some((gi, ii)) {
                                p.rect_filled(rr.shrink2(Vec2::new(2.0, 0.0)), 2.0, rgb(0xFFE8A6));
                                p.rect_stroke(rr.shrink2(Vec2::new(2.0, 0.0)), 2.0, Stroke::new(1.0_f32, HOVER_LINE));
                            } else if rresp.hovered() {
                                p.rect_filled(rr.shrink2(Vec2::new(2.0, 0.0)), 2.0, rgb(0xE3ECF8));
                            }
                            draw_icon(&p, Rect::from_center_size(Pos2::new(rr.left() + 18.0, rr.center().y), Vec2::splat(16.0)), kind.icon());
                            p.text(Pos2::new(rr.left() + 32.0, rr.center().y), Align2::LEFT_CENTER, name, FontId::proportional(13.0), Color32::BLACK);
                            if rresp.clicked() {
                                self.nav_sel = Some((gi, ii));
                            }
                            if rresp.double_clicked() {
                                to_open = Some((kind, name.to_string()));
                            }
                            rresp.context_menu(|ui| {
                                if ui.button("Abrir").clicked() {
                                    to_open = Some((kind, name.to_string()));
                                    ui.close_menu();
                                }
                                if kind == ObjKind::Query && ui.button("Vista SQL").clicked() {
                                    to_sql = Some(name.to_string());
                                    ui.close_menu();
                                }
                            });
                        }
                    }
                });
                let full = ui.max_rect();
                ui.painter().line_segment([full.right_top(), full.right_bottom()], Stroke::new(1.0_f32, RIBBON_LINE));
            });
        if let Some((k, n)) = to_open {
            self.open_doc(k, n);
        }
        if let Some(n) = to_sql {
            self.abrir_vista_sql(&n);
        }
    }
}
