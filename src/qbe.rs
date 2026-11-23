//! Construtor Visual de Consultas (Query By Example - QBE).
use crate::{db::Database, estado::{AccessApp, Doc, ObjKind, DocWindow}, tema::*};
use eframe::egui::{self, Align2, Color32, FontId, Id, Pos2, Rect, Sense, Stroke, Ui, Vec2};

// ────────────────── MODELO DE DADOS ──────────────────

#[derive(Clone, Debug)]
pub(crate) struct QbeTable {
    pub name: String, pub alias: String, pub rect: Rect, pub fields: Vec<QbeField>, pub expanded: bool,
}
#[derive(Clone, Debug)]
pub(crate) struct QbeField { pub name: String, pub tipo: String, pub table_alias: String, pub grid_row: Option<usize> }
#[derive(Clone, Debug, Default)]
pub(crate) struct QbeJoin { pub left_table: String, pub left_field: String, pub right_table: String, pub right_field: String, pub join_type: JoinType }
#[derive(Clone, Copy, Debug, PartialEq, Default)] pub(crate) enum JoinType { #[default] Inner, Left, Right }

#[derive(Clone, Debug, Default)]
pub(crate) struct GridRow {
    pub field: String, pub table: String, pub alias: String, pub sort: SortOrder,
    pub show: bool, pub criteria: String, pub or: Vec<String>, pub group_by: GroupBy,
}
#[derive(Clone, Copy, Debug, PartialEq, Default)] pub(crate) enum SortOrder { #[default] None, Asc, Desc }
#[derive(Clone, Copy, Debug, PartialEq, Default)] pub(crate) enum GroupBy { #[default] None, GroupBy, Sum, Avg, Min, Max, Count, First, Last, Where }

// Pública para sql_parser.rs
#[derive(Default)]
pub(crate) struct ParsedQbe { pub tables: Vec<QbeTable>, pub joins: Vec<QbeJoin>, pub grid: Vec<GridRow>, pub next_alias_id: usize }

#[derive(Default, Clone)]
pub(crate) struct QueryDesign {
    pub tables: Vec<QbeTable>, pub joins: Vec<QbeJoin>, pub grid: Vec<GridRow>,
    pub sql_text: String, pub sql_error: String, pub drag: Option<DragState>,
    pub next_alias_id: usize, pub show_sql_preview: bool,
}

#[derive(Clone, Debug)]
pub(crate) enum DragState {
    MoveTable(usize, Vec2),
    ConnectField { from_table: usize, from_field: String, start_pos: Pos2, current_pos: Pos2 },
}
impl DragState { fn as_connect(&self) -> Option<(Pos2, Pos2)> { match self { DragState::ConnectField { start_pos, current_pos, .. } => Some((*start_pos, *current_pos)), _ => None } } }

// ────────────────── IMPLEMENTAÇÃO ACCESSAPP ──────────────────

impl AccessApp {
    pub(crate) fn abrir_qbe(&mut self, nome_existente: Option<String>) {
        if self.db.is_none() { self.msg = "Abra ou crie uma base de dados primeiro".into(); return; }
        let mut qbe = QueryDesign::default(); qbe.next_alias_id = 1;
        if let Some(ref nome) = nome_existente {
            if let Some(db) = &self.db { if let Ok(sql) = db.view_sql(nome) { qbe.sql_text = sql.clone(); match crate::sql_parser::parse_sql_to_qbe(&sql, db) { Ok(parsed) => { qbe.tables = parsed.tables; qbe.joins = parsed.joins; qbe.grid = parsed.grid; qbe.next_alias_id = parsed.next_alias_id; } Err(e) => { qbe.sql_error = e; } } } }
        } else { qbe.sql_error = "Adicione tabelas com o botão «Adicionar Tabela» ou menu de contexto no painel superior.".into(); }
        self.n_consulta += 1;
        let nome = format!("Consulta{}", self.n_consulta);
        self.novo_doc_qbe(qbe, nome, nome_existente);
    }

    pub(crate) fn qbe_gerar_sql(&mut self, apenas_preview: bool) {
        let idx = self.active;
        let sql_res = match self.docs.get(idx).and_then(|d| d.qbe.as_ref()) {
            Some(qbe) => build_sql_from_qbe(qbe),
            None => return,
        };
        match sql_res {
            Ok(sql) => {
                if let Some(doc) = self.docs.get_mut(idx) { if let Some(qbe) = doc.qbe.as_mut() { qbe.sql_text = sql.clone(); qbe.sql_error = String::new(); } }
                if !apenas_preview {
                    let kind_antigo = { let doc = &mut self.docs[idx]; doc.sql = sql; let k = doc.kind; doc.kind = ObjKind::Sql; k };
                    self.executar_sql();
                    self.docs[idx].kind = kind_antigo;
                }
            }
            Err(e) => { if let Some(doc) = self.docs.get_mut(idx) { if let Some(qbe) = doc.qbe.as_mut() { qbe.sql_error = e; } } }
        }
    }

    pub(crate) fn qbe_guardar_como_view(&mut self) {
        let Some(doc) = self.docs.get(self.active) else { return };
        let Some(qbe) = &doc.qbe else { return };
        if qbe.sql_text.is_empty() || !qbe.sql_error.is_empty() { return; }
        let nome = doc.vista.clone().unwrap_or_else(|| doc.name.clone());
        self.pedir_nome = Some(nome);
    }

    pub(crate) fn qbe_dialogo_adicionar_tabela_inline(&mut self, ctx: &egui::Context) {
        let todos = {
            let Some(db) = self.db.as_ref() else { self.qbe_add_table_aberto = false; return; };
            let mut t = db.list_tables().unwrap_or_default();
            t.append(&mut db.list_views().unwrap_or_default());
            t.sort();
            t
        };
        let mut aberto = self.qbe_add_table_aberto;
        let mut escolhida: Option<String> = None;
        egui::Window::new("Adicionar Tabela").open(&mut aberto).collapsible(false).resizable(true).default_width(300.0).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            ui.label("Faça duplo clique para adicionar:");
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for t in &todos { if ui.selectable_label(false, t).double_clicked() { escolhida = Some(t.clone()); } }
            });
        });
        self.qbe_add_table_aberto = aberto;
        if let Some(nome) = escolhida {
            if let Some(db) = self.db.as_ref() {
                if let Some(doc) = self.docs.get_mut(self.active) {
                    if let Some(qbe) = doc.qbe.as_mut() {
                        AccessApp::qbe_adicionar_tabela(qbe, db, &nome);
                    }
                }
            }
        }
    }

    fn qbe_adicionar_tabela(qbe: &mut QueryDesign, db: &Database, table_name: &str) {
        if let Ok((campos, _)) = db.info_tabela(table_name) {
            let alias = format!("t{}", qbe.next_alias_id); qbe.next_alias_id += 1;
            let fields: Vec<QbeField> = campos.into_iter().map(|c| QbeField { name: c.nome, tipo: c.tipo, table_alias: alias.clone(), grid_row: None }).collect();
            let x = 50.0 + qbe.tables.len() as f32 * 220.0; let y = 80.0; let h = 40.0 + fields.len() as f32 * 22.0;
            let rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(200.0, h));
            qbe.tables.push(QbeTable { name: table_name.to_string(), alias, rect, fields, expanded: true });
        }
    }

    fn qbe_adicionar_campo_na_grelha(&mut self, qbe: &mut QueryDesign, table_idx: usize, field_name: &str, table_alias: &str) {
        let row_idx = qbe.grid.len();
        qbe.grid.push(GridRow { field: field_name.to_string(), table: table_alias.to_string(), alias: String::new(), sort: SortOrder::None, show: true, criteria: String::new(), or: Vec::new(), group_by: GroupBy::None });
        if let Some(t) = qbe.tables.get_mut(table_idx) { if let Some(f) = t.fields.iter_mut().find(|f| f.name == field_name) { f.grid_row = Some(row_idx); } }
    }

    fn qbe_sync_grid_row(qbe: &mut QueryDesign, grid_idx: usize, field_full: &str) {
        for t in &mut qbe.tables { for f in &mut t.fields { if f.grid_row == Some(grid_idx) { f.grid_row = None; } } }
        if let Some((t_alias, f_name)) = field_full.split_once('.') { if let Some(t) = qbe.tables.iter_mut().find(|t| t.alias == t_alias) { if let Some(f) = t.fields.iter_mut().find(|f| f.name == f_name) { f.grid_row = Some(grid_idx); } } }
    }

    pub(crate) fn qbe_view(&mut self, ui: &mut Ui, qbe: &mut QueryDesign) {
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("▶ Executar").min_size(Vec2::new(90.0, 24.0))).clicked() { self.qbe_gerar_sql(false); }
            if ui.add(egui::Button::new("👁 Pré-visualizar SQL").min_size(Vec2::new(130.0, 24.0))).clicked() { self.qbe_gerar_sql(true); qbe.show_sql_preview = true; }
            if ui.add(egui::Button::new("💾 Guardar como Consulta").min_size(Vec2::new(150.0, 24.0))).clicked() { self.qbe_guardar_como_view(); }
            ui.separator();
            if ui.add(egui::Button::new("➕ Adicionar Tabela").min_size(Vec2::new(130.0, 24.0))).clicked() { self.qbe_add_table_aberto = true; }
            if ui.add(egui::Button::new("❌ Limpar Tudo").min_size(Vec2::new(100.0, 24.0))).clicked() { qbe.tables.clear(); qbe.joins.clear(); qbe.grid.clear(); qbe.sql_error.clear(); }
        });

        if qbe.show_sql_preview && !qbe.sql_text.is_empty() {
            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| { ui.add(egui::TextEdit::multiline(&mut qbe.sql_text).font(egui::FontId::monospace(12.0)).desired_width(f32::INFINITY).interactive(false)); });
            ui.separator();
        }
        if !qbe.sql_error.is_empty() { ui.colored_label(Color32::from_rgb(0xB0,0,0x20), &qbe.sql_error); ui.separator(); }

        let available = ui.available_rect_before_wrap();
        let top_height = (available.height() * 0.45).clamp(150.0, 300.0);
        
        let (top_rect, _top_resp) = ui.allocate_exact_size(Vec2::new(available.width(), top_height), Sense::hover());
        let painter = ui.painter().clone();
        painter.rect_filled(top_rect, 0.0, Color32::from_rgb(0xF8,0xF8,0xF8));
        painter.rect_stroke(top_rect, 0.0, Stroke::new(1.0, GRID));

        for join in &qbe.joins {
            if let (Some(left_t), Some(right_t)) = (qbe.tables.iter().find(|t| t.alias == join.left_table), qbe.tables.iter().find(|t| t.alias == join.right_table)) {
                let p1 = Pos2::new(left_t.rect.right(), left_t.rect.center().y);
                let p2 = Pos2::new(right_t.rect.left(), right_t.rect.center().y);
                let color = if join.join_type == JoinType::Inner { C_BLUE } else { C_ORANGE };
                painter.line_segment([p1, p2], Stroke::new(2.0, color));
                let mid = (p1 + p2.to_vec2()) * 0.5; let dir = (p2 - p1).normalized(); let perp = Vec2::new(-dir.y, dir.x) * 5.0;
                painter.add(eframe::egui::Shape::convex_polygon(vec![mid + dir * 10.0, mid - dir * 5.0 + perp, mid - dir * 5.0 - perp], color, Stroke::NONE));
            }
        }

        let mut table_to_remove: Option<usize> = None;
        for ti in 0..qbe.tables.len() {
            let rect = qbe.tables[ti].rect;
            let resp = ui.interact(rect, Id::new(("qbe_table", ti)), Sense::click_and_drag());
            if resp.dragged() { let nr = rect.translate(resp.drag_delta()); let dy = if nr.min.y < top_rect.min.y { top_rect.min.y - nr.min.y } else { 0.0 }; qbe.tables[ti].rect = nr.translate(Vec2::new(0.0, dy)); }
            
            let header = Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.min.y + 24.0));
            painter.rect_filled(header, 0.0, RIBBON_BG);
            painter.text(header.center(), Align2::CENTER_CENTER, qbe.tables[ti].name.clone(), FontId::proportional(12.0), TEXT);
            
            let close_r = Rect::from_min_size(Pos2::new(rect.max.x - 20.0, rect.min.y + 2.0), Vec2::splat(18.0));
            if ui.put(close_r, egui::Button::new("✕")).clicked() { table_to_remove = Some(ti); }
            
            let alias = qbe.tables[ti].alias.clone();
            let mut y = header.max.y + 4.0;
            for fi in 0..qbe.tables[ti].fields.len() {
                let field_name = qbe.tables[ti].fields[fi].name.clone();
                let field_tipo = qbe.tables[ti].fields[fi].tipo.clone();
                let in_grid = qbe.tables[ti].fields[fi].grid_row.is_some();
                let field_rect = Rect::from_min_size(Pos2::new(rect.min.x + 4.0, y), Vec2::new(rect.width() - 8.0, 20.0));
                let bg = if in_grid { Color32::from_rgb(0xCF,0xE0,0xF7) } else { Color32::TRANSPARENT };
                painter.rect_filled(field_rect, 0.0, bg);
                
                let pk = field_name.eq_ignore_ascii_case("id") || field_name.to_lowercase().ends_with("_id");
                if pk { painter.text(Pos2::new(field_rect.min.x + 4.0, field_rect.center().y), Align2::LEFT_CENTER, "🔑", FontId::proportional(10.0), C_RED); }
                
                let txt = format!("{} ({})", field_name, field_tipo);
                painter.text(Pos2::new(field_rect.min.x + (if pk {20.0} else {4.0}), field_rect.center().y), Align2::LEFT_CENTER, txt, FontId::proportional(11.0), TEXT);
                
                let field_resp = ui.interact(field_rect, Id::new(("qbe_field", ti, &field_name)), Sense::click_and_drag());
                if field_resp.double_clicked() { self.qbe_adicionar_campo_na_grelha(qbe, ti, &field_name, &alias); }
                if field_resp.drag_started() { qbe.drag = Some(DragState::ConnectField { from_table: ti, from_field: field_name.clone(), start_pos: field_resp.interact_pointer_pos().unwrap_or(field_rect.center()), current_pos: field_resp.interact_pointer_pos().unwrap_or(field_rect.center()) }); }
                if field_resp.dragged() { if let Some(DragState::ConnectField { current_pos, .. }) = &mut qbe.drag { *current_pos = field_resp.interact_pointer_pos().unwrap_or(*current_pos); } }
                if field_resp.drag_stopped() {
                    if let Some(DragState::ConnectField { from_table, from_field, current_pos, .. }) = qbe.drag.take() {
                        let from_alias = qbe.tables[from_table].alias.clone();
                        for ti2 in 0..qbe.tables.len() {
                            if ti2 == from_table { continue; }
                            let t2_alias = qbe.tables[ti2].alias.clone();
                            let t2_rect = qbe.tables[ti2].rect;
                            for fi2 in 0..qbe.tables[ti2].fields.len() {
                                let f2_name = qbe.tables[ti2].fields[fi2].name.clone();
                                let f2_rect = Rect::from_min_size(Pos2::new(t2_rect.min.x + 4.0, t2_rect.min.y + 28.0 + fi2 as f32 * 22.0), Vec2::new(t2_rect.width() - 8.0, 20.0));
                                if f2_rect.contains(current_pos) {
                                    qbe.joins.push(QbeJoin { left_table: from_alias.clone(), left_field: from_field.clone(), right_table: t2_alias.clone(), right_field: f2_name.clone(), join_type: JoinType::Inner }); break;
                                }
                            }
                        }
                    }
                }
                y += 22.0;
            }
        }
        if let Some(i) = table_to_remove { let alias = qbe.tables[i].alias.clone(); qbe.tables.remove(i); qbe.joins.retain(|j| j.left_table != alias && j.right_table != alias); qbe.grid.retain(|r| r.table != alias); }

        if let Some(DragState::ConnectField { start_pos, current_pos, .. }) = &qbe.drag { painter.line_segment([*start_pos, *current_pos], Stroke::new(2.0, C_GREEN)); }

        ui.add_space(4.0);
        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("qbe_grid").striped(true).spacing([4.0, 2.0]).show(ui, |ui| {
                for h in ["Campo", "Tabela", "Alias", "Ordenar", "Mostrar", "Agrupar", "Critério", "Ou..."] { ui.label(egui::RichText::new(h).strong().size(11.0)); }
                ui.end_row();
                let mut to_remove: Option<usize> = None;
                let field_options: Vec<String> = qbe.tables.iter().flat_map(|t| t.fields.iter().map(|f| format!("{}.{}", t.alias, f.name))).collect();
                for ri in 0..qbe.grid.len() {
                    egui::ComboBox::from_id_salt(("qbe_field_sel", ri)).selected_text(if qbe.grid[ri].field.contains('.') { qbe.grid[ri].field.clone() } else { format!("{}.{}", qbe.grid[ri].table, qbe.grid[ri].field) }).show_ui(ui, |ui| {
                        for opt in &field_options { if ui.selectable_label(qbe.grid[ri].field == opt.split('.').nth(1).unwrap_or(""), opt).clicked() { if let Some((t, f)) = opt.split_once('.') { qbe.grid[ri].table = t.to_string(); qbe.grid[ri].field = f.to_string(); Self::qbe_sync_grid_row(qbe, ri, opt); } } }
                    });
                    ui.label(qbe.grid[ri].table.clone()); ui.text_edit_singleline(&mut qbe.grid[ri].alias);
                    egui::ComboBox::from_id_salt(("qbe_sort", ri)).selected_text(match qbe.grid[ri].sort { SortOrder::Asc => "Ascendente", SortOrder::Desc => "Descendente", _ => "(não)" }).show_ui(ui, |ui| { ui.selectable_value(&mut qbe.grid[ri].sort, SortOrder::None, "(não)"); ui.selectable_value(&mut qbe.grid[ri].sort, SortOrder::Asc, "Ascendente"); ui.selectable_value(&mut qbe.grid[ri].sort, SortOrder::Desc, "Descendente"); });
                    ui.checkbox(&mut qbe.grid[ri].show, "");
                    egui::ComboBox::from_id_salt(("qbe_group", ri)).selected_text(match qbe.grid[ri].group_by { GroupBy::GroupBy => "Agrupar", GroupBy::Sum => "Soma", GroupBy::Avg => "Média", GroupBy::Min => "Mín", GroupBy::Max => "Máx", GroupBy::Count => "Contar", GroupBy::First => "Primeiro", GroupBy::Last => "Último", GroupBy::Where => "Onde", _ => "(não)" }).show_ui(ui, |ui| { for (v, l) in [(GroupBy::GroupBy, "Agrupar"), (GroupBy::Sum, "Soma"), (GroupBy::Avg, "Média"), (GroupBy::Min, "Mín"), (GroupBy::Max, "Máx"), (GroupBy::Count, "Contar"), (GroupBy::Where, "Onde")] { ui.selectable_value(&mut qbe.grid[ri].group_by, v, l); } ui.selectable_value(&mut qbe.grid[ri].group_by, GroupBy::None, "(não)"); });
                    ui.text_edit_singleline(&mut qbe.grid[ri].criteria).on_hover_text("Ex: >10, \"Lisboa\", Between 1 And 10, Is Null, Like \"A%\"");
                    if ui.small_button("...").clicked() { /* TODO */ }
                    if ui.small_button("✕").clicked() { to_remove = Some(ri); }
                    ui.end_row();
                }
                if let Some(i) = to_remove { qbe.grid.remove(i); }
            });
        });
    }
}

// ────────────────── GERAÇÃO DE SQL ──────────────────

fn build_sql_from_qbe(qbe: &QueryDesign) -> Result<String, String> {
    if qbe.tables.is_empty() { return Err("Nenhuma tabela adicionada".into()); }
    if qbe.grid.iter().filter(|r| r.show).count() == 0 { return Err("Nenhum campo marcado «Mostrar»".into()); }
    let mut select = Vec::new(); let mut from_table = None; let mut joins_sql = Vec::new(); let mut where_clauses = Vec::new(); let mut group_by_fields = Vec::new(); let mut order_by_fields = Vec::new(); let mut has_aggregate = false;
    for row in &qbe.grid { if !row.show { continue; } let field_ref = if row.table.is_empty() { row.field.clone() } else { format!("\"{}\".\"{}\"", row.table, row.field) }; let expr = match row.group_by { GroupBy::None => field_ref, GroupBy::Sum => { has_aggregate = true; format!("SUM({})", field_ref) } GroupBy::Avg => { has_aggregate = true; format!("AVG({})", field_ref) } GroupBy::Min => { has_aggregate = true; format!("MIN({})", field_ref) } GroupBy::Max => { has_aggregate = true; format!("MAX({})", field_ref) } GroupBy::Count => { has_aggregate = true; format!("COUNT({})", field_ref) } GroupBy::First => { has_aggregate = true; format!("MIN({})", field_ref) } GroupBy::Last => { has_aggregate = true; format!("MAX({})", field_ref) } GroupBy::GroupBy => { group_by_fields.push(field_ref.clone()); field_ref } GroupBy::Where => continue }; let alias = if row.alias.is_empty() { String::new() } else { format!(" AS \"{}\"", row.alias) }; select.push(format!("{}{}", expr, alias)); }
    let mut is_right = std::collections::HashSet::new(); for j in &qbe.joins { is_right.insert(j.right_table.clone()); } let root = qbe.tables.iter().find(|t| !is_right.contains(&t.alias)).map(|t| t.alias.clone()); if let Some(r) = root { let t = qbe.tables.iter().find(|x| x.alias == r).unwrap(); from_table = Some(format!("\"{}\" \"{}\"", t.name, t.alias)); } else if let Some(t) = qbe.tables.first() { from_table = Some(format!("\"{}\" \"{}\"", t.name, t.alias)); }
    for j in &qbe.joins { let jtype = match j.join_type { JoinType::Inner => "INNER JOIN", JoinType::Left => "LEFT JOIN", JoinType::Right => "RIGHT JOIN" }; let left = format!("\"{}\".\"{}\"", j.left_table, j.left_field); let right = format!("\"{}\".\"{}\"", j.right_table, j.right_field); let right_table_name = qbe.tables.iter().find(|t| t.alias == j.right_table).map(|t| t.name.as_str()).unwrap_or(""); joins_sql.push(format!("{} \"{}\" \"{}\" ON {} = {}", jtype, right_table_name, j.right_table, left, right)); }
    for row in &qbe.grid { if row.group_by == GroupBy::Where || row.group_by == GroupBy::None { let field_ref = if row.table.is_empty() { row.field.clone() } else { format!("\"{}\".\"{}\"", row.table, row.field) }; let crit = parse_criteria(&row.criteria, &field_ref); if !crit.is_empty() { where_clauses.push(crit); } for or_val in &row.or { let or_crit = parse_criteria(or_val, &field_ref); if !or_crit.is_empty() { where_clauses.push(format!("OR {}", or_crit)); } } } }
    if has_aggregate { for row in &qbe.grid { if row.group_by == GroupBy::GroupBy { let field_ref = if row.table.is_empty() { row.field.clone() } else { format!("\"{}\".\"{}\"", row.table, row.field) }; group_by_fields.push(field_ref); } } }
    for row in &qbe.grid { if row.sort != SortOrder::None { let field_ref = if row.table.is_empty() { row.field.clone() } else { format!("\"{}\".\"{}\"", row.table, row.field) }; order_by_fields.push(format!("{} {}", field_ref, if row.sort == SortOrder::Asc { "ASC" } else { "DESC" })); } }
    let mut sql = String::new(); sql.push_str("SELECT "); sql.push_str(&select.join(", ")); sql.push_str(" FROM "); sql.push_str(&from_table.unwrap()); for j in joins_sql { sql.push_str(" "); sql.push_str(&j); } if !where_clauses.is_empty() { sql.push_str(" WHERE "); sql.push_str(&where_clauses.join(" AND ")); } if !group_by_fields.is_empty() { sql.push_str(" GROUP BY "); sql.push_str(&group_by_fields.join(", ")); } if !order_by_fields.is_empty() { sql.push_str(" ORDER BY "); sql.push_str(&order_by_fields.join(", ")); } sql.push(';'); Ok(sql)
}

fn parse_criteria(txt: &str, field: &str) -> String {
    let t = txt.trim(); if t.is_empty() { return String::new(); }
    if t.eq_ignore_ascii_case("Is Null") { return format!("{} IS NULL", field); }
    if t.eq_ignore_ascii_case("Is Not Null") { return format!("{} IS NOT NULL", field); }
    if let Some(rest) = t.strip_prefix("Between ") { if let Some((a, b)) = rest.split_once(" And ") { return format!("{} BETWEEN {} AND {}", field, a.trim(), b.trim()); } }
    if t.starts_with("In(") && t.ends_with(')') { return format!("{} IN ({})", field, &t[3..t.len()-1]); }
    for op in [">=", "<=", "<>", ">", "<", "=", "Like "] { if let Some(val) = t.strip_prefix(op) { let val = val.trim(); let sql_val = if val.starts_with('"') || val.starts_with('\'') || val.parse::<f64>().is_ok() { val.to_string() } else { format!("'{}'", val) }; return format!("{} {} {}", field, if op=="Like " { "LIKE" } else { op }, sql_val); } }
    let sql_val = if t.parse::<f64>().is_ok() { t.to_string() } else { format!("'{}'", t) }; format!("{} = {}", field, sql_val)
}