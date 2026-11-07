//! Vista SQL: editor, execução, resultados e consultas guardadas como VIEW.

use crate::{db::TableData, estado::*, tema::*};
use eframe::egui::{self, Align2, FontId, RichText, Ui, Vec2};

impl AccessApp {
    /// Abre um separador novo com o editor SQL (e `sql` como texto inicial).
    pub(crate) fn nova_consulta_sql(&mut self, sql: &str) {
        if self.db.is_none() {
            self.msg = "Abra ou crie uma base de dados primeiro".into();
            return;
        }
        self.n_consulta += 1;
        let nome = format!("Consulta{}", self.n_consulta);
        self.docs.push(Doc {
            kind: ObjKind::Sql,
            name: nome,
            data: TableData::default(),
            sql: sql.to_string(),
            info: String::new(),
            info_erro: false,
            vista: None,
            estr: None,
        });
        self.active = self.docs.len() - 1;
        self.sel = (0, 1);
        self.edit = None;
        self.row_sel = false;
    }

    /// Abre o texto SQL de uma consulta guardada (VIEW) para a editar.
    pub(crate) fn abrir_vista_sql(&mut self, nome: &str) {
        if let Some(i) = self.docs.iter().position(|d| d.kind == ObjKind::Sql && d.vista.as_deref() == Some(nome)) {
            self.active = i;
            return;
        }
        let sql = match self.db.as_ref().map(|d| d.view_sql(nome)) {
            Some(Ok(s)) => s,
            Some(Err(e)) => {
                self.msg = format!("Não foi possível ler a consulta «{}»: {}", nome, e);
                return;
            }
            None => return,
        };
        self.docs.push(Doc {
            kind: ObjKind::Sql,
            name: nome.to_string(),
            data: TableData::default(),
            sql,
            info: String::new(),
            info_erro: false,
            vista: Some(nome.to_string()),
            estr: None,
        });
        self.active = self.docs.len() - 1;
        self.sel = (0, 1);
        self.edit = None;
        self.row_sel = false;
    }

    /// Executa o texto do editor do separador ativo.
    pub(crate) fn executar_sql(&mut self) {
        let Some(doc) = self.docs.get(self.active) else { return };
        let sql = doc.sql.clone();
        let res = match self.db.as_ref() {
            Some(db) => db.executar(&sql),
            None => {
                self.msg = "Nenhuma base de dados aberta".into();
                return;
            }
        };
        let doc = &mut self.docs[self.active];
        match res {
            Ok(r) => {
                let mut partes: Vec<String> = Vec::new();
                if let Some(d) = &r.dados {
                    let lim = if d.truncado { " (limitado às primeiras 100000)" } else { "" };
                    partes.push(format!("{} linha(s){}", d.rows.len(), lim));
                }
                if r.afetadas > 0 {
                    partes.push(format!("{} linha(s) afetada(s)", r.afetadas));
                }
                doc.info = if partes.is_empty() {
                    format!("{} instrução(ões) executada(s)", r.instrucoes)
                } else {
                    partes.join(" · ")
                };
                doc.info_erro = false;
                doc.data = r.dados.unwrap_or_default();
            }
            Err(e) => {
                doc.info = e;
                doc.info_erro = true;
            }
        }
        self.sel = (0, 1);
        self.row_sel = false;
        self.refresh_nav();
    }

    /// «Guardar como Consulta»: grava logo se já tem nome, senão pede o nome.
    pub(crate) fn guardar_consulta(&mut self) {
        let Some(doc) = self.docs.get(self.active) else { return };
        match doc.vista.clone() {
            Some(nome) => {
                self.gravar_vista(&nome, true);
            }
            None => self.pedir_nome = Some(String::new()),
        }
    }

    fn gravar_vista(&mut self, nome: &str, substituir: bool) -> bool {
        let sql = self.docs[self.active].sql.clone();
        let r = match self.db.as_ref() {
            Some(db) => db.guardar_vista(nome, &sql, substituir),
            None => return false,
        };
        match r {
            Ok(()) => {
                let nome = nome.trim().to_string();
                let doc = &mut self.docs[self.active];
                doc.vista = Some(nome.clone());
                doc.name = nome.clone();
                doc.info = format!("Consulta «{}» guardada", nome);
                doc.info_erro = false;
                self.msg = doc.info.clone();
                self.refresh_nav();
                true
            }
            Err(e) => {
                let doc = &mut self.docs[self.active];
                doc.info = e.clone();
                doc.info_erro = true;
                self.msg = e;
                false
            }
        }
    }

    pub(crate) fn dialogo_guardar_consulta(&mut self, ctx: &egui::Context) {
        let Some(mut nome) = self.pedir_nome.clone() else { return };
        let mut acao: Option<bool> = None;
        egui::Window::new("Guardar como")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Nome da consulta:");
                let r = ui.add(egui::TextEdit::singleline(&mut nome).id(egui::Id::new("nome_consulta")).desired_width(260.0));
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
        self.pedir_nome = Some(nome.clone());
        match acao {
            Some(true) => {
                if self.gravar_vista(&nome, false) {
                    self.pedir_nome = None;
                }
            }
            Some(false) => self.pedir_nome = None,
            None => {}
        }
    }

    /// Separador de consulta: barra de botões, editor SQL e grelha de resultados.
    pub(crate) fn sql_view(&mut self, ui: &mut Ui) {
        // F5 ou Ctrl+Enter executam (consumidos para não escreverem no editor)
        let correr = ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter) || i.consume_key(egui::Modifiers::NONE, egui::Key::F5)
        });
        let mut executar = correr;
        let mut guardar = false;
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            if ui.add(egui::Button::new("▶ Executar").min_size(Vec2::new(90.0, 24.0))).on_hover_text("F5 ou Ctrl+Enter").clicked() {
                executar = true;
            }
            if ui.add(egui::Button::new("Guardar como Consulta").min_size(Vec2::new(150.0, 24.0))).clicked() {
                guardar = true;
            }
        });
        ui.add_space(4.0);
        let altura = (ui.available_height() * 0.38).clamp(90.0, 320.0);
        let mut sql = std::mem::take(&mut self.docs[self.active].sql);
        egui::ScrollArea::vertical().id_source("sql_scroll").max_height(altura).auto_shrink([false, true]).show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut sql)
                    .id(egui::Id::new("sql_editor"))
                    .font(FontId::monospace(13.0))
                    .desired_width(f32::INFINITY)
                    .desired_rows(((altura / 16.0) as usize).max(4))
                    .lock_focus(true)
                    .hint_text("Escreva aqui a consulta SQL (F5 ou Ctrl+Enter para executar)"),
            );
        });
        self.docs[self.active].sql = sql;
        if executar {
            self.executar_sql();
        }
        if guardar {
            self.guardar_consulta();
        }
        let (info, erro) = {
            let d = &self.docs[self.active];
            (d.info.clone(), d.info_erro)
        };
        ui.add_space(4.0);
        if !info.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.add_space(8.0);
                ui.label(RichText::new(info).color(if erro { rgb(0xB00020) } else { rgb(0x2B5A9E) }));
            });
            ui.add_space(2.0);
        }
        self.datasheet(ui, "");
    }
}
