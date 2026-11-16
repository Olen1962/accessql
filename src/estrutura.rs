//! Vista Estrutura: ver, criar e alterar os campos de uma tabela.

use crate::{
    db::{Campo, TIPOS},
    estado::*,
    icones::Icon,
    tema::*,
    widgets::{big_btn, group},
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
    /// Campo selecionado (o F2 muda-lhe o nome).
    pub(crate) sel: Option<usize>,
    pub(crate) renomear: Option<(usize, String)>,
    /// Pedir o foco para a caixa de texto do nome, no próximo desenho.
    pub(crate) foco_ren: bool,
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
    /// Abre a estrutura de uma tabela. Se a tabela já estiver aberta como folha de
    /// dados, o mesmo separador passa a mostrar a estrutura.
    pub(crate) fn abrir_estrutura(&mut self, nome: &str) {
        let existente = self.procurar_objeto(nome);
        if let Some(i) = existente {
            match self.docs[i].kind {
                ObjKind::Design => {
                    self.active = i;
                    return;
                }
                ObjKind::Table => {}
                _ => {
                    self.msg = "Só as tabelas têm Vista Estrutura".into();
                    return;
                }
            }
        }
        let r = match self.db.as_ref() {
            Some(db) => db.info_tabela(nome),
            None => return,
        };
        match r {
            Ok((campos, sql)) => {
                let est = Estrutura { campos, sql, novo: campo_texto(), ..Default::default() };
                let doc = Doc {
                    kind: ObjKind::Design,
                    name: nome.to_string(),
                    data: Default::default(),
                    sql: String::new(),
                    info: String::new(),
                    info_erro: false,
                    vista: None,
                    sql_orig: String::new(),
                    ordem: None,
                    filtros: Vec::new(),
                    filtro_ativo: true,
                    estr: Some(est),
                };
                match existente {
                    Some(i) => {
                        self.docs[i] = doc;
                        self.active = i;
                    }
                    None => {
                        self.docs.push(doc);
                        self.active = self.docs.len() - 1;
                    }
                }
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
            sql_orig: String::new(),
            ordem: None,
            filtros: Vec::new(),
            filtro_ativo: true,
            estr: Some(est),
        });
        self.active = self.docs.len() - 1;
        self.edit = None;
        self.row_sel = false;
    }

    /// Botão «Vista»: troca o modo do separador ativo (dados <-> estrutura, ou
    /// dados <-> SQL nas consultas). O objeto continua com um só separador.
    pub(crate) fn alternar_vista(&mut self) {
        let info = self
            .docs
            .get(self.active)
            .map(|d| (d.kind, d.name.clone(), d.estr.as_ref().map_or(false, |e| e.nova), d.vista.is_some()));
        match info {
            Some((ObjKind::Table, n, _, _)) => self.abrir_estrutura(&n),
            Some((ObjKind::Design, n, false, _)) => self.open_doc(ObjKind::Table, n),
            Some((ObjKind::Design, _, true, _)) => self.msg = "Crie primeiro a tabela".into(),
            Some((ObjKind::Query, n, _, _)) => self.abrir_vista_sql(&n),
            Some((ObjKind::Sql, n, _, true)) => self.open_doc(ObjKind::Query, n),
            Some((ObjKind::Sql, _, _, false)) => self.msg = "Guarde a consulta para a poder ver como folha de dados".into(),
            None => self.msg = "Não há nenhum separador aberto".into(),
        }
    }

    /// Grupo «Vistas» do ribbon.
    pub(crate) fn grupo_vistas(&mut self, ui: &mut Ui) {
        let (rotulo, icone) = match self.docs.get(self.active).map(|d| (d.kind, d.vista.is_some())) {
            Some((ObjKind::Table, _)) => ("Vista\nEstrutura", Icon::Badge("E", C_GREEN)),
            Some((ObjKind::Design, _)) => ("Vista Folha\nde Dados", Icon::Table),
            Some((ObjKind::Query, _)) => ("Vista\nSQL", Icon::Badge("S", C_BLUE)),
            Some((ObjKind::Sql, true)) => ("Vista Folha\nde Dados", Icon::Table),
            _ => ("Vista", Icon::Table),
        };
        group(ui, "Vistas", |ui| {
            if big_btn(ui, icone, rotulo, false).clicked() {
                self.alternar_vista();
            }
        });
    }

    /// Volta a ler as folhas de dados abertas de `tabela` (a estrutura mudou).
    pub(crate) fn recarregar_abertos(&mut self, tabela: &str) {
        let idx: Vec<usize> = self
            .docs
            .iter()
            .enumerate()
            .filter(|(_, d)| d.kind == ObjKind::Table && d.name == tabela)
            .map(|(i, _)| i)
            .collect();
        for i in idx {
            self.carregar_doc(i);
        }
    }

    pub(crate) fn estrutura_view(&mut self, ui: &mut Ui) {
        let f2 = !self.foco_nav && !a_escrever(ui.ctx()) && ui.input(|i| i.key_pressed(egui::Key::F2));
        let Some(mut est) = self.docs[self.active].estr.take() else {
            ui.label("Sem dados de estrutura.");
            return;
        };
        let tabela = self.docs[self.active].name.clone();
        // F2: muda o nome do campo selecionado (ou vai para a caixa de texto do nome)
        if f2 {
            match (est.nova, est.sel) {
                (false, Some(i)) if i < est.campos.len() => {
                    est.renomear = Some((i, est.campos[i].nome.clone()));
                    est.foco_ren = true;
                }
                (true, Some(i)) if i < est.campos.len() => {
                    ui.memory_mut(|m| m.request_focus(egui::Id::new(("campo_nome", i))));
                }
                (true, None) => ui.memory_mut(|m| m.request_focus(egui::Id::new("nome_tabela"))),
                _ => {}
            }
        }
        if est.renomear.is_some() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            est.renomear = None;
        }
        let mut ac: Option<Ac> = None;
        egui::ScrollArea::vertical().id_source("estr_scroll").auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(6.0);
            if est.nova {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new("Nome da tabela:").strong());
                    ui.add(egui::TextEdit::singleline(&mut est.nome_tabela).id(egui::Id::new("nome_tabela")).desired_width(200.0));
                });
            } else {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("Estrutura da tabela «{}»", tabela)).strong().size(15.0));
                    ui.weak("  (F2 muda o nome do campo selecionado)");
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
                            if ui.selectable_label(est.sel == Some(i), if c.pk { "🔑" } else { "▸" }).clicked() {
                                est.sel = Some(i);
                            }
                            let r = ui.add(
                                egui::TextEdit::singleline(&mut c.nome)
                                    .id(egui::Id::new(("campo_nome", i)))
                                    .desired_width(170.0),
                            );
                            if r.gained_focus() {
                                est.sel = Some(i);
                            }
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
                                        let mut enter = false;
                                        if let Some((_, t)) = est.renomear.as_mut() {
                                            let r = ui.add(
                                                egui::TextEdit::singleline(t)
                                                    .id(egui::Id::new("renomear_campo"))
                                                    .desired_width(120.0),
                                            );
                                            if est.foco_ren {
                                                r.request_focus();
                                                est.foco_ren = false;
                                            }
                                            enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                                        }
                                        (ui.small_button("OK").clicked() || enter, ui.small_button("✕").clicked())
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
                            } else if ui.selectable_label(est.sel == Some(i), &c.nome).clicked() {
                                est.sel = Some(i);
                            }
                            ui.label(&c.tipo);
                            ui.label(if c.obrigatorio { "Sim" } else { "Não" });
                            ui.label(&c.omissao);
                            ui.label(if c.pk { "Sim" } else { "" });
                            ui.horizontal(|ui| {
                                if ui.small_button("Renomear").clicked() {
                                    est.sel = Some(i);
                                    est.renomear = Some((i, c.nome.clone()));
                                    est.foco_ren = true;
                                }
                                if ui.small_button("Eliminar").clicked() {
                                    est.sel = Some(i);
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
                    ui.weak("Uma chave primária INTEGER é numerada automaticamente. Valor por omissão: número, texto, NULL ou CURRENT_TIMESTAMP. F2 vai para o nome do campo selecionado.");
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
                est.sel = None;
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
                        est.sel = None;
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
                        est.sel = None;
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
