use eframe::egui::{self, Align, Align2, Color32, FontId, Frame, Layout, Margin, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use crate::{dados_exemplo::*, estado::*, icones::*, tema::*, widgets::*};
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
                    format!("{} : Base de Dados SQLite - AccesSQL", self.db_path),
                    FontId::proportional(12.5),
                    TEXT,
                );
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(1.0, 0.0);
                    ui.add_space(6.0);
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                    draw_icon(ui.painter(), r, Icon::Badge("A", ACCENT));
                    ui.add_space(6.0);
                    for (ic, tip) in [(Icon::Save, "Guardar"), (Icon::Undo, "Anular"), (Icon::Redo, "Refazer")] {
                        let r = small_btn(ui, ic, "", false).on_hover_text(tip);
                        if r.clicked() {
                            self.msg = format!("«{}» – apenas apresentação visual", tip);
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
                        self.dropdown = None;
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
                            self.dropdown = None;
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (r, resp) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
                        let p = ui.painter();
                        p.circle_filled(r.center(), 8.0, if resp.hovered() { rgb(0x5B8BC9) } else { C_BLUE });
                        p.text(r.center(), Align2::CENTER_CENTER, "?", FontId::proportional(12.0), Color32::WHITE);
                        resp.on_hover_text("Ajuda do AccesSQL (F1)");
                    });
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
        group(ui, "Vistas", |ui| self.rb(ui, Icon::Table, "Vista", Some("vista")));
        group(ui, "Área de Transferência", |ui| {
            self.rb(ui, Icon::Paste, "Colar", Some("colar"));
            self.stack(ui, &[
                (Icon::Badge("X", C_GRAY), "Cortar", None),
                (Icon::Badge("C", C_BLUE), "Copiar", None),
                (Icon::Badge("F", C_ORANGE), "Pincel de Formatos", None),
            ]);
        });
        group(ui, "Ordenar e Filtrar", |ui| {
            self.rb(ui, Icon::Filter, "Filtro", None);
            self.stack(ui, &[
                (Icon::Badge("A", C_BLUE), "Ascendente", None),
                (Icon::Badge("Z", C_BLUE), "Descendente", None),
                (Icon::Badge("x", C_RED), "Remover Ordenação", None),
            ]);
            self.stack(ui, &[
                (Icon::Filter, "Seleção", Some("sel")),
                (Icon::Filter, "Avançadas", Some("avanc")),
                (Icon::Filter, "Alternar Filtro", None),
            ]);
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
                if small_btn(ui, Icon::Save, "Guardar", false).clicked() {
                    self.msg = "Os registos são gravados automaticamente".into();
                }
                if small_btn(ui, Icon::Badge("x", C_RED), "Eliminar", false).clicked() {
                    self.pedir_eliminar();
                }
            });
            self.stack(ui, &[
                (Icon::Badge("S", C_PURPLE), "Totais", None),
                (Icon::Badge("O", C_BLUE), "Ortografia", None),
                (Icon::Badge("…", C_GRAY), "Mais", Some("mais_reg")),
            ]);
        });
        group(ui, "Localizar", |ui| {
            self.rb(ui, Icon::Badge("L", C_BLUE), "Localizar", None);
            self.stack(ui, &[
                (Icon::Badge("S", C_ORANGE), "Substituir", None),
                (Icon::Badge("→", C_GRAY), "Ir Para", Some("goto")),
                (Icon::Badge("▢", C_GRAY), "Selecionar", Some("selecionar")),
            ]);
        });
        group(ui, "Formatação de Texto", |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(2.0, 4.0);
                ui.horizontal(|ui| {
                    faux_combo(ui, "Calibri", 110.0);
                    faux_combo(ui, "11", 34.0);
                });
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    for (t, c) in [("N", C_GRAY), ("I", C_GRAY), ("S", C_GRAY), ("A", C_RED), ("◧", C_ORANGE)] {
                        let r = small_btn(ui, Icon::Badge(t, c), "", false);
                        self.act(&r, "Formatação", None);
                    }
                });
            });
        });
    }
    pub(crate) fn ribbon_criar(&mut self, ui: &mut Ui) {
        group(ui, "Modelos", |ui| self.rb(ui, Icon::Badge("P", C_PURPLE), "Partes da\nAplicação", Some("partes")));
        group(ui, "Tabelas", |ui| {
            self.rb(ui, Icon::Table, "Tabela", None);
            self.rb(ui, Icon::Badge("M", C_BLUE), "Modelos de\nTabela", Some("modelos"));
            self.rb(ui, Icon::Badge("E", C_GRAY), "Estrutura\nda Tabela", None);
        });
        group(ui, "Consultas", |ui| {
            self.rb(ui, Icon::Query, "Assistente de\nConsultas", None);
            self.rb(ui, Icon::Badge("E", C_GREEN), "Estrutura da\nConsulta", None);
        });
        group(ui, "Formulários", |ui| {
            self.rb(ui, Icon::Form, "Formulário", None);
            self.rb(ui, Icon::Badge("E", C_ORANGE), "Estrutura do\nFormulário", None);
            self.rb(ui, Icon::Badge("B", C_GRAY), "Formulário\nem Branco", None);
            self.stack(ui, &[
                (Icon::Form, "Assistente de Formulários", None),
                (Icon::Badge("N", C_BLUE), "Navegação", Some("navegacao")),
                (Icon::Form, "Mais Formulários", Some("mais_forms")),
            ]);
        });
        group(ui, "Relatórios", |ui| {
            self.rb(ui, Icon::Report, "Relatório", None);
            self.rb(ui, Icon::Badge("E", C_RED), "Estrutura do\nRelatório", None);
            self.rb(ui, Icon::Badge("B", C_GRAY), "Relatório\nem Branco", None);
            self.stack(ui, &[
                (Icon::Report, "Assistente de Relatórios", None),
                (Icon::Badge("Et", C_BLUE), "Etiquetas", None),
            ]);
        });
        group(ui, "Macros e Código", |ui| {
            self.rb(ui, Icon::Badge("M", C_ORANGE), "Macro", None);
            self.stack(ui, &[
                (Icon::Badge("M", C_BLUE), "Módulo", None),
                (Icon::Badge("C", C_BLUE), "Módulo de Classe", None),
                (Icon::Badge("V", C_PURPLE), "Visual Basic", None),
            ]);
        });
    }
    pub(crate) fn ribbon_externos(&mut self, ui: &mut Ui) {
        group(ui, "Importar e Ligar", |ui| {
            self.rb(ui, Icon::Badge("X", C_GREEN), "Folha de\nCálculo Excel", None);
            self.rb(ui, Icon::Table, "Access", None);
            self.rb(ui, Icon::Badge("S", C_BLUE), "Base de Dados\nSQLite", None);
            self.rb(ui, Icon::Badge("D", C_GRAY), "Base de Dados\nODBC", None);
            self.rb(ui, Icon::Badge("T", C_BLUE), "Ficheiro\nde Texto", None);
            self.rb(ui, Icon::Badge("…", C_GRAY), "Mais", Some("imp_mais"));
        });
        group(ui, "Exportar", |ui| {
            self.rb(ui, Icon::Badge("X", C_GREEN), "Excel", None);
            self.rb(ui, Icon::Badge("T", C_BLUE), "Ficheiro\nde Texto", None);
            self.rb(ui, Icon::Badge("<>", C_ORANGE), "XML", None);
            self.rb(ui, Icon::Badge("P", C_RED), "PDF ou\nXPS", None);
            self.rb(ui, Icon::Badge("@", C_BLUE), "Correio\nEletrónico", None);
            self.rb(ui, Icon::Badge("…", C_GRAY), "Mais", Some("exp_mais"));
        });
    }
    pub(crate) fn ribbon_ferramentas(&mut self, ui: &mut Ui) {
        group(ui, "Ferramentas", |ui| {
            self.rb(ui, Icon::Badge("C", C_GREEN), "Compactar e\nReparar", None);
            self.rb(ui, Icon::Badge("M", C_ORANGE), "Executar\nMacro", None);
        });
        group(ui, "Relações", |ui| {
            self.rb(ui, Icon::Badge("R", C_BLUE), "Relações", None);
            self.stack(ui, &[(Icon::Badge("D", C_GRAY), "Dependências de Objetos", None)]);
        });
        group(ui, "Analisar", |ui| {
            self.rb(ui, Icon::Report, "Documentador de\nBases de Dados", None);
            self.stack(ui, &[
                (Icon::Badge("A", C_PURPLE), "Analisar Desempenho", None),
                (Icon::Table, "Analisar Tabela", None),
            ]);
        });
        group(ui, "Mover Dados", |ui| {
            self.rb(ui, Icon::Badge("S", C_BLUE), "Base de Dados\nSQLite", None);
            self.rb(ui, Icon::Table, "Base de Dados\nAccess", None);
        });
    }
    pub(crate) fn ribbon_folha(&mut self, ui: &mut Ui) {
        group(ui, "Vistas", |ui| self.rb(ui, Icon::Table, "Vista", Some("vista")));
        group(ui, "Campos e Colunas", |ui| {
            self.rb(ui, Icon::Badge("+", C_GREEN), "Novo\nCampo", None);
            self.rb(ui, Icon::Table, "Adicionar Campos\nExistentes", None);
            self.stack(ui, &[
                (Icon::Badge("I", C_BLUE), "Inserir", None),
                (Icon::Badge("P", C_ORANGE), "Modificar Pesquisas", None),
                (Icon::Badge("x", C_RED), "Eliminar", None),
            ]);
        });
        group(ui, "Tipo de Dados e Formatação", |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Tipo de Dados:").size(12.0).color(TEXT));
                    faux_combo(ui, "Texto Curto", 100.0);
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Formato:").size(12.0).color(TEXT));
                    faux_combo(ui, "", 120.0);
                });
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    for t in ["€", "%", ",", ".0", ".00"] {
                        let r = small_btn(ui, Icon::Badge(t, C_GRAY), "", false);
                        self.act(&r, "Formato", None);
                    }
                });
            });
        });
        group(ui, "Validação de Campos", |ui| {
            self.stack(ui, &[
                (Icon::Badge("!", C_RED), "Obrigatório", None),
                (Icon::Badge("1", C_BLUE), "Único", None),
                (Icon::Badge("i", C_GREEN), "Indexado", None),
            ]);
        });
    }
}
