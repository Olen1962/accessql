use eframe::egui::{self, Align, Align2, Color32, FontId, Frame, Layout, Margin, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use crate::{dados_exemplo::*, estado::*, icones::*, tema::*, widgets::*};

impl AccessApp {
// ───────────── Backstage (separador Ficheiro) ─────────────

    pub(crate) fn bs_item(ui: &mut Ui, text: &str, selected: bool) -> Response {

        let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 29.0), Sense::click());

        let p = ui.painter();

        if selected {

            p.rect_filled(rect, 0.0, rgb(0xFFE08A));

        } else if resp.hovered() {

            p.rect_filled(rect, 0.0, rgb(0xE3ECF8));

        }

        p.text(Pos2::new(rect.left() + 22.0, rect.center().y), Align2::LEFT_CENTER, text, FontId::proportional(13.5), TEXT);

        resp

    }



    pub(crate) fn bs_action(&mut self, ctx: &egui::Context, name: &str) {

        match name {

            "Abrir" => {

                if let Some(path) = rfd::FileDialog::new()

                    .add_filter("Bases de dados (accdb, sqlite, db)", &["accdb", "mdb", "sqlite", "sqlite3", "db"])

                    .pick_file()

                {

                    self.db_path = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();

                    self.msg = format!("Selecionado: {}", path.display());

                    self.backstage = false;

                }

            }

            "Sair" => ctx.send_viewport_cmd(egui::ViewportCommand::Close),

            _ => self.msg = format!("«{}» – apenas apresentação visual", name),

        }

    }



    pub(crate) fn backstage_ui(&mut self, ctx: &egui::Context) {

        const ACTIONS: [&str; 5] = ["Guardar", "Guardar Objeto Como", "Guardar Base de Dados Como", "Abrir", "Fechar Base de Dados"];

        const PAGES: [&str; 5] = ["Informações", "Recente", "Novo", "Imprimir", "Ajuda"];

        egui::SidePanel::left("bs")

            .exact_width(215.0)

            .resizable(false)

            .frame(Frame::none().fill(rgb(0xF4F7FB)))

            .show(ctx, |ui| {

                ui.spacing_mut().item_spacing = Vec2::ZERO;

                ui.add_space(8.0);

                let (r, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0), Sense::click());

                let c = Pos2::new(r.left() + 32.0, r.center().y);

                let p = ui.painter();

                p.circle_filled(c, 13.0, if resp.hovered() { rgb(0x7FA6DE) } else { rgb(0x5B8BC9) });

                let w = Stroke::new(2.0_f32, Color32::WHITE);

                p.line_segment([c + Vec2::new(6.0, 0.0), c + Vec2::new(-6.0, 0.0)], w);

                p.line_segment([c + Vec2::new(-6.0, 0.0), c + Vec2::new(-1.0, -5.0)], w);

                p.line_segment([c + Vec2::new(-6.0, 0.0), c + Vec2::new(-1.0, 5.0)], w);

                if resp.clicked() {

                    self.backstage = false;

                }

                ui.add_space(6.0);

                for a in ACTIONS {

                    if Self::bs_item(ui, a, false).clicked() {

                        self.bs_action(ctx, a);

                    }

                }

                ui.add_space(4.0);

                let y = ui.cursor().top();

                ui.painter().line_segment([Pos2::new(14.0, y), Pos2::new(200.0, y)], Stroke::new(1.0_f32, RIBBON_LINE));

                ui.add_space(4.0);

                for (i, pg) in PAGES.iter().enumerate() {

                    if Self::bs_item(ui, pg, self.bs_page == i).clicked() {

                        self.bs_page = i;

                    }

                }

                ui.add_space(4.0);

                let y = ui.cursor().top();

                ui.painter().line_segment([Pos2::new(14.0, y), Pos2::new(200.0, y)], Stroke::new(1.0_f32, RIBBON_LINE));

                ui.add_space(4.0);

                for a in ["Opções", "Sair"] {

                    if Self::bs_item(ui, a, false).clicked() {

                        self.bs_action(ctx, a);

                    }

                }

                let full = ui.max_rect();

                ui.painter().line_segment([full.right_top(), full.right_bottom()], Stroke::new(1.0_f32, RIBBON_LINE));

            });



        egui::CentralPanel::default().frame(Frame::none().fill(Color32::WHITE).inner_margin(Margin::same(28.0))).show(ctx, |ui| {

            match self.bs_page {

                0 => {

                    ui.label(RichText::new(format!("Informações sobre {}", self.db_path)).size(22.0).color(BLUE_TITLE));

                    ui.separator();

                    ui.add_space(8.0);

                    ui.horizontal_top(|ui| {

                        ui.vertical(|ui| {

                            if card(ui, Icon::Badge("C", C_GREEN), "Compactar e Reparar", "Ajuda a evitar e corrigir problemas no ficheiro da base de dados.").clicked() {

                                self.msg = "«Compactar e Reparar» – apenas apresentação visual".into();

                            }

                            ui.add_space(6.0);

                            card(ui, Icon::Badge("*", C_ORANGE), "Cifrar com Palavra-Passe", "Utilize uma palavra-passe para restringir o acesso à base de dados.");

                        });

                        ui.add_space(40.0);

                        ui.vertical(|ui| {

                            ui.label(RichText::new("Propriedades da Base de Dados").size(14.0).color(BLUE_TITLE));

                            ui.add_space(6.0);

                            egui::Grid::new("props").spacing([18.0, 6.0]).show(ui, |ui| {

                                for (k, v) in [("Nome:", self.db_path.clone()), ("Tamanho:", "1,2 MB".into()), ("Motor de dados:", "SQLite (local)".into()), ("Estado:", "Apenas local, sem sincronização".into())] {

                                    ui.label(RichText::new(k).color(rgb(0x5A6B85)));

                                    ui.label(v);

                                    ui.end_row();

                                }

                            });

                        });

                    });

                }

                1 => {

                    ui.label(RichText::new("Bases de Dados Recentes").size(22.0).color(BLUE_TITLE));

                    ui.separator();

                    ui.add_space(8.0);

                    for (n, path) in [

                        ("Inventário.sqlite", "/home/utilizador/Documentos"),

                        ("Clientes.accdb", "/home/utilizador/Documentos/Access"),

                        ("Faturação.sqlite", "/home/utilizador/Trabalho"),

                        ("Base de Dados1.accdb", "/home/utilizador"),

                    ] {

                        if card(ui, Icon::Table, n, path).clicked() {

                            self.db_path = n.split('.').next().unwrap_or(n).to_string();

                            self.backstage = false;

                        }

                    }

                }

                2 => {

                    ui.label(RichText::new("Bases de Dados Disponíveis").size(22.0).color(BLUE_TITLE));

                    ui.separator();

                    ui.add_space(8.0);

                    ui.horizontal_top(|ui| {

                        ui.vertical(|ui| {

                            card(ui, Icon::Table, "Base de Dados em Branco", "Criar uma base de dados local vazia.");

                            ui.add_space(6.0);

                            card(ui, Icon::Badge("M", C_BLUE), "Modelos Locais", "Modelos instalados neste computador.");

                        });

                        ui.add_space(40.0);

                        ui.vertical(|ui| {

                            ui.label(RichText::new("Base de Dados em Branco").size(16.0).color(BLUE_TITLE));

                            ui.add_space(8.0);

                            ui.label("Nome do ficheiro");

                            ui.horizontal(|ui| {

                                ui.add(egui::TextEdit::singleline(&mut self.new_db_name).desired_width(260.0));

                                if ui.button("📁").on_hover_text("Escolher pasta...").clicked() {

                                    if let Some(p) = rfd::FileDialog::new().set_file_name(self.new_db_name.as_str()).save_file() {

                                        self.new_db_name = p.display().to_string();

                                    }

                                }

                            });

                            ui.add_space(10.0);

                            if ui.add(egui::Button::new(RichText::new("Criar").size(14.0)).min_size(Vec2::new(90.0, 30.0))).clicked() {

                                self.db_path = self.new_db_name.split('.').next().unwrap_or("Base de Dados1").to_string();

                                self.msg = "«Criar» – apenas apresentação visual".into();

                                self.backstage = false;

                            }

                        });

                    });

                }

                p => {

                    let t = ["", "", "", "Imprimir", "Ajuda"][p.min(4)];

                    ui.label(RichText::new(t).size(22.0).color(BLUE_TITLE));

                    ui.separator();

                    ui.add_space(8.0);

                    ui.label(RichText::new("Página ilustrativa – sem funcionalidade.").color(rgb(0x5A6B85)));

                }

            }

        });

    }

}
