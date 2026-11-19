//! Dados Externos: importar e exportar ficheiros CSV.

use crate::{
    db::Campo,
    estado::*,
};
use eframe::egui::{self, Align2, RichText, Vec2};
use std::path::Path;

/// Estado do diálogo «Importar Ficheiro CSV».
pub(crate) struct ImportCsv {
    pub(crate) nome_ficheiro: String,
    pub(crate) bruto: String,
    pub(crate) codificacao: &'static str,
    pub(crate) sep: char,
    pub(crate) sep_analisado: char,
    pub(crate) cabecalho: bool,
    pub(crate) virgula: bool,
    /// `true` = criar tabela nova; `false` = acrescentar a uma existente.
    pub(crate) novo: bool,
    pub(crate) nome_novo: String,
    pub(crate) destino: usize,
    pub(crate) linhas: Vec<Vec<String>>,
    pub(crate) erro: String,
}

/// Estado do diálogo «Exportar CSV».
pub(crate) struct ExportCsv {
    pub(crate) sep: char,
    pub(crate) cabecalho: bool,
    pub(crate) bom: bool,
}

fn nome_sep(c: char) -> &'static str {
    match c {
        ';' => "; (ponto e vírgula)",
        ',' => ", (vírgula)",
        '\t' => "Tabulador",
        '|' => "| (barra vertical)",
        _ => "?",
    }
}

fn abreviar(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let t: String = s.chars().take(n).collect();
        format!("{}…", t)
    }
}

/// Divide texto CSV em linhas e campos (aspas, aspas duplicadas e quebras de linha dentro de campos).
pub(crate) fn analisar(texto: &str, sep: char) -> Vec<Vec<String>> {
    let mut linhas: Vec<Vec<String>> = Vec::new();
    let mut linha: Vec<String> = Vec::new();
    let mut campo = String::new();
    let mut aspas = false;
    let mut tem = false;
    let mut it = texto.chars().peekable();
    while let Some(c) = it.next() {
        if aspas {
            if c == '"' {
                if it.peek() == Some(&'"') {
                    campo.push('"');
                    it.next();
                } else {
                    aspas = false;
                }
            } else {
                campo.push(c);
            }
        } else if c == '"' && campo.is_empty() {
            aspas = true;
            tem = true;
        } else if c == sep {
            linha.push(std::mem::take(&mut campo));
            tem = true;
        } else if c == '\n' || c == '\r' {
            if c == '\r' && it.peek() == Some(&'\n') {
                it.next();
            }
            if tem || !campo.is_empty() || !linha.is_empty() {
                linha.push(std::mem::take(&mut campo));
                linhas.push(std::mem::take(&mut linha));
            }
            tem = false;
        } else {
            campo.push(c);
            tem = true;
        }
    }
    if tem || !campo.is_empty() || !linha.is_empty() {
        linha.push(campo);
        linhas.push(linha);
    }
    linhas
}

/// Escolhe o separador mais frequente na primeira linha.
fn detectar_separador(texto: &str) -> char {
    let primeira = texto.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let mut melhor = ';';
    let mut max = 0;
    for s in [';', ',', '\t', '|'] {
        let n = primeira.matches(s).count();
        if n > max {
            max = n;
            melhor = s;
        }
    }
    melhor
}

fn cp1252(b: u8) -> char {
    match b {
        0x80 => '€',
        0x82 => '‚',
        0x83 => 'ƒ',
        0x84 => '„',
        0x85 => '…',
        0x86 => '†',
        0x87 => '‡',
        0x88 => 'ˆ',
        0x89 => '‰',
        0x8A => 'Š',
        0x8B => '‹',
        0x8C => 'Œ',
        0x8E => 'Ž',
        0x91 => '‘',
        0x92 => '’',
        0x93 => '“',
        0x94 => '”',
        0x95 => '•',
        0x96 => '–',
        0x97 => '—',
        0x98 => '˜',
        0x99 => '™',
        0x9A => 'š',
        0x9B => '›',
        0x9C => 'œ',
        0x9E => 'ž',
        0x9F => 'Ÿ',
        _ => char::from(b),
    }
}

/// UTF-8 (com ou sem BOM) ou, se não for válido, Windows-1252 (o que o Excel costuma gravar).
fn decodificar(bytes: Vec<u8>) -> (String, &'static str) {
    let bytes = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) { bytes[3..].to_vec() } else { bytes };
    match String::from_utf8(bytes) {
        Ok(s) => (s, "UTF-8"),
        Err(e) => (e.into_bytes().into_iter().map(cp1252).collect(), "Windows-1252"),
    }
}

/// Valor numérico «limpo» (sem zeros à esquerda, que indicam códigos como 0123).
fn numerico_valido(v: &str, virgula: bool) -> Option<f64> {
    let t = if virgula { v.replace(',', ".") } else { v.to_string() };
    if !t.chars().all(|c| c.is_ascii_digit() || "+-.eE".contains(c)) {
        return None;
    }
    let sem_sinal = t.trim_start_matches(['+', '-']);
    if sem_sinal.len() > 1 && sem_sinal.starts_with('0') && !sem_sinal[1..].starts_with('.') {
        return None;
    }
    t.parse::<f64>().ok()
}

fn inferir_tipo(linhas: &[Vec<String>], col: usize, virgula: bool) -> &'static str {
    let mut inteiro = true;
    let mut real = true;
    let mut algum = false;
    for l in linhas {
        let v = l.get(col).map(|s| s.trim()).unwrap_or("");
        if v.is_empty() {
            continue;
        }
        algum = true;
        match numerico_valido(v, virgula) {
            None => return "TEXT",
            Some(_) => {
                let t = if virgula { v.replace(',', ".") } else { v.to_string() };
                if inteiro && t.parse::<i64>().is_err() {
                    inteiro = false;
                }
            }
        }
    }
    real &= algum;
    if !algum {
        "TEXT"
    } else if inteiro {
        "INTEGER"
    } else if real {
        "REAL"
    } else {
        "TEXT"
    }
}

fn numerica(tipo: &str) -> bool {
    let t = tipo.to_ascii_uppercase();
    t.contains("INT") || t.contains("REAL") || t.contains("FLOA") || t.contains("DOUB") || t.contains("NUMERIC") || t.contains("DECIMAL")
}

/// Valida todas as linhas importadas antes de as inserir (a importação é uma transação).
fn validar_linhas(campos: &[Campo], mapa: &[(usize, String, bool)], dados: &[Vec<String>]) -> Result<(), String> {
    for &(ci, ref nome, _) in mapa {
        let Some(campo) = campos.iter().find(|f| f.nome.eq_ignore_ascii_case(nome)) else { continue };
        for (k, linha) in dados.iter().enumerate() {
            let valor = linha.get(ci).map(String::as_str).unwrap_or("");
            crate::validacao::validar(&campo.tipo, campo.obrigatorio, valor, nome)
                .map_err(|m| format!("Linha {} de dados: {}", k + 1, m))?;
        }
    }
    Ok(())
}

fn nomes_campos(cab: Option<&Vec<String>>, n: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for i in 0..n {
        let mut nome = cab.and_then(|c| c.get(i)).map(|s| s.trim().to_string()).unwrap_or_default();
        if nome.is_empty() {
            nome = format!("campo{}", i + 1);
        }
        let base = nome.clone();
        let mut k = 2;
        while out.iter().any(|o| o.eq_ignore_ascii_case(&nome)) {
            nome = format!("{}_{}", base, k);
            k += 1;
        }
        out.push(nome);
    }
    out
}

fn campo_csv(v: &str, sep: char) -> String {
    if v.contains(sep) || v.contains('"') || v.contains('\n') || v.contains('\r') {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.to_string()
    }
}

fn escrever_csv(path: &Path, cols: &[String], rows: &[Vec<String>], e: &ExportCsv) -> std::io::Result<()> {
    let sep = e.sep.to_string();
    let mut s = String::new();
    if e.cabecalho {
        let l: Vec<String> = cols.iter().map(|c| campo_csv(c, e.sep)).collect();
        s.push_str(&l.join(sep.as_str()));
        s.push_str("\r\n");
    }
    for r in rows {
        let l: Vec<String> = r.iter().map(|c| campo_csv(c, e.sep)).collect();
        s.push_str(&l.join(sep.as_str()));
        s.push_str("\r\n");
    }
    let mut bytes: Vec<u8> = Vec::new();
    if e.bom {
        bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    }
    bytes.extend_from_slice(s.as_bytes());
    std::fs::write(path, bytes)
}

impl AccessApp {
    /// «Importar → Ficheiro CSV»: escolhe o ficheiro e abre o diálogo de importação.
    pub(crate) fn pedir_importar_csv(&mut self) {
        if self.db.is_none() {
            self.msg = "Abra ou crie uma base de dados primeiro".into();
            return;
        }
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Ficheiros de texto (csv, txt, tsv)", &["csv", "txt", "tsv"])
            .pick_file()
        else {
            return;
        };
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                self.msg = format!("Não foi possível ler o ficheiro: {}", e);
                return;
            }
        };
        let (bruto, codificacao) = decodificar(bytes);
        let sep = detectar_separador(&bruto);
        let linhas = analisar(&bruto, sep);
        if linhas.is_empty() {
            self.msg = "O ficheiro está vazio".into();
            return;
        }
        let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "importada".into());
        let nome_ficheiro = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        self.importar = Some(ImportCsv {
            nome_ficheiro,
            bruto,
            codificacao,
            sep,
            sep_analisado: sep,
            cabecalho: true,
            virgula: sep != ',',
            novo: true,
            nome_novo: stem,
            destino: 0,
            linhas,
            erro: String::new(),
        });
    }

    /// «Exportar → Ficheiro CSV»: exporta o que está no separador ativo.
    pub(crate) fn pedir_exportar_csv(&mut self) {
        match self.docs.get(self.active) {
            Some(d) if matches!(d.kind, ObjKind::Table | ObjKind::Query | ObjKind::Sql) && !d.data.cols.is_empty() => {
                self.exportar_csv = Some(ExportCsv { sep: ';', cabecalho: true, bom: true });
            }
            _ => self.msg = "Abra uma tabela ou consulta com dados para a exportar".into(),
        }
    }

    pub(crate) fn dialogo_exportar_csv(&mut self, ctx: &egui::Context) {
        let Some(mut e) = self.exportar_csv.take() else { return };
        let nome = self.docs.get(self.active).map(|d| d.name.clone()).unwrap_or_default();
        let mut acao: Option<bool> = None;
        egui::Window::new("Exportar para CSV")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(format!("Vai exportar «{}», tal como está a ser mostrada (com a ordenação e os filtros).", nome));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("Separador:");
                    egui::ComboBox::from_id_salt("csv_exp_sep")
                        .width(150.0)
                        .selected_text(nome_sep(e.sep))
                        .show_ui(ui, |ui| {
                            for s in [';', ',', '\t', '|'] {
                                ui.selectable_value(&mut e.sep, s, nome_sep(s));
                            }
                        });
                });
                ui.checkbox(&mut e.cabecalho, "Incluir os nomes dos campos na primeira linha");
                ui.checkbox(&mut e.bom, "Compatível com o Excel (UTF-8 com marca BOM)");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Escolher ficheiro e exportar…").clicked() {
                        acao = Some(true);
                    }
                    if ui.button("Cancelar").clicked() {
                        acao = Some(false);
                    }
                });
            });
        match acao {
            Some(true) => self.exportar_agora(&e, &nome),
            Some(false) => {}
            None => self.exportar_csv = Some(e),
        }
    }

    fn exportar_agora(&mut self, e: &ExportCsv, nome: &str) {
        let Some(path) = rfd::FileDialog::new()
            .set_file_name(format!("{}.csv", nome))
            .add_filter("CSV", &["csv"])
            .save_file()
        else {
            // o utilizador cancelou a escolha do ficheiro: volta ao diálogo
            self.exportar_csv = Some(ExportCsv { sep: e.sep, cabecalho: e.cabecalho, bom: e.bom });
            return;
        };
        let Some(doc) = self.docs.get(self.active) else { return };
        let cols: Vec<String> = doc.data.cols.iter().map(|c| c.0.clone()).collect();
        match escrever_csv(&path, &cols, &doc.data.rows, e) {
            Ok(()) => self.msg = format!("Exportadas {} linha(s) para {}", doc.data.rows.len(), path.display()),
            Err(err) => self.msg = format!("Não foi possível gravar o ficheiro: {}", err),
        }
    }

    pub(crate) fn dialogo_importar_csv(&mut self, ctx: &egui::Context) {
        let Some(mut imp) = self.importar.take() else { return };
        let tabelas: Vec<String> = self.nav.first().map(|g| g.items.clone()).unwrap_or_default();
        let mut acao: Option<bool> = None;
        egui::Window::new("Importar Ficheiro CSV")
            .collapsible(false)
            .resizable(true)
            .default_width(620.0)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(format!("Ficheiro: {}   ({})", imp.nome_ficheiro, imp.codificacao));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Separador:");
                    egui::ComboBox::from_id_salt("csv_sep")
                        .width(150.0)
                        .selected_text(nome_sep(imp.sep))
                        .show_ui(ui, |ui| {
                            for s in [';', ',', '\t', '|'] {
                                ui.selectable_value(&mut imp.sep, s, nome_sep(s));
                            }
                        });
                });
                ui.checkbox(&mut imp.cabecalho, "A primeira linha tem os nomes dos campos");
                ui.checkbox(&mut imp.virgula, "A vírgula é o separador decimal (1,5)");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.radio_value(&mut imp.novo, true, "Criar uma tabela nova:");
                    ui.add(egui::TextEdit::singleline(&mut imp.nome_novo).id(egui::Id::new("csv_nome_novo")).desired_width(200.0));
                });
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!tabelas.is_empty(), |ui| {
                        ui.radio_value(&mut imp.novo, false, "Acrescentar à tabela:");
                        egui::ComboBox::from_id_salt("csv_dest")
                            .width(180.0)
                            .selected_text(tabelas.get(imp.destino).cloned().unwrap_or_default())
                            .show_ui(ui, |ui| {
                                for (i, t) in tabelas.iter().enumerate() {
                                    ui.selectable_value(&mut imp.destino, i, t);
                                }
                            });
                    });
                });
                ui.separator();
                let ncols = imp.linhas.iter().map(|l| l.len()).max().unwrap_or(0);
                let dados = imp.linhas.len().saturating_sub(if imp.cabecalho { 1 } else { 0 });
                ui.label(format!("{} linha(s) de dados, {} coluna(s). Pré-visualização:", dados, ncols));
                egui::ScrollArea::both().id_salt("csv_prev_scroll").max_height(170.0).show(ui, |ui| {
                    egui::Grid::new("csv_prev").striped(true).spacing([14.0, 4.0]).show(ui, |ui| {
                        for (ri, l) in imp.linhas.iter().take(7).enumerate() {
                            for c in l.iter().take(10) {
                                let t = abreviar(c, 22);
                                if ri == 0 && imp.cabecalho {
                                    ui.label(RichText::new(t).strong());
                                } else {
                                    ui.label(t);
                                }
                            }
                            ui.end_row();
                        }
                    });
                });
                if !imp.erro.is_empty() {
                    ui.add_space(4.0);
                    ui.colored_label(egui::Color32::from_rgb(0xB0, 0x00, 0x20), imp.erro.clone());
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("Importar").min_size(Vec2::new(90.0, 24.0))).clicked() {
                        acao = Some(true);
                    }
                    if ui.button("Cancelar").clicked() {
                        acao = Some(false);
                    }
                });
            });
        if imp.sep != imp.sep_analisado {
            imp.linhas = analisar(&imp.bruto, imp.sep);
            imp.sep_analisado = imp.sep;
            imp.erro.clear();
        }
        match acao {
            Some(true) => match self.fazer_importacao(&imp) {
                Ok(m) => self.msg = m,
                Err(e) => {
                    imp.erro = e;
                    self.importar = Some(imp);
                }
            },
            Some(false) => {}
            None => self.importar = Some(imp),
        }
    }

    fn fazer_importacao(&mut self, imp: &ImportCsv) -> Result<String, String> {
        if imp.linhas.is_empty() {
            return Err("O ficheiro está vazio".into());
        }
        let ncols = imp.linhas.iter().map(|l| l.len()).max().unwrap_or(0);
        if ncols == 0 {
            return Err("O ficheiro não tem colunas".into());
        }
        let (cab, dados): (Option<&Vec<String>>, &[Vec<String>]) =
            if imp.cabecalho { (imp.linhas.first(), &imp.linhas[1..]) } else { (None, &imp.linhas[..]) };
        let nomes = nomes_campos(cab, ncols);
        if imp.novo {
            let tipos: Vec<&str> = (0..ncols).map(|c| inferir_tipo(dados, c, imp.virgula)).collect();
            let campos: Vec<Campo> = nomes
                .iter()
                .zip(tipos.iter())
                .map(|(n, t)| Campo { nome: n.clone(), tipo: t.to_string(), ..Default::default() })
                .collect();
            let mapa: Vec<(usize, String, bool)> =
                (0..ncols).map(|i| (i, nomes[i].clone(), tipos[i] != "TEXT")).collect();
            validar_linhas(&campos, &mapa, dados)?;
            let nome = imp.nome_novo.trim().to_string();
            let n = {
                let db = self.db.as_ref().ok_or("Nenhuma base de dados aberta")?;
                db.criar_tabela(&nome, &campos)?;
                match db.inserir_linhas(&nome, &mapa, dados, imp.virgula) {
                    Ok(n) => n,
                    Err(e) => {
                        let _ = db.eliminar_tabela(&nome);
                        return Err(e);
                    }
                }
            };
            self.limpar_historico();
            self.refresh_nav();
            self.open_doc(ObjKind::Table, nome.clone());
            Ok(format!("Importadas {} linha(s) para a tabela nova «{}»", n, nome))
        } else {
            let tabelas: Vec<String> = self.nav.first().map(|g| g.items.clone()).unwrap_or_default();
            let destino = tabelas.get(imp.destino).cloned().ok_or("Escolha a tabela de destino")?;
            let (campos, _) = self.db.as_ref().ok_or("Nenhuma base de dados aberta")?.info_tabela(&destino)?;
            let mut mapa: Vec<(usize, String, bool)> = Vec::new();
            if imp.cabecalho {
                for (i, n) in nomes.iter().enumerate() {
                    if let Some(c) = campos.iter().find(|c| c.nome.eq_ignore_ascii_case(n)) {
                        mapa.push((i, c.nome.clone(), numerica(&c.tipo)));
                    }
                }
            } else {
                for (i, c) in campos.iter().enumerate().take(ncols) {
                    mapa.push((i, c.nome.clone(), numerica(&c.tipo)));
                }
            }
            if mapa.is_empty() {
                return Err("Nenhuma coluna do ficheiro corresponde a campos da tabela (compare os nomes)".into());
            }
            validar_linhas(&campos, &mapa, dados)?;
            let n = self
                .db
                .as_ref()
                .ok_or("Nenhuma base de dados aberta")?
                .inserir_linhas(&destino, &mapa, dados, imp.virgula)?;
            self.limpar_historico();
            self.recarregar_abertos(&destino);
            let ignoradas = ncols - mapa.len().min(ncols);
            let extra = if ignoradas > 0 { format!(" ({} coluna(s) do ficheiro sem campo correspondente foram ignoradas)", ignoradas) } else { String::new() };
            Ok(format!("Importadas {} linha(s) para «{}»{}", n, destino, extra))
        }
    }
}

