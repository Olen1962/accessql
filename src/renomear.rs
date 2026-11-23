//! Mudar o nome de tabelas e consultas (F2 no painel de navegação).

use crate::estado::*;
use eframe::egui::{self, Align2, Vec2};

impl AccessApp {
    /// F2 no painel de navegação: pede o novo nome do objeto selecionado.
    pub(crate) fn pedir_renomear_nav(&mut self) {
        let Some((g, i)) = self.nav_sel else { return };
        let Some(grp) = self.nav.get(g) else { return };
        let Some(nome) = grp.items.get(i) else { return };
        self.renomear_obj = Some((grp.kind, nome.clone(), nome.clone()));
    }

    pub(crate) fn dialogo_renomear(&mut self, ctx: &egui::Context) {
        let Some((kind, de, mut texto)) = self.renomear_obj.clone() else {
            return;
        };
        let mut acao: Option<bool> = None;
        egui::Window::new("Mudar o Nome")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(format!("Novo nome para «{}»:", de));
                let r = ui.add(
                    egui::TextEdit::singleline(&mut texto)
                        .id(egui::Id::new("nome_objeto"))
                        .desired_width(260.0),
                );
                r.request_focus();
                if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    acao = Some(true);
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        acao = Some(true);
                    }
                    if ui.button("Cancelar").clicked() {
                        acao = Some(false);
                    }
                });
            });
        self.renomear_obj = Some((kind, de.clone(), texto.clone()));
        match acao {
            Some(true) => {
                if self.renomear_objeto(kind, &de, &texto) {
                    self.renomear_obj = None;
                }
            }
            Some(false) => self.renomear_obj = None,
            None => {}
        }
    }

    /// Devolve `true` se o diálogo se pode fechar (mudou o nome ou não havia nada a mudar).
    fn renomear_objeto(&mut self, kind: ObjKind, de: &str, para: &str) -> bool {
        let para = para.trim();
        if para == de {
            return true;
        }
        let r = match self.db.as_ref() {
            Some(db) => db.renomear_objeto(kind == ObjKind::Table, de, para),
            None => return true,
        };
        match r {
            Ok(()) => {
                for d in self.docs.iter_mut() {
                    let do_objeto = match d.kind {
                        ObjKind::Table | ObjKind::Query => true,
                        ObjKind::Design => d.estr.as_ref().map_or(false, |e| !e.nova),
                        ObjKind::Sql => d.vista.as_deref() == Some(de),
                        ObjKind::QueryDesign => d.qbe.is_some(),
                    };
                    if do_objeto && d.name == de {
                        d.name = para.to_string();
                        if d.kind == ObjKind::Sql {
                            d.vista = Some(para.to_string());
                        }
                    }
                }
                self.limpar_historico();
                self.refresh_nav();
                for (g, grp) in self.nav.iter().enumerate() {
                    if let Some(i) = grp.items.iter().position(|n| n == para) {
                        self.nav_sel = Some((g, i));
                    }
                }
                self.msg = format!("«{}» passou a chamar-se «{}»", de, para);
                true
            }
            Err(e) => {
                self.msg = e;
                false
            }
        }
    }
}
