use eframe::egui::{self, Align, Align2, Color32, FontId, Frame, Layout, Margin, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use crate::{dados_exemplo::*, estado::*, icones::*, tema::*, widgets::*};

impl AccessApp {
// ───────────── Barra de estado ─────────────

    pub(crate) fn status_bar(&mut self, ctx: &egui::Context) {

        let view = if self.backstage {

            "Pronto"

        } else {

            match self.docs.get(self.active).map(|d| d.kind) {

                Some(ObjKind::Table) | Some(ObjKind::Query) => "Vista Folha de Dados",

                Some(ObjKind::Form) => "Vista Formulário",

                Some(ObjKind::Report) => "Pré-visualização",

                None => "Pronto",

            }

        };

        egui::TopBottomPanel::bottom("status")

            .exact_height(23.0)

            .frame(Frame::none().fill(STATUS_BG).inner_margin(Margin::symmetric(8.0, 0.0)))

            .show(ctx, |ui| {

                let full = ui.max_rect();

                ui.painter().line_segment([full.left_top(), full.right_top()], Stroke::new(1.0, RIBBON_LINE));

                ui.horizontal_centered(|ui| {

                    ui.label(RichText::new(view).size(12.0).color(TEXT));

                    ui.add_space(20.0);

                    ui.label(RichText::new(&self.msg).size(11.5).color(rgb(0x5A6B85)));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {

                        small_btn(ui, Icon::Badge("E", C_GRAY), "", false).on_hover_text("Vista Estrutura");

                        small_btn(ui, Icon::Table, "", false).on_hover_text("Vista Folha de Dados");

                        ui.add_space(10.0);

                        ui.label(RichText::new("Bloq Num").size(12.0).color(rgb(0x5A6B85)));

                    });

                });

            });

    }

}
