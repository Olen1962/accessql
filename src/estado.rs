use eframe::egui::{self, Color32, FontId, Rect, Response, Stroke, Ui, Vec2};
use crate::{icones::*, tema::*, widgets::*};

// ───────────────────────────── Estado da aplicação ─────────────────────────────

#[derive(Clone, Copy, PartialEq)]

pub(crate) enum ObjKind {

    Table,

    Query,

    Form,

    Report,

}

impl ObjKind {

    pub(crate) fn icon(self) -> Icon {

        match self {

            ObjKind::Table => Icon::Table,

            ObjKind::Query => Icon::Query,

            ObjKind::Form => Icon::Form,

            ObjKind::Report => Icon::Report,

        }

    }

}



pub(crate) const OBJECTS: [(&str, ObjKind, &[&str]); 4] = [

    ("Tabelas", ObjKind::Table, &["Clientes", "Encomendas", "Fornecedores", "Produtos"]),

    ("Consultas", ObjKind::Query, &["Encomendas por Cliente", "Produtos em Falta"]),

    ("Formulários", ObjKind::Form, &["Clientes", "Encomendas"]),

    ("Relatórios", ObjKind::Report, &["Lista de Clientes", "Faturas"]),

];



pub(crate) struct Doc {

    pub(crate) kind: ObjKind,

    pub(crate) name: String,

}



pub(crate) struct AccessApp {

    pub(crate) tab: usize,

    pub(crate) tab_x: (f32, f32),

    pub(crate) backstage: bool,

    pub(crate) bs_page: usize,

    pub(crate) nav_filter: String,

    pub(crate) nav_open: [bool; 4],

    pub(crate) nav_sel: Option<(usize, usize)>,

    pub(crate) docs: Vec<Doc>,

    pub(crate) active: usize,

    pub(crate) sel: (usize, usize),

    pub(crate) dropdown: Option<(&'static str, Rect)>,

    pub(crate) rec_search: String,

    pub(crate) new_db_name: String,

    pub(crate) db_path: String,

    pub(crate) msg: String,

}



impl AccessApp {

    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {

        let mut style = (*cc.egui_ctx.style()).clone();

        style.visuals = egui::Visuals::light();

        style.text_styles.insert(egui::TextStyle::Body, FontId::proportional(13.0));

        style.text_styles.insert(egui::TextStyle::Button, FontId::proportional(13.0));

        style.visuals.widgets.hovered.weak_bg_fill = HOVER_FILL;

        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, HOVER_LINE);

        style.visuals.widgets.active.weak_bg_fill = PRESS_FILL;

        style.visuals.selection.bg_fill = rgb(0xCFE0F7);

        cc.egui_ctx.set_style(style);

        Self {

            tab: 0,

            tab_x: (0.0, 0.0),

            backstage: false,

            bs_page: 0,

            nav_filter: String::new(),

            nav_open: [true; 4],

            nav_sel: Some((0, 0)),

            docs: vec![Doc { kind: ObjKind::Table, name: "Clientes".into() }],

            active: 0,

            sel: (0, 1),

            dropdown: None,

            rec_search: String::new(),

            new_db_name: "Base de Dados1.accdb".into(),

            db_path: "Base de Dados1".into(),

            msg: String::new(),

        }

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

            self.docs.push(Doc { kind, name });

            self.active = self.docs.len() - 1;

        }

        self.sel = (0, 1);

    }

}
