//! Relações: ver as chaves estrangeiras, ligar ou desligar a integridade referencial
//! e verificar se há registos que referem valores inexistentes.

use crate::{db::Relacao, estado::*};
use eframe::egui::{self, Align2, Color32, RichText, Vec2};

/// Estado da janela «Relações».
#[derive(Default)]
pub(crate) struct EstRelacoes {
    pub(crate) lista: Vec<Relacao>,
    pub(crate) impor: bool,
    pub(crate) violacoes: Option<Vec<String>>,
    pub(crate) info: String,
    pub(crate) info_erro: bool,
}

enum AcR {
    Impor(bool),
    Verificar,
}

impl AccessApp {
    pub(crate) fn abrir_relacoes(&mut self) {
        let Some(db) = self.db.as_ref() else {
            self.msg = "Abra ou crie uma base de dados primeiro".into();
            return;
        };
        let mut e = EstRelacoes { impor: db.foreign_keys_ativas(), ..Default::default() };
        match db.lista_relacoes() {
            Ok(l) => e.lista = l,
            Err(err) => {
                e.info = err;
                e.info_erro = true;
            }
        }
        self.relacoes = Some(e);
    }

    pub(crate) fn dialogo_relacoes(&mut self, ctx: &egui::Context) {
        let Some(mut e) = self.relacoes.take() else { return };
        let mut aberto = true;
        let mut ac: Option<AcR> = None;
        egui::Window::new("Relações")
            .open(&mut aberto)
            .collapsible(false)
            .resizable(true)
            .default_width(640.0)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                let antes = e.impor;
                ui.checkbox(&mut e.impor, "Impor a integridade referencial (não deixa eliminar nem criar registos que quebrem uma relação)");
                if e.impor != antes {
                    ac = Some(AcR::Impor(e.impor));
                }
                ui.separator();
                if e.lista.is_empty() {
                    ui.label("Não há relações definidas nesta base de dados.");
                    ui.weak("Crie-as ao desenhar uma tabela (Criar → Estrutura da Tabela) ou ao acrescentar um campo na Vista Estrutura: escolha o campo «Refere a».");
                } else {
                    egui::ScrollArea::both().id_salt("relacoes_scroll").max_height(240.0).show(ui, |ui| {
                        egui::Grid::new("relacoes_grid").striped(true).spacing([16.0, 5.0]).show(ui, |ui| {
                            for h in ["Tabela", "Campo", "Refere a", "Ao eliminar", "Ao atualizar"] {
                                ui.label(RichText::new(h).strong());
                            }
                            ui.end_row();
                            for r in &e.lista {
                                ui.label(&r.tabela);
                                ui.label(&r.colunas);
                                ui.label(format!("{}({})", r.ref_tabela, r.ref_colunas));
                                ui.label(&r.ao_apagar);
                                ui.label(&r.ao_atualizar);
                                ui.end_row();
                            }
                        });
                    });
                }
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Verificar integridade").clicked() {
                        ac = Some(AcR::Verificar);
                    }
                    ui.weak("Procura registos que refiram valores que já não existem.");
                });
                if let Some(v) = &e.violacoes {
                    ui.add_space(4.0);
                    if v.is_empty() {
                        ui.label(RichText::new("Nenhum problema encontrado.").color(Color32::from_rgb(0x2B, 0x7A, 0x3D)));
                    } else {
                        ui.label(RichText::new(format!("{} problema(s):", v.len())).color(Color32::from_rgb(0xB0, 0x00, 0x20)));
                        egui::ScrollArea::vertical().id_salt("viol_scroll").max_height(120.0).show(ui, |ui| {
                            for l in v {
                                ui.label(l);
                            }
                        });
                    }
                }
                if !e.info.is_empty() {
                    ui.add_space(4.0);
                    ui.label(RichText::new(e.info.clone()).color(if e.info_erro {
                        Color32::from_rgb(0xB0, 0x00, 0x20)
                    } else {
                        Color32::from_rgb(0x2B, 0x5A, 0x9E)
                    }));
                }
            });
        if !aberto {
            return;
        }
        match ac {
            Some(AcR::Impor(on)) => {
                let r = self.db.as_ref().map(|db| db.definir_foreign_keys(on));
                match r {
                    Some(Ok(())) => {
                        e.info = if on { "Integridade referencial ligada".into() } else { "Integridade referencial desligada".into() };
                        e.info_erro = false;
                    }
                    Some(Err(err)) => {
                        e.info = err;
                        e.info_erro = true;
                        e.impor = !on;
                    }
                    None => {}
                }
            }
            Some(AcR::Verificar) => {
                let r = self.db.as_ref().map(|db| db.verificar_relacoes());
                match r {
                    Some(Ok(v)) => e.violacoes = Some(v),
                    Some(Err(err)) => {
                        e.info = err;
                        e.info_erro = true;
                    }
                    None => {}
                }
            }
            None => {}
        }
        self.relacoes = Some(e);
    }
}
