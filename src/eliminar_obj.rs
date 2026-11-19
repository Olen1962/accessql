//! Eliminar tabelas e consultas, com confirmação e proteção das relações.

use crate::estado::*;
use eframe::egui::{self, Align2, Color32, RichText, Vec2};

/// Pedido de eliminação de uma tabela ou consulta, à espera de confirmação.
pub(crate) struct EliminarObj {
    pub(crate) kind: ObjKind,
    pub(crate) nome: String,
    pub(crate) registos: Option<i64>,
    /// Motivo pelo qual não se pode eliminar (a tabela é referida por outras).
    pub(crate) bloqueio: Option<String>,
    /// Consultas que mencionam o objeto e deixarão de funcionar.
    pub(crate) afetadas: Vec<String>,
}

impl AccessApp {
    /// Delete (ou «Eliminar» do menu) sobre o objeto selecionado no painel de navegação.
    pub(crate) fn pedir_eliminar_nav(&mut self) {
        let Some((g, i)) = self.nav_sel else { return };
        let Some(grp) = self.nav.get(g) else { return };
        let Some(nome) = grp.items.get(i) else { return };
        let (kind, nome) = (grp.kind, nome.clone());
        self.pedir_eliminar_objeto(kind, nome);
    }

    pub(crate) fn pedir_eliminar_objeto(&mut self, kind: ObjKind, nome: String) {
        let Some(db) = self.db.as_ref() else { return };
        let mut e = EliminarObj { kind, nome: nome.clone(), registos: None, bloqueio: None, afetadas: Vec::new() };
        if kind == ObjKind::Table {
            e.registos = db.contar_registos(&nome).ok();
            if let Ok(rels) = db.lista_relacoes() {
                // tabelas que guardam uma chave estrangeira para esta (uma tabela que se refere a si própria não conta)
                let mut filhas: Vec<String> = Vec::new();
                let mut detalhes: Vec<String> = Vec::new();
                for r in rels.iter().filter(|r| r.ref_tabela.eq_ignore_ascii_case(&nome) && !r.tabela.eq_ignore_ascii_case(&nome)) {
                    let d = format!("«{}» (campo {})", r.tabela, r.colunas);
                    if !detalhes.contains(&d) {
                        detalhes.push(d);
                    }
                    if !filhas.contains(&r.tabela) {
                        filhas.push(r.tabela.clone());
                    }
                }
                if !detalhes.is_empty() {
                    let quais = filhas.iter().map(|t| format!("«{}»", t)).collect::<Vec<_>>().join(", ");
                    e.bloqueio = Some(format!(
                        "A tabela «{}» tem relações: é referida por {}.\n\nPara a eliminar, elimine primeiro {}, que guarda{} a relação.",
                        nome,
                        detalhes.join(", "),
                        quais,
                        if filhas.len() > 1 { "m" } else { "" }
                    ));
                }
            }
        }
        e.afetadas = db.consultas_que_mencionam(&nome).unwrap_or_default();
        self.eliminar_obj = Some(e);
    }

    pub(crate) fn dialogo_eliminar_obj(&mut self, ctx: &egui::Context) {
        let Some(e) = self.eliminar_obj.take() else { return };
        let tipo = if e.kind == ObjKind::Table { "tabela" } else { "consulta" };
        let mut acao: Option<bool> = None;
        egui::Window::new(format!("Eliminar {}", tipo))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_max_width(440.0);
                if let Some(b) = &e.bloqueio {
                    ui.colored_label(Color32::from_rgb(0xB0, 0x00, 0x20), b.clone());
                    ui.add_space(8.0);
                    if ui.button("Fechar").clicked() {
                        acao = Some(false);
                    }
                } else {
                    let regs = match e.registos {
                        Some(n) => format!(" e os seus {} registo(s)", n),
                        None => String::new(),
                    };
                    ui.label(format!("Vai eliminar a {} «{}»{}.", tipo, e.nome, regs));
                    ui.label(RichText::new("Esta operação não pode ser anulada.").strong());
                    if !e.afetadas.is_empty() {
                        ui.add_space(6.0);
                        ui.colored_label(
                            Color32::from_rgb(0xB4, 0x5F, 0x06),
                            format!("Estas consultas usam «{}» e deixarão de funcionar: {}.", e.nome, e.afetadas.join(", ")),
                        );
                    }
                    ui.add_space(6.0);
                    ui.weak("Se quiser uma cópia de segurança, use Ficheiro → Guardar Cópia Como antes de eliminar.");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let b = egui::Button::new(RichText::new("Eliminar").color(Color32::WHITE)).fill(Color32::from_rgb(0xB0, 0x20, 0x20));
                        if ui.add(b).clicked() {
                            acao = Some(true);
                        }
                        if ui.button("Cancelar").clicked() {
                            acao = Some(false);
                        }
                    });
                }
            });
        match acao {
            Some(true) => {
                let (k, n) = (e.kind, e.nome.clone());
                self.eliminar_objeto_agora(k, &n);
            }
            Some(false) => {}
            None => self.eliminar_obj = Some(e),
        }
    }

    fn eliminar_objeto_agora(&mut self, kind: ObjKind, nome: &str) {
        let r = self.db.as_ref().map(|db| db.eliminar_objeto(kind == ObjKind::Table, nome));
        match r {
            Some(Ok(())) => {
                // fecha os separadores do objeto, seja qual for o modo em que estejam
                while let Some(i) = self.procurar_objeto(nome) {
                    self.docs.remove(i);
                }
                if self.active >= self.docs.len() {
                    self.active = self.docs.len().saturating_sub(1);
                }
                self.sel = (0, 0);
                self.scroll_sel = true;
                self.edit = None;
                self.row_sel = false;
                self.confirm_del = None;
                if self.indices.as_ref().map_or(false, |e| e.tabela.eq_ignore_ascii_case(nome)) {
                    self.indices = None;
                }
                self.relacoes = None;
                self.limpar_historico();
                self.refresh_nav();
                self.msg = format!("«{}» eliminad{}", nome, if kind == ObjKind::Table { "o" } else { "a" });
            }
            Some(Err(e)) => self.msg = e,
            None => {}
        }
    }
}
