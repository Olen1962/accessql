//! Duplicar tabelas e consultas guardadas no painel de navegação.

use crate::estado::*;
use eframe::egui::{self, Align2, Vec2};

/// Nome proposto para a cópia (o utilizador pode mudar no diálogo).
pub(crate) fn nome_sugerido(nome: &str) -> String {
    format!("{}_copia", nome)
}

impl AccessApp {
    /// Pede o nome da cópia do objeto selecionado no painel de navegação.
    pub(crate) fn pedir_duplicar_nav(&mut self) {
        let Some((g, i)) = self.nav_sel else { return };
        let Some(grp) = self.nav.get(g) else { return };
        let Some(nome) = grp.items.get(i).cloned() else {
            return;
        };
        let kind = grp.kind;
        self.pedir_duplicar_objeto(kind, &nome);
    }

    /// Abre o diálogo para duplicar um objeto conhecido (menu de contexto ou atalho).
    pub(crate) fn pedir_duplicar_objeto(&mut self, kind: ObjKind, nome: &str) {
        self.duplicar_obj = Some((kind, nome.to_string(), nome_sugerido(nome)));
    }

    pub(crate) fn dialogo_duplicar(&mut self, ctx: &egui::Context) {
        let Some((kind, de, mut texto)) = self.duplicar_obj.clone() else {
            return;
        };
        let mut acao: Option<bool> = None;
        egui::Window::new("Duplicar")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                let tipo = if kind == ObjKind::Table {
                    "tabela"
                } else {
                    "consulta"
                };
                ui.label(format!("Nome para a cópia da {} «{}»:", tipo, de));
                let r = ui.add(
                    egui::TextEdit::singleline(&mut texto)
                        .id(egui::Id::new("nome_duplicar"))
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
        self.duplicar_obj = Some((kind, de.clone(), texto.clone()));
        match acao {
            Some(true) => {
                if self.duplicar_objeto(kind, &de, &texto) {
                    self.duplicar_obj = None;
                }
            }
            Some(false) => self.duplicar_obj = None,
            None => {}
        }
    }

    /// Devolve `true` se o diálogo se pode fechar.
    fn duplicar_objeto(&mut self, kind: ObjKind, de: &str, para: &str) -> bool {
        let para = para.trim();
        if para.is_empty() {
            self.msg = "Indique um nome para a cópia".into();
            return false;
        }
        let r = match self.db.as_ref() {
            Some(db) if kind == ObjKind::Table => db.duplicar_tabela(de, para),
            Some(db) => db.duplicar_vista(de, para),
            None => return true,
        };
        match r {
            Ok(()) => {
                self.limpar_historico();
                self.refresh_nav();
                for (g, grp) in self.nav.iter().enumerate() {
                    if let Some(i) = grp.items.iter().position(|n| n == para) {
                        self.nav_sel = Some((g, i));
                    }
                }
                self.foco_nav = false;
                if kind == ObjKind::Table {
                    self.open_doc(ObjKind::Table, para.to_string());
                } else {
                    self.abrir_vista_sql(para);
                }
                self.msg = format!("«{}» duplicada como «{}»", de, para);
                true
            }
            Err(e) => {
                self.msg = e;
                false
            }
        }
    }
}
