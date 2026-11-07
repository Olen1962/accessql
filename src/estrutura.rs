//! Vista Estrutura: ver, criar e alterar os campos de uma tabela.

use crate::{
    db::{Campo, TIPOS},
    estado::*,
    tema::*,
};
use eframe::egui::{self, RichText, Ui};

/// Estado do separador de estrutura (uma tabela existente ou uma tabela nova).
#[derive(Default)]
pub(crate) struct Estrutura {
    /// `true` = tabela ainda por criar (os campos são editáveis).
    pub(crate) nova: bool,
    pub(crate) nome_tabela: String,
    pub(crate) campos: Vec<Campo>,
    /// Texto `CREATE TABLE` original (só em tabelas existentes).
    pub(crate) sql: String,
    pub(crate) renomear: Option<(usize, String)>,
    pub(crate) eliminar: Option<usize>,
    /// Campo a acrescentar a uma tabela existente.
    pub(crate) novo: Campo,
    pub(crate) info: String,
    pub(crate) info_erro: bool,
}

fn campo_texto() -> Campo {
    Campo { tipo: "TEXT".into(), ..Default::default() }
}

fn combo_tipo(ui: &mut Ui, id: impl std::hash::Hash, valor: &mut String) {
    egui::ComboBox::from_id_source(id)
        .width(100.0)
        .selected_text(valor.clone())
        .show_ui(ui, |ui| {
            for t in TIPOS {
                ui.selectable_value(&mut *valor, t.to_string(), t);
            }
        });
}

enum Ac {
    Criar,
    Acrescentar,
    Renomear(usize, String),
    Eliminar(usize),
    AddLinha,
    RemLinha(usize),
}

impl AccessApp {
    /// Abre a estrutura de uma tabela existente.
    pub(crate) fn abrir_estrutura(&mut self, nome: &str) {
        if let Some(i) = self
            .docs
            .iter()
            .position(|d| d.kind == ObjKind::Design && d.name == nome && d.estr.as_ref().map_or(false, |e| !e.nova))
        {
            self.active = i;
            return;
        }
        let r = match self.db.as_ref() {
            Some(db) => db.info_tabela(nome),
            None => return,
        };
        match r {
            Ok((campos, sql)) => {
                let est = Estrutura { campos, sql, novo: campo_texto(), ..Default::default() };
                self.docs.push(Doc {
                    kind: ObjKind::Design,
                    name: nome.to_string(),
                    data: Default::default(),
                    sql: String::new(),
                    info: String::new(),
                    info_erro: false,
                    vista: None,
                    estr: Some(est),
                });
                self.active = self.docs.len() - 1;
                self.edit = None;
                self.row_sel = false;
            }
            Err(e) => self.msg = format!("Não foi possível ler a estrutura de «{}»: {}", nome, e),
        }
    }

    /// Abre o desenhador de uma tabela nova.
    pub(crate) fn nova_tabela_design(&mut self) {
        if self.db.is_none() {
            self.msg = "Abra ou crie uma base de dados primeiro".into();
            return;
        }
        self.n_tabela += 1;
        let nome = format!("Tabela{}", self.n_tabela);
        let id = Campo { nome: "id".into(), tipo: "INTEGER".into(), pk: true, ..Default::default() };
        let est = Estrutura {
            nova: true,
            nome_tabela: nome.clone(),
            campos: vec![id, Campo { nome: "nome".into(), ..campo_texto() }],
            novo: campo_texto(),
            ..Default::default()
        };
        self.docs.push(Doc {
            kind: ObjKind::Design,
            name: nome,
            data: Default::default(),
            sql: String::new(),
            info: String::new(),
            info_erro: false,
            vista: None,
            estr: Some(est),
        });
        self.active = self.docs.len() - 1;
        self.edit = None;
        self.row_sel = false;
    }

    /// Botão «Vista»: alterna entre a folha de dados e a estrutura da tabela.
    pub(crate) fn alternar_vista(&mut self) {
        let info = self
            .docs
            .get(self.active)
            .map(|d| (d.kind, d.name.clone(), d.estr.as_ref().map_or(false, |e| e.nova)));
        match info {
            Some((ObjKind::Table, n, _)) => self.abrir_estrutura(&n),
            Some((ObjKind::Design, n, false)) => self.open_doc(ObjKind::Table, n),
            Some((ObjKind::Design, _, true)) => self.msg = "Crie primeiro a tabela".into(),
            _ => self.msg = "Esta vista só existe para tabelas".into(),
        }
    }

    /// Grupo «Vistas» do ribbon.
    pub(crate) fn grupo_vistas(&mut self, ui: &mut Ui) {
        let (rotulo, icone) = match self.docs.get(self.active).map(|d| d.kind) {
            Some(ObjKind::Table) => ("Vista\nEstrutura", crate::icones::Icon::Badge("E", C_GREEN)),
            Some(ObjKind::Design) => ("Vista Folha\nde Dados", crate::icones::Icon::Table),
            _ => ("Vista", crate::icones::Icon::Table),
        };
        crate::widgets::group(ui, "Vistas", |ui| {
            if crate::widgets::big_btn(ui, icone, rotulo, false).clicked() {
                self.alternar_vista();
            }
        });
    }

    /// Volta a ler as folhas de dados abertas de `tabela` (a estrutura mudou).
    fn recarregar_abertos(&mut self, tabela: &str) {
        let idx: Vec<usize> = self
            .docs
            .iter()
            .enumerate()
            .filter(|(_, d)| d.kind == ObjKind::Table && d.name == tabela)
            .map(|(i, _)| i)
            .collect();
        for i in idx {
            if let Some(Ok(d)) = self.db.as_ref().map(|db| db.load(tabela, true)) {
                self.docs[i].data = d;
            }
        }
    }

    pub(crate) fn estrutura_view(&mut self, ui: &mut Ui) {
        let Some(mut est) = self.docs[self.active].estr.take() else {
            ui.label("Sem dados de estrutura.");
            return;
        };
        let tabela = self.docs[self.active].name.clone();
        let mut ac: Option<Ac> = None;
        egui::ScrollArea::vertical().id_source("estr_scroll").auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(6.0);
            if est.nova {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new("Nome da tabela:").strong());
                    ui.add(egui::TextEdit::singleline(&mut est.nome_tabela).desired_width(200.0));
                });
            } else {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("Estrutura da tabela «{}»", tabela)).strong().size(15.0));
                });
            }
            ui.add_space(6.0);
            if let Some(i) = est.eliminar {
                let nome = est.campos.get(i).map(|c| c.nome.clone()).unwrap_or_default();
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(8.0);
                    ui.colored_label(
                        rgb(0xB00020),
                        format!("Eliminar o campo «{}» e todos os seus dados? Não poderá anular esta operação.", nome),
                    );
                    if ui.button("Sim, eliminar").clicked() {
                        ac = Some(Ac::Eliminar(i));
                    }
                    if ui.button("Cancelar").clicked() {
                        est.eliminar = None;
                    }
                });
                ui.add_space(6.0);
            }
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                egui::Grid::new("estr_grid").striped(true).spacing([12.0, 6.0]).show(ui, |ui| {
                    for h in ["", "Nome do Campo", "Tipo de Dados", "Obrigatório", "Valor por Omissão", "Chave Primária", ""] {
                        ui.label(RichText::new(h).strong());
                    }
                    ui.end_row();
                    for (i, c) in est.campos.iter_mut().enumerate() {
                        if est.nova {
                            ui.label(if c.pk { "🔑" } else { "" });
                            ui.add(egui::TextEdit::singleline(&mut c.nome).desired_width(170.0));
                            combo_tipo(ui, ("tipo", i), &mut c.tipo);
                            ui.checkbox(&mut c.obrigatorio, "");
                            ui.add(egui::TextEdit::singleline(&mut c.omissao).desired_width(130.0));
                            ui.checkbox(&mut c.pk, "");
                            if ui.small_button("✕").on_hover_text("Remover campo").clicked() {
                                ac = Some(Ac::RemLinha(i));
                            }
                        } else {
                            ui.label(if c.pk { "🔑" } else { "" });
                            let a_renomear = matches!(&est.renomear, Some((ri, _)) if *ri == i);
                            if a_renomear {
                                let (ok, cancelar) = ui
                                    .horizontal(|ui| {
                                        if let Some((_, t)) = est.renomear.as_mut() {
                                            ui.add(egui::TextEdit::singleline(t).desired_width(120.0));
                                        }
                                        (ui.small_button("OK").clicked(), ui.small_button("✕").clicked())
                                    })
                                    .inner;
                                if ok {
                                    if let Some((_, t)) = &est.renomear {
                                        ac = Some(Ac::Renomear(i, t.clone()));
                                    }
                                }
                                if cancelar {
                                    est.renomear = None;
                                }
                            } else {
                                ui.label(&c.nome);
                            }
                            ui.label(&c.tipo);
                            ui.label(if c.obrigatorio { "Sim" } else { "Não" });
                            ui.label(&c.omissao);
                            ui.label(if c.pk { "Sim" } else { "" });
                            ui.horizontal(|ui| {
                                if ui.small_button("Renomear").clicked() {
                                    est.renomear = Some((i, c.nome.clone()));
                                }
                                if ui.small_button("Eliminar").clicked() {
                                    est.eliminar = Some(i);
                                }
                            });
                        }
                        ui.end_row();
                    }
                });
            });
            ui.add_space(10.0);
            if est.nova {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    if ui.button("+ Acrescentar campo").clicked() {
                        ac = Some(Ac::AddLinha);
                    }
                    if ui.add(egui::Button::new("Criar Tabela").min_size(egui::vec2(110.0, 24.0))).clicked() {
                        ac = Some(Ac::Criar);
                    }
                });
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(8.0);
                    ui.weak("Uma chave primária INTEGER é numerada automaticamente. Valor por omissão: número, texto, NULL ou CURRENT_TIMESTAMP.");
                });
            } else {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new("Acrescentar campo:").strong());
                    ui.add(egui::TextEdit::singleline(&mut est.novo.nome).hint_text("nome").desired_width(150.0));
                    combo_tipo(ui, "tipo_novo", &mut est.novo.tipo);
                    ui.checkbox(&mut est.novo.obrigatorio, "Obrigatório");
                    ui.add(egui::TextEdit::singleline(&mut est.novo.omissao).hint_text("valor por omissão").desired_width(130.0));
                    if ui.button("Acrescentar").clicked() {
                        ac = Some(Ac::Acrescentar);
                    }
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    egui::CollapsingHeader::new("Definição SQL").show(ui, |ui| {
                        ui.label(RichText::new(est.sql.clone()).monospace());
                    });
                });
            }
            if !est.info.is_empty() {
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new(est.info.clone()).color(if est.info_erro { rgb(0xB00020) } else { rgb(0x2B5A9E) }));
                });
            }
        });

        match ac {
            Some(Ac::AddLinha) => est.campos.push(campo_texto()),
            Some(Ac::RemLinha(i)) => {
                if i < est.campos.len() {
                    est.campos.remove(i);
                }
            }
            Some(Ac::Criar) => {
                let nome = est.nome_tabela.trim().to_string();
                let r = self
                    .db
                    .as_ref()
                    .map(|db| db.criar_tabela(&nome, &est.campos).and_then(|_| db.info_tabela(&nome)));
                match r {
                    Some(Ok((campos, sql))) => {
                        est.campos = campos;
                        est.sql = sql;
                        est.nova = false;
                        est.info = format!("Tabela «{}» criada", nome);
                        est.info_erro = false;
                        self.docs[self.active].name = nome.clone();
                        self.msg = est.info.clone();
                        self.refresh_nav();
                    }
                    Some(Err(e)) => {
                        est.info = e;
                        est.info_erro = true;
                    }
                    None => {}
                }
            }
            Some(Ac::Acrescentar) => {
                let r = self
                    .db
                    .as_ref()
                    .map(|db| db.acrescentar_campo(&tabela, &est.novo).and_then(|_| db.info_tabela(&tabela)));
                match r {
                    Some(Ok((campos, sql))) => {
                        est.info = format!("Campo «{}» acrescentado", est.novo.nome.trim());
                        est.info_erro = false;
                        est.campos = campos;
                        est.sql = sql;
                        est.novo = campo_texto();
                        self.recarregar_abertos(&tabela);
                    }
                    Some(Err(e)) => {
                        est.info = e;
                        est.info_erro = true;
                    }
                    None => {}
                }
            }
            Some(Ac::Renomear(i, para)) => {
                let de = est.campos.get(i).map(|c| c.nome.clone()).unwrap_or_default();
                let r = self
                    .db
                    .as_ref()
                    .map(|db| db.renomear_campo(&tabela, &de, &para).and_then(|_| db.info_tabela(&tabela)));
                est.renomear = None;
                match r {
                    Some(Ok((campos, sql))) => {
                        est.info = format!("Campo «{}» renomeado para «{}»", de, para.trim());
                        est.info_erro = false;
                        est.campos = campos;
                        est.sql = sql;
                        self.recarregar_abertos(&tabela);
                    }
                    Some(Err(e)) => {
                        est.info = e;
                        est.info_erro = true;
                    }
                    None => {}
                }
            }
            Some(Ac::Eliminar(i)) => {
                let nome = est.campos.get(i).map(|c| c.nome.clone()).unwrap_or_default();
                let r = self
                    .db
                    .as_ref()
                    .map(|db| db.eliminar_campo(&tabela, &nome).and_then(|_| db.info_tabela(&tabela)));
                est.eliminar = None;
                match r {
                    Some(Ok((campos, sql))) => {
                        est.info = format!("Campo «{}» eliminado", nome);
                        est.info_erro = false;
                        est.campos = campos;
                        est.sql = sql;
                        self.recarregar_abertos(&tabela);
                    }
                    Some(Err(e)) => {
                        est.info = e;
                        est.info_erro = true;
                    }
                    None => {}
                }
            }
            None => {}
        }
        self.docs[self.active].estr = Some(est);
    }
}
