use eframe::egui::{self, Align2, FontId, Frame, Margin, Pos2, Sense, Stroke, Ui, Vec2};
use crate::{db::Op, estado::*, icones::*, tema::*, widgets::*};
impl AccessApp {
// ───────────── Barra de título / Acesso Rápido ─────────────
    pub(crate) fn qat(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("qat")
            .exact_height(26.0)
            .frame(Frame::none().fill(TITLE_BG))
            .show(ctx, |ui| {
                let full = ui.max_rect();
                ui.painter().text(
                    full.center(),
                    Align2::CENTER_CENTER,
                    if self.db.is_some() { format!("{} : Base de Dados SQLite - AccesSQL", self.db_path) } else { "AccesSQL".to_string() },
                    FontId::proportional(12.5),
                    TEXT,
                );
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(1.0, 0.0);
                    ui.add_space(6.0);
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                    draw_icon(ui.painter(), r, Icon::Logo);
                    ui.add_space(6.0);
                    for (ic, tip) in [
                        (Icon::Undo, "Desfazer (Ctrl+Z)"),
                        (Icon::Redo, "Refazer (Ctrl+Y)"),
                    ] {
                        let r = small_btn(ui, ic, "", false).on_hover_text(tip);
                        if r.clicked() {
                            match ic {
                                Icon::Undo => self.desfazer(),
                                Icon::Redo => self.refazer(),
                                _ => {}
                            }
                        }
                    }
                });
            });
    }
    // ───────────── Separadores do ribbon ─────────────
    pub(crate) fn tab_strip(&mut self, ctx: &egui::Context) {
        let is_table = self.docs.get(self.active).map_or(false, |d| d.kind == ObjKind::Table);
        if self.tab == 4 && !is_table {
            self.tab = 0;
        }
        egui::TopBottomPanel::top("tabs")
            .exact_height(27.0)
            .frame(Frame::none().fill(TAB_BG).inner_margin(Margin { left: 6.0, right: 6.0, top: 3.0, bottom: 0.0 }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);
                    if ribbon_tab(ui, "Ficheiro", self.backstage, TabStyle::File).clicked() {
                        self.backstage = !self.backstage;
                    }
                    let names = ["Base", "Criar", "Dados Externos", "Ferramentas de Bases de Dados", "Folha de Dados"];
                    for (i, n) in names.iter().enumerate() {
                        if i == 4 && !is_table {
                            continue;
                        }
                        let sel = !self.backstage && self.tab == i;
                        let r = ribbon_tab(ui, n, sel, if i == 4 { TabStyle::Context } else { TabStyle::Normal });
                        if sel {
                            self.tab_x = (r.rect.left(), r.rect.right());
                        }
                        if r.clicked() {
                            self.tab = i;
                            self.backstage = false;
                            }
                    }
                });
            });
    }
    pub(crate) fn ribbon(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("ribbon")
            .exact_height(96.0)
            .frame(Frame::none().fill(RIBBON_BG).inner_margin(Margin { left: 6.0, right: 6.0, top: 4.0, bottom: 2.0 }))
            .show(ctx, |ui| {
                let full = ui.max_rect();
                let st = Stroke::new(1.0_f32, RIBBON_LINE);
                let (a, b) = self.tab_x;
                let y = full.top() + 0.5;
                ui.painter().line_segment([Pos2::new(full.left(), y), Pos2::new(a + 1.0, y)], st);
                ui.painter().line_segment([Pos2::new(b - 1.0, y), Pos2::new(full.right(), y)], st);
                ui.painter().line_segment([Pos2::new(full.left(), full.bottom() - 0.5), Pos2::new(full.right(), full.bottom() - 0.5)], st);
                ui.horizontal_top(|ui| match self.tab {
                    0 => self.ribbon_base(ui),
                    1 => self.ribbon_criar(ui),
                    2 => self.ribbon_externos(ui),
                    3 => self.ribbon_ferramentas(ui),
                    _ => self.ribbon_folha(ui),
                });
            });
    }
    pub(crate) fn ribbon_base(&mut self, ui: &mut Ui) {
        self.grupo_vistas(ui);
        group(ui, "Área de Transferência", |ui| {
            // «Colar» só existe pelo teclado (Ctrl+V): o egui não deixa ler a área de transferência
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                if small_btn(ui, Icon::Badge("X", C_GRAY), "Cortar (Ctrl+X)", false).clicked() {
                    self.clip_acao = Some(1);
                }
                if small_btn(ui, Icon::Badge("C", C_BLUE), "Copiar (Ctrl+C)", false).clicked() {
                    self.clip_acao = Some(0);
                }
            });
        });
        group(ui, "Ordenar e Filtrar", |ui| {
            if big_btn(ui, Icon::Filter, "Filtro", false).clicked() {
                let pos = ui.ctx().pointer_latest_pos().unwrap_or(Pos2::new(300.0, 200.0));
                self.abrir_menu_coluna(self.sel.1, pos);
            }
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                if small_btn(ui, Icon::Badge("A", C_BLUE), "Ascendente", false).clicked() {
                    self.ordenar(self.sel.1, true);
                }
                if small_btn(ui, Icon::Badge("Z", C_BLUE), "Descendente", false).clicked() {
                    self.ordenar(self.sel.1, false);
                }
                if small_btn(ui, Icon::Badge("x", C_RED), "Remover Ordenação", false).clicked() {
                    self.limpar_ordem();
                }
            });
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                if small_btn(ui, Icon::Filter, "Seleção", false).clicked() {
                    self.filtrar_selecao(Op::Igual);
                }
                if small_btn(ui, Icon::Badge("x", C_RED), "Remover Filtros", false).clicked() {
                    self.remover_todos_filtros();
                }
                if small_btn(ui, Icon::Filter, "Alternar Filtro", false).clicked() {
                    self.alternar_filtro();
                }
            });
        });
        group(ui, "Registos", |ui| {
            if big_btn(ui, Icon::Badge("R", C_GREEN), "Atualizar\nTudo", false).clicked() {
                self.recarregar_doc();
            }
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                if small_btn(ui, Icon::Badge("+", C_GREEN), "Novo", false).clicked() {
                    self.novo_registo();
                }
                if small_btn(ui, Icon::Badge("x", C_RED), "Eliminar", false).clicked() {
                    self.pedir_eliminar();
                }
            });
            if big_btn(ui, Icon::Badge("Σ", C_PURPLE), "Totais", false).clicked() {
                self.totais_aberto = true;
            }
        });
        group(ui, "Localizar", |ui| {
            if big_btn(ui, Icon::Badge("L", C_BLUE), "Localizar", false).clicked() {
                self.focar_busca = true;
            }
        });
    }
    pub(crate) fn ribbon_criar(&mut self, ui: &mut Ui) {
        group(ui, "Tabelas", |ui| {
            if big_btn(ui, Icon::Table, "Estrutura\nda Tabela", false).clicked() {
                self.nova_tabela_design();
            }
            if big_btn(ui, Icon::Table, "Tabela\n(SQL)", false).clicked() {
                self.nova_consulta_sql("CREATE TABLE nova_tabela (\n    id INTEGER PRIMARY KEY,\n    nome TEXT\n);");
            }
        });
        group(ui, "Consultas", |ui| {
            if big_btn(ui, Icon::Query, "Consulta\nSQL", false).clicked() {
                self.nova_consulta_sql("SELECT * FROM ");
            }
        });
    }
    pub(crate) fn ribbon_externos(&mut self, ui: &mut Ui) {
        group(ui, "Importar", |ui| {
            if big_btn(ui, Icon::Badge("↓", C_BLUE), "Ficheiro\nCSV", false).clicked() {
                self.pedir_importar_csv();
            }
        });
        group(ui, "Exportar", |ui| {
            if big_btn(ui, Icon::Badge("↑", C_ORANGE), "Ficheiro\nCSV", false).clicked() {
                self.pedir_exportar_csv();
            }
        });
    }
    pub(crate) fn ribbon_ferramentas(&mut self, ui: &mut Ui) {
        group(ui, "Ferramentas", |ui| {
            if big_btn(ui, Icon::Badge("C", C_GREEN), "Compactar e\nReparar", false).clicked() {
                self.compactar_bd();
            }
            if big_btn(ui, Icon::Badge("I", C_BLUE), "Índices", false).clicked() {
                self.abrir_indices();
            }
            if big_btn(ui, Icon::Badge("R", C_ORANGE), "Relações", false).clicked() {
                self.abrir_relacoes();
            }
        });
    }
    pub(crate) fn ribbon_folha(&mut self, ui: &mut Ui) {
        self.grupo_vistas(ui);
        group(ui, "Campos e Colunas", |ui| {
            if big_btn(ui, Icon::Badge("+", C_GREEN), "Novo\nCampo", false).clicked() {
                self.campo_na_estrutura(0);
            }
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                if small_btn(ui, Icon::Badge("R", C_BLUE), "Renomear Campo", false).clicked() {
                    self.campo_na_estrutura(1);
                }
                if small_btn(ui, Icon::Badge("x", C_RED), "Eliminar Campo", false).clicked() {
                    self.campo_na_estrutura(2);
                }
            });
        });
    }
}
