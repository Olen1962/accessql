use eframe::egui::{self, Align, Align2, Color32, FontId, Frame, Layout, Margin, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use crate::{dados_exemplo::*, estado::*, icones::*, tema::*, widgets::*};

impl AccessApp {
// ───────────── Menus pendentes (dropdowns do ribbon / painel) ─────────────

    pub(crate) fn show_dropdown(&mut self, ctx: &egui::Context) {

        let Some((key, btn)) = self.dropdown else { return };

        let items = dd_items(key);

        let mut picked: Option<&'static str> = None;

        let area = egui::Area::new(egui::Id::new("dd_area"))

            .order(egui::Order::Foreground)

            .fixed_pos(btn.left_bottom() + Vec2::new(0.0, 1.0))

            .show(ctx, |ui| {

                Frame::popup(ui.style())

                    .fill(Color32::WHITE)

                    .stroke(Stroke::new(1.0, rgb(0x8FA9CF)))

                    .inner_margin(Margin::same(2.0))

                    .show(ui, |ui| {

                        ui.set_min_width(200.0);

                        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {

                            for it in items {

                                if ui.selectable_label(false, RichText::new(*it).size(12.5)).clicked() {

                                    picked = Some(*it);

                                }

                            }

                        });

                    });

            });

        if let Some(it) = picked {

            self.msg = format!("«{}» – apenas apresentação visual", it);

            self.dropdown = None;

            return;

        }

        if ctx.input(|i| i.pointer.any_pressed()) {

            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {

                if !area.response.rect.contains(pos) && !btn.contains(pos) {

                    self.dropdown = None;

                }

            }

        }

    }

}
