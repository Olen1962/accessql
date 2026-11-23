//! Índices: ver, criar e eliminar os índices de uma tabela.

use crate::{db::Indice, estado::*};
use eframe::egui::{self, Align2, RichText, Vec2};

/// Estado da janela «Índices».
#[derive(Default)]
pub(crate) struct EstIndices {
    pub(crate) tabela: String,
    pub(crate) lista: Vec<Indice>,
    pub(crate) colunas: Vec<String>,
    pub(crate) sel: Vec<bool>,
    pub(crate) nome: String,
    pub(crate) unico: bool,
    pub(crate) eliminar: Option<String>,
    pub(crate) info: String,
    pub(crate) info_erro: bool,
}

enum AcI {
    Trocar,
    Criar,
    Eliminar(String),
}

impl AccessApp {
    /// Abre a janela de índices da tabela do separador ativo (ou da primeira tabela).
    pub(crate) fn abrir_indices(&mut self) {
        if self.db.is_none() {
            self.msg = "Abra ou crie uma base de dados primeiro".into();
            return;
        }
        let ativa = self.docs.get(self.active).and_then(|d| match d.kind {
            ObjKind::Table => Some(d.name.clone()),
            ObjKind::Design if d.estr.as_ref().map_or(false, |e| !e.nova) => Some(d.name.clone()),
            _ => None,
        });
        let tabela = ativa.or_else(|| self.nav.first().and_then(|g| g.items.first().cloned()));
        let Some(tabela) = tabela else {
            self.msg = "Não há tabelas".into();
            return;
        };
        let mut e = EstIndices {
            tabela,
            ..Default::default()
        };
        self.carregar_indices(&mut e);
        self.indices = Some(e);
    }

    fn carregar_indices(&self, e: &mut EstIndices) {
        let Some(db) = self.db.as_ref() else { return };
        match db.lista_indices(&e.tabela) {
            Ok(l) => e.lista = l,
            Err(err) => {
                e.info = err;
                e.info_erro = true;
            }
        }
        if let Ok((campos, _)) = db.info_tabela(&e.tabela) {
            e.colunas = campos.into_iter().map(|c| c.nome).collect();
        }
        e.sel = vec![false; e.colunas.len()];
        e.eliminar = None;
    }

    pub(crate) fn dialogo_indices(&mut self, ctx: &egui::Context) {
        let Some(mut e) = self.indices.take() else {
            return;
        };
        let tabelas: Vec<String> = self
            .nav
            .first()
            .map(|g| g.items.clone())
            .unwrap_or_default();
        let mut aberto = true;
        let mut ac: Option<AcI> = None;
        egui::Window::new("Índices")
            .open(&mut aberto)
            .collapsible(false)
            .resizable(true)
            .default_width(560.0)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Tabela:").strong());
                    egui::ComboBox::from_id_salt("indices_tabela")
                        .width(200.0)
                        .selected_text(e.tabela.clone())
                        .show_ui(ui, |ui| {
                            for t in &tabelas {
                                if ui.selectable_value(&mut e.tabela, t.clone(), t).changed() {
                                    ac = Some(AcI::Trocar);
                                }
                            }
                        });
                });
                ui.separator();
                if let Some(nome) = e.eliminar.clone() {
                    ui.horizontal_wrapped(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(0xB0, 0x00, 0x20),
                            format!("Eliminar o índice «{}»?", nome),
                        );
                        if ui.button("Sim, eliminar").clicked() {
                            ac = Some(AcI::Eliminar(nome.clone()));
                        }
                        if ui.button("Cancelar").clicked() {
                            e.eliminar = None;
                        }
                    });
                }
                if e.lista.is_empty() {
                    ui.label("Esta tabela ainda não tem índices.");
                } else {
                    egui::Grid::new("indices_grid")
                        .striped(true)
                        .spacing([16.0, 5.0])
                        .show(ui, |ui| {
                            for h in ["Nome", "Campos", "Único", "Origem", ""] {
                                ui.label(RichText::new(h).strong());
                            }
                            ui.end_row();
                            for ix in &e.lista {
                                ui.label(&ix.nome);
                                ui.label(ix.colunas.join(", "));
                                ui.label(if ix.unico { "Sim" } else { "Não" });
                                ui.label(match ix.origem.as_str() {
                                    "c" => "criado por si",
                                    "u" => "restrição UNIQUE",
                                    "pk" => "chave primária",
                                    _ => "outra",
                                });
                                if ix.origem == "c" {
                                    if ui.small_button("Eliminar").clicked() {
                                        e.eliminar = Some(ix.nome.clone());
                                    }
                                } else {
                                    ui.label("");
                                }
                                ui.end_row();
                            }
                        });
                }
                ui.separator();
                ui.label(RichText::new("Criar um índice novo").strong());
                ui.horizontal_wrapped(|ui| {
                    ui.label("Campos:");
                    for (i, c) in e.colunas.iter().enumerate() {
                        if let Some(s) = e.sel.get_mut(i) {
                            ui.checkbox(s, c);
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Nome:");
                    ui.add(
                        egui::TextEdit::singleline(&mut e.nome)
                            .id(egui::Id::new("nome_indice"))
                            .hint_text("(opcional)")
                            .desired_width(200.0),
                    );
                    ui.checkbox(&mut e.unico, "Único");
                    if ui.button("Criar índice").clicked() {
                        ac = Some(AcI::Criar);
                    }
                });
                if !e.info.is_empty() {
                    ui.add_space(4.0);
                    ui.label(RichText::new(e.info.clone()).color(if e.info_erro {
                        egui::Color32::from_rgb(0xB0, 0x00, 0x20)
                    } else {
                        egui::Color32::from_rgb(0x2B, 0x5A, 0x9E)
                    }));
                }
            });
        if !aberto {
            return;
        }
        match ac {
            Some(AcI::Trocar) => {
                e.info.clear();
                self.carregar_indices(&mut e);
            }
            Some(AcI::Criar) => {
                let cols: Vec<String> = e
                    .colunas
                    .iter()
                    .zip(e.sel.iter())
                    .filter(|(_, s)| **s)
                    .map(|(c, _)| c.clone())
                    .collect();
                let nome = if e.nome.trim().is_empty() {
                    format!("idx_{}_{}", e.tabela, cols.join("_"))
                } else {
                    e.nome.trim().to_string()
                };
                let r = self
                    .db
                    .as_ref()
                    .map(|db| db.criar_indice(&nome, &e.tabela, &cols, e.unico));
                match r {
                    Some(Ok(())) => {
                        self.carregar_indices(&mut e);
                        e.info = format!("Índice «{}» criado", nome);
                        e.info_erro = false;
                        e.nome.clear();
                        e.unico = false;
                    }
                    Some(Err(err)) => {
                        e.info = err;
                        e.info_erro = true;
                    }
                    None => {}
                }
            }
            Some(AcI::Eliminar(nome)) => {
                let r = self.db.as_ref().map(|db| db.eliminar_indice(&nome));
                match r {
                    Some(Ok(())) => {
                        self.carregar_indices(&mut e);
                        e.info = format!("Índice «{}» eliminado", nome);
                        e.info_erro = false;
                    }
                    Some(Err(err)) => {
                        e.info = err;
                        e.info_erro = true;
                        e.eliminar = None;
                    }
                    None => {}
                }
            }
            None => {}
        }
        self.indices = Some(e);
    }
}
