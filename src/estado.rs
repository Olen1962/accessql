use eframe::egui::{self, FontId, Pos2, Rect, Stroke};
use std::path::PathBuf;
use crate::{db::{Condicao, Database, TableData}, icones::*, tema::*};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ObjKind {
    Table,
    Query,
    Sql,
    Design,
    QueryDesign,
}
impl ObjKind {
    pub(crate) fn icon(self) -> Icon {
        match self {
            ObjKind::Table => Icon::Table,
            ObjKind::Query => Icon::Query,
            ObjKind::Sql => Icon::Badge("S", C_BLUE),
            ObjKind::Design => Icon::Badge("E", C_GREEN),
            ObjKind::QueryDesign => Icon::Badge("QD", C_PURPLE),
        }
    }
}
#[derive(Clone)]
pub(crate) struct NavGroup {
    pub(crate) name: &'static str,
    pub(crate) kind: ObjKind,
    pub(crate) items: Vec<String>,
}
#[derive(Clone, Default)]
pub(crate) struct DocWindow {
    pub(crate) flutuante: bool,
    pub(crate) rect: Option<Rect>,
    pub(crate) rect_anterior: Option<Rect>,
    pub(crate) minimizada: bool,
    pub(crate) maximizada: bool,
}

#[derive(Clone)]
pub(crate) struct Doc {
    pub(crate) kind: ObjKind,
    pub(crate) name: String,
    pub(crate) data: TableData,
    pub(crate) sql: String,
    pub(crate) info: String,
    pub(crate) info_erro: bool,
    pub(crate) vista: Option<String>,
    pub(crate) sql_orig: String,
    pub(crate) ordem: Option<(String, bool)>,
    pub(crate) filtros: Vec<Condicao>,
    pub(crate) filtro_ativo: bool,
    pub(crate) estr: Option<crate::estrutura::Estrutura>,
    pub(crate) qbe: Option<crate::qbe::QueryDesign>,
    pub(crate) window: DocWindow,
}

impl Doc {
    pub(crate) fn por_guardar(&self) -> bool {
        match self.kind {
            ObjKind::Sql => self.vista.is_some() && self.sql != self.sql_orig,
            ObjKind::QueryDesign => self.qbe.as_ref().map_or(false, |q| !q.sql_error.is_empty() || q.sql_text != self.sql_orig),
            _ => false,
        }
    }
    pub(crate) fn destacar(&mut self) { self.window.flutuante = true; }
    pub(crate) fn ancorar(&mut self) { self.window.flutuante = false; self.window.minimizada = false; }
    pub(crate) fn flutuante(&self) -> bool { self.window.flutuante }
    pub(crate) fn minimizado(&self) -> bool { self.window.minimizada }
}

pub(crate) fn a_escrever(ctx: &egui::Context) -> bool {
    ctx.memory(|m| {
        matches!(m.focused(), Some(id) if ["nav_filter", "rec_search", "sql_editor", "nome_consulta", "nome_objeto", "nome_duplicar", "filtro_texto", "csv_nome_novo", "nome_indice"]
            .iter()
            .any(|n| id == egui::Id::new(*n)))
    })
}

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
    pub(crate) foco_nav: bool,
    pub(crate) nav_rect: Option<Rect>,
    pub(crate) renomear_obj: Option<(ObjKind, String, String)>,
    pub(crate) duplicar_obj: Option<(ObjKind, String, String)>,
    pub(crate) menu_col: Option<(usize, Pos2)>,
    pub(crate) menu_texto: String,
    pub(crate) focar_busca: bool,
    pub(crate) importar: Option<crate::csvio::ImportCsv>,
    pub(crate) exportar_csv: Option<crate::csvio::ExportCsv>,
    pub(crate) undo: Vec<Vec<crate::desfazer::Acao>>,
    pub(crate) redo: Vec<Vec<crate::desfazer::Acao>>,
    pub(crate) hist_pend: Vec<crate::desfazer::Acao>,
    pub(crate) clip_acao: Option<u8>,
    pub(crate) totais_aberto: bool,
    pub(crate) qbe_add_table_aberto: bool,
    pub(crate) indices: Option<crate::indices::EstIndices>,
    pub(crate) relacoes: Option<crate::relacoes::EstRelacoes>,
    pub(crate) eliminar_obj: Option<crate::eliminar_obj::EliminarObj>,
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
            db: None, db_file: None, recentes, new_db_dir: crate::ficheiro::pasta_inicial(),
            nav: Vec::new(), tab: 0, tab_x: (0.0, 0.0), backstage: true, bs_page,
            nav_filter: String::new(), nav_open: [true; 2], nav_sel: None,
            docs: Vec::new(), active: 0, sel: (0, 1), rec_search: String::new(),
            new_db_name: "Base1.accsql".into(), db_path: String::new(), msg: String::new(),
            edit: None, scroll_sel: false, confirm_del: None, row_sel: false,
            pedir_nome: None, n_consulta: 0, n_tabela: 0, foco_nav: false,
            nav_rect: None, renomear_obj: None, duplicar_obj: None, menu_col: None,
            menu_texto: String::new(), focar_busca: false, importar: None, exportar_csv: None,
            undo: Vec::new(), redo: Vec::new(), hist_pend: Vec::new(), clip_acao: None,
            totais_aberto: false, qbe_add_table_aberto: false, indices: None, relacoes: None, eliminar_obj: None,
        };
        app.refresh_nav();
        app
    }
    
    pub(crate) fn abrir_doc_flutuante(&mut self) {
        if let Some(doc) = self.docs.get(self.active).cloned() {
            let mut d = doc; d.window.flutuante = true; self.docs[self.active] = d;
        }
    }
    
    pub(crate) fn reintegrar_doc(&mut self) {
        if let Some(doc) = self.docs.get(self.active).cloned() {
            let mut d = doc; d.window.flutuante = false; d.window.minimizada = false; self.docs[self.active] = d;
        }
    }
    
    pub(crate) fn doc_flutuante(&self) -> bool { self.docs.get(self.active).map_or(false, |d| d.flutuante()) }
    pub(crate) fn doc_minimizado(&self) -> bool { self.docs.get(self.active).map_or(false, |d| d.minimizado()) }
    pub(crate) fn doc_flutuante_ativa(&self) -> bool { self.docs.get(self.active).map_or(false, |d| d.flutuante() && !d.minimizado()) }

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

    pub(crate) fn procurar_objeto(&self, nome: &str) -> Option<usize> {
        self.docs.iter().position(|d| {
            d.name == nome
                && match d.kind {
                    ObjKind::Table | ObjKind::Query => true,
                    ObjKind::Design => d.estr.as_ref().map_or(false, |e| !e.nova),
                    ObjKind::Sql => d.vista.as_deref() == Some(nome),
                    ObjKind::QueryDesign => d.qbe.is_some(),
                }
        })
    }

    pub(crate) fn open_doc(&mut self, kind: ObjKind, name: String) {
        let existente = self.procurar_objeto(&name);
        if let Some(i) = existente {
            if self.docs[i].kind == kind { self.active = i; self.sel = (0, 1); self.edit = None; self.row_sel = false; return; }
            if self.docs[i].por_guardar() { self.active = i; self.msg = format!("«{}» tem alterações por guardar: guarde a consulta antes de mudar de vista", name); return; }
        }
        let data = match self.db.as_ref().map(|d| d.load(&name, kind == ObjKind::Table)) {
            Some(Ok(d)) => d, Some(Err(e)) => { self.msg = format!("Erro ao abrir «{}»: {}", name, e); return; }
            None => { self.msg = "Nenhuma base de dados aberta".into(); return; }
        };
        let doc = Doc { kind, name, data, sql: String::new(), info: String::new(), info_erro: false, vista: None, sql_orig: String::new(), ordem: None, filtros: Vec::new(), filtro_ativo: true, estr: None, qbe: None, window: DocWindow::default() };
        match existente { Some(i) => { self.docs[i] = doc; self.active = i; } None => { self.docs.push(doc); self.active = self.docs.len() - 1; } }
        self.sel = (0, 1); self.edit = None; self.row_sel = false;
    }

    pub(crate) fn novo_doc_qbe(&mut self, qbe: crate::qbe::QueryDesign, nome: String, vista: Option<String>) {
        let doc = Doc { kind: ObjKind::QueryDesign, name: nome, data: Default::default(), sql: String::new(), info: String::new(), info_erro: false, vista, sql_orig: String::new(), ordem: None, filtros: Vec::new(), filtro_ativo: true, estr: None, qbe: Some(qbe), window: DocWindow::default() };
        self.docs.push(doc); self.active = self.docs.len() - 1; self.sel = (0, 1); self.edit = None; self.row_sel = false;
    }
}