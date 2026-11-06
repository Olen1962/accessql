use eframe::egui::{self, Align2, Color32, FontId, Frame, Margin, Pos2, Response, RichText, Sense, Stroke, Ui, Vec2};
use crate::{estado::*, icones::*, tema::*, widgets::*};

fn tamanho_legivel(b: u64) -> String {
    if b < 1024 {
        format!("{} bytes", b)
    } else if b < 1024 * 1024 {
        format!("{:.1} KB", b as f64 / 1024.0).replace('.', ",")
    } else {
        format!("{:.1} MB", b as f64 / 1_048_576.0).replace('.', ",")
    }
}

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
            "Abrir" => self.dialogo_abrir(),
            "Guardar Cópia Como" => self.guardar_copia(),
            "Fechar Base de Dados" => self.fechar_bd(),
            "Sair" => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            _ => {}
        }
    }

    pub(crate) fn backstage_ui(&mut self, ctx: &egui::Context) {
        const ACTIONS: [&str; 3] = ["Abrir", "Guardar Cópia Como", "Fechar Base de Dados"];
        const PAGES: [&str; 3] = ["Informações", "Recente", "Novo"];
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
                if Self::bs_item(ui, "Sair", false).clicked() {
                    self.bs_action(ctx, "Sair");
                }
                let full = ui.max_rect();
                ui.painter().line_segment([full.right_top(), full.right_bottom()], Stroke::new(1.0_f32, RIBBON_LINE));
            });
        egui::CentralPanel::default().frame(Frame::none().fill(Color32::WHITE).inner_margin(Margin::same(28.0))).show(ctx, |ui| {
            match self.bs_page {
                0 => {
                    let titulo = if self.db.is_some() { format!("Informações sobre {}", self.db_path) } else { "Informações".to_string() };
                    ui.label(RichText::new(titulo).size(22.0).color(BLUE_TITLE));
                    ui.separator();
                    ui.add_space(8.0);
                    if self.db.is_none() {
                        ui.label(RichText::new("Nenhuma base de dados aberta.").color(rgb(0x5A6B85)));
                    } else {
                        let caminho = self.db_file.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "Em memória (não fica guardada)".into());
                        let tamanho = self
                            .db_file
                            .as_ref()
                            .and_then(|p| std::fs::metadata(p).ok())
                            .map(|m| tamanho_legivel(m.len()))
                            .unwrap_or_else(|| "—".into());
                        let n_tab = self.nav.first().map_or(0, |g| g.items.len());
                        let n_con = self.nav.get(1).map_or(0, |g| g.items.len());
                        ui.horizontal_top(|ui| {
                            ui.vertical(|ui| {
                                if card(ui, Icon::Badge("C", C_GREEN), "Compactar e Reparar", "Verifica a integridade e reduz o tamanho do ficheiro da base de dados.").clicked() {
                                    self.compactar_bd();
                                }
                            });
                            ui.add_space(40.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Propriedades da Base de Dados").size(14.0).color(BLUE_TITLE));
                                ui.add_space(6.0);
                                egui::Grid::new("props").spacing([18.0, 6.0]).show(ui, |ui| {
                                    for (k, v) in [
                                        ("Nome:", self.db_path.clone()),
                                        ("Caminho:", caminho),
                                        ("Tamanho:", tamanho),
                                        ("Tabelas:", n_tab.to_string()),
                                        ("Consultas:", n_con.to_string()),
                                        ("Motor de dados:", format!("SQLite {}", rusqlite::version())),
                                    ] {
                                        ui.label(RichText::new(k).color(rgb(0x5A6B85)));
                                        ui.label(v);
                                        ui.end_row();
                                    }
                                });
                            });
                        });
                    }
                }
                1 => {
                    ui.label(RichText::new("Bases de Dados Recentes").size(22.0).color(BLUE_TITLE));
                    ui.separator();
                    ui.add_space(8.0);
                    if self.recentes.is_empty() {
                        ui.label(RichText::new("Ainda não abriu nenhuma base de dados.").color(rgb(0x5A6B85)));
                    } else {
                        let lista = self.recentes.clone();
                        let mut abrir = None;
                        for p in &lista {
                            let nome = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                            let pasta = p.parent().map(|d| d.display().to_string()).unwrap_or_default();
                            if card(ui, Icon::Table, &nome, &pasta).clicked() {
                                abrir = Some(p.clone());
                            }
                        }
                        if let Some(p) = abrir {
                            self.abrir_ficheiro(p);
                        }
                    }
                }
                _ => {
                    ui.label(RichText::new("Bases de Dados Disponíveis").size(22.0).color(BLUE_TITLE));
                    ui.separator();
                    ui.add_space(8.0);
                    ui.horizontal_top(|ui| {
                        ui.vertical(|ui| {
                            card(ui, Icon::Table, "Base de Dados em Branco", "Criar uma base de dados local vazia.");
                            ui.add_space(6.0);
                            if card(ui, Icon::Badge("D", C_BLUE), "Base de Dados de Demonstração", "Dados de exemplo em memória (não ficam guardados).").clicked() {
                                self.abrir_demo();
                            }
                        });
                        ui.add_space(40.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Base de Dados em Branco").size(16.0).color(BLUE_TITLE));
                            ui.add_space(8.0);
                            ui.label("Nome do ficheiro");
                            ui.add(egui::TextEdit::singleline(&mut self.new_db_name).desired_width(300.0));
                            ui.add_space(6.0);
                            ui.label("Pasta");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.new_db_dir).desired_width(260.0));
                                if ui.button("📁").on_hover_text("Escolher pasta...").clicked() {
                                    if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                        self.new_db_dir = p.display().to_string();
                                    }
                                }
                            });
                            ui.add_space(10.0);
                            if ui.add(egui::Button::new(RichText::new("Criar").size(14.0)).min_size(Vec2::new(90.0, 30.0))).clicked() {
                                self.criar_ficheiro();
                            }
                        });
                    });
                }
            }
        });
    }
}
