use eframe::egui::{self, Color32, FontId, Rect, Response, Stroke, Ui, Vec2};
use std::path::PathBuf;
use crate::{db::{Database, TableData}, icones::*, tema::*, widgets::*};
// ───────────────────────────── Estado da aplicação ─────────────────────────────
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ObjKind {
    Table,
    Query,
    Sql,
    Design,
}
impl ObjKind {
    pub(crate) fn icon(self) -> Icon {
        match self {
            ObjKind::Table => Icon::Table,
            ObjKind::Query => Icon::Query,
            ObjKind::Sql => Icon::Badge("S", C_BLUE),
            ObjKind::Design => Icon::Badge("E", C_GREEN),
        }
    }
}
#[derive(Clone)]
pub(crate) struct NavGroup {
    pub(crate) name: &'static str,
    pub(crate) kind: ObjKind,
    pub(crate) items: Vec<String>,
}
pub(crate) struct Doc {
    pub(crate) kind: ObjKind,
    pub(crate) name: String,
    pub(crate) data: TableData,
    /// Texto SQL (só nos separadores de consulta).
    pub(crate) sql: String,
    /// Mensagem sob o editor SQL (resultado ou erro).
    pub(crate) info: String,
    pub(crate) info_erro: bool,
    /// Nome da consulta guardada (VIEW) a que este separador corresponde.
    pub(crate) vista: Option<String>,
    /// Estado da Vista Estrutura (só nos separadores de estrutura).
    pub(crate) estr: Option<crate::estrutura::Estrutura>,
}

/// Célula da folha de dados que está a ser editada.
pub(crate) struct Edicao {
    pub(crate) row: usize,
    pub(crate) col: usize,
    pub(crate) texto: String,
    pub(crate) foco: bool,
}
pub(crate) struct AccessApp {
    pub(crate) tab: usize,
    pub(crate) tab_x: (f32, f32),
    pub(crate) backstage: bool,
    pub(crate) bs_page: usize,
    pub(crate) nav_filter: String,
    pub(crate) db: Option<Database>,
    pub(crate) db_file: Option<PathBuf>,
    pub(crate) recentes: Vec<PathBuf>,
    pub(crate) new_db_dir: String,
    pub(crate) nav: Vec<NavGroup>,
    pub(crate) nav_open: [bool; 2],
    pub(crate) nav_sel: Option<(usize, usize)>,
    pub(crate) docs: Vec<Doc>,
    pub(crate) active: usize,
    pub(crate) sel: (usize, usize),
    pub(crate) dropdown: Option<(&'static str, Rect)>,
    pub(crate) rec_search: String,
    pub(crate) new_db_name: String,
    pub(crate) db_path: String,
    pub(crate) msg: String,
    pub(crate) edit: Option<Edicao>,
    pub(crate) scroll_sel: bool,
    pub(crate) confirm_del: Option<usize>,
    pub(crate) row_sel: bool,
    pub(crate) pedir_nome: Option<String>,
    pub(crate) n_consulta: usize,
    pub(crate) n_tabela: usize,
}
impl AccessApp {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.text_styles.insert(egui::TextStyle::Body, FontId::proportional(13.0));
        style.text_styles.insert(egui::TextStyle::Button, FontId::proportional(13.0));
        style.visuals.widgets.hovered.weak_bg_fill = HOVER_FILL;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, HOVER_LINE);
        style.visuals.widgets.active.weak_bg_fill = PRESS_FILL;
        style.visuals.selection.bg_fill = rgb(0xCFE0F7);
        cc.egui_ctx.set_style(style);
        let recentes = crate::recentes::carregar();
        let bs_page = if recentes.is_empty() { 2 } else { 1 };
        let mut app = Self {
            db: None,
            db_file: None,
            recentes,
            new_db_dir: crate::ficheiro::pasta_inicial(),
            nav: Vec::new(),
            tab: 0,
            tab_x: (0.0, 0.0),
            backstage: true,
            bs_page,
            nav_filter: String::new(),
            nav_open: [true; 2],
            nav_sel: None,
            docs: Vec::new(),
            active: 0,
            sel: (0, 1),
            dropdown: None,
            rec_search: String::new(),
            new_db_name: "Base1.accsql".into(),
            db_path: String::new(),
            msg: String::new(),
            edit: None,
            scroll_sel: false,
            confirm_del: None,
            row_sel: false,
            pedir_nome: None,
            n_consulta: 0,
            n_tabela: 0,
        };
        app.refresh_nav();
        app
    }

    /// Volta a ler o catálogo da base de dados para o painel de navegação.
    pub(crate) fn refresh_nav(&mut self) {
        let (tabelas, consultas) = match &self.db {
            Some(db) => (db.list_tables().unwrap_or_default(), db.list_views().unwrap_or_default()),
            None => (Vec::new(), Vec::new()),
        };
        self.nav = vec![
            NavGroup { name: "Tabelas", kind: ObjKind::Table, items: tabelas },
            NavGroup { name: "Consultas", kind: ObjKind::Query, items: consultas },
        ];
        self.nav_sel = None;
    }
    pub(crate) fn act(&mut self, resp: &Response, label: &str, dd: Option<&'static str>) {
        if resp.clicked() {
            match dd {
                Some(k) => {
                    if matches!(self.dropdown, Some((kk, _)) if kk == k) {
                        self.dropdown = None;
                    } else {
                        self.dropdown = Some((k, resp.rect));
                    }
                }
                None => self.msg = format!("«{}» – apenas apresentação visual", label.replace('\n', " ")),
            }
        }
    }
    pub(crate) fn rb(&mut self, ui: &mut Ui, icon: Icon, label: &str, dd: Option<&'static str>) {
        let r = big_btn(ui, icon, label, dd.is_some());
        self.act(&r, label, dd);
    }
    pub(crate) fn stack(&mut self, ui: &mut Ui, items: &[(Icon, &str, Option<&'static str>)]) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            for (ic, l, dd) in items {
                let r = small_btn(ui, *ic, l, dd.is_some());
                self.act(&r, l, *dd);
            }
        });
    }
    pub(crate) fn open_doc(&mut self, kind: ObjKind, name: String) {
        if let Some(i) = self.docs.iter().position(|d| d.kind == kind && d.name == name) {
            self.active = i;
        } else {
            let data = match self.db.as_ref().map(|d| d.load(&name, kind == ObjKind::Table)) {
                Some(Ok(d)) => d,
                Some(Err(e)) => {
                    self.msg = format!("Erro ao abrir «{}»: {}", name, e);
                    return;
                }
                None => {
                    self.msg = "Nenhuma base de dados aberta".into();
                    return;
                }
            };
            self.docs.push(Doc { kind, name, data, sql: String::new(), info: String::new(), info_erro: false, vista: None, estr: None });
            self.active = self.docs.len() - 1;
        }
        self.sel = (0, 1);
        self.edit = None;
        self.row_sel = false;
    }
}
