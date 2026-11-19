//! Camada de dados do AccesSQL (SQLite local, via rusqlite).

use rusqlite::{params, params_from_iter, types::Value, types::ValueRef, Connection, OpenFlags, Row, Statement};
use std::{collections::HashMap, path::Path};

/// Conteúdo de uma tabela ou consulta já carregado em memória.
#[derive(Default)]
pub(crate) struct TableData {
    /// Cada coluna: (nome, largura em píxeis na folha de dados).
    pub(crate) cols: Vec<(String, f32)>,
    /// Tipo declarado de cada coluna (vazio quando o SQLite não o conhece).
    pub(crate) tipos: Vec<String>,
    /// `true` nas colunas `NOT NULL`.
    pub(crate) obrig: Vec<bool>,
    pub(crate) rows: Vec<Vec<String>>,
    /// `rowid` de cada linha (só em tabelas editáveis; vazio em consultas).
    pub(crate) rowids: Vec<i64>,
    /// `true` se for uma tabela com `rowid` (permite editar, inserir e eliminar).
    pub(crate) editable: bool,
    /// `true` se o resultado foi cortado por ter demasiadas linhas.
    pub(crate) truncado: bool,
}

/// Um campo (coluna) de uma tabela, tal como aparece na Vista Estrutura.
#[derive(Clone, Default)]
pub(crate) struct Campo {
    pub(crate) nome: String,
    pub(crate) tipo: String,
    pub(crate) obrigatorio: bool,
    pub(crate) omissao: String,
    pub(crate) pk: bool,
    /// Chave estrangeira: tabela e campo a que este campo se refere (vazios = nenhuma).
    pub(crate) ref_tabela: String,
    pub(crate) ref_coluna: String,
    /// Restrição UNIQUE ao nível da coluna.
    pub(crate) unico: bool,
    /// Corpo de uma restrição CHECK ao nível da coluna (sem o `CHECK (` inicial).
    pub(crate) check: String,
    /// Chave primária INTEGER com numeração automática (`AUTOINCREMENT`).
    pub(crate) autoincrement: bool,
}

/// Uma relação (chave estrangeira) entre duas tabelas.
pub(crate) struct Relacao {
    pub(crate) tabela: String,
    pub(crate) colunas: String,
    pub(crate) ref_tabela: String,
    pub(crate) ref_colunas: String,
    pub(crate) ao_apagar: String,
    pub(crate) ao_atualizar: String,
}

/// Um índice de uma tabela.
pub(crate) struct Indice {
    pub(crate) nome: String,
    pub(crate) unico: bool,
    /// `c` = criado com CREATE INDEX, `u` = restrição UNIQUE, `pk` = chave primária.
    pub(crate) origem: String,
    pub(crate) colunas: Vec<String>,
}

/// Tipos de dados oferecidos (nomes declarados do SQLite).
pub(crate) const TIPOS: [&str; 8] = ["TEXT", "INTEGER", "REAL", "NUMERIC", "DATE", "DATETIME", "BOOLEAN", "BLOB"];

/// Máximo de linhas mostradas no resultado de uma consulta SQL.
const LIMITE_SQL: usize = 100_000;

/// Resultado de executar um script SQL.
pub(crate) struct ResultadoSql {
    pub(crate) dados: Option<TableData>,
    pub(crate) afetadas: usize,
    pub(crate) instrucoes: usize,
}

pub(crate) struct Database {
    conn: Connection,
}

/// Escapa um identificador SQL: "nome" (aspas duplicadas dentro do nome).
pub(crate) fn quote(id: &str) -> String {
    format!("\"{}\"", id.replace('"', "\"\""))
}

/// Mensagens de erro do SQLite em português, para a barra de estado.
pub(crate) fn traduz(e: rusqlite::Error) -> String {
    let s = e.to_string();
    if s.contains("not a database") || s.contains("encrypted") {
        "O ficheiro não é uma base de dados SQLite válida".into()
    } else if s.contains("FOREIGN KEY constraint failed") {
        "Viola uma relação: o valor não existe na tabela relacionada, ou há registos que dependem deste".into()
    } else if s.contains("NOT NULL constraint failed") {
        "Há registos com o campo vazio: não é possível torná-lo obrigatório sem corrigir esses registos".into()
    } else if s.contains("UNIQUE constraint failed") {
        "Já existe um registo com esse valor num campo que não admite repetições".into()
    } else if s.contains("CHECK constraint failed") {
        "O valor não cumpre a verificação definida para o campo".into()
    } else if s.contains("AUTOINCREMENT is only allowed") {
        "A numeração automática só é possível numa chave primária INTEGER".into()
    } else if s.contains("unable to open") {
        "Não foi possível abrir o ficheiro (confirme o caminho e as permissões)".into()
    } else {
        s
    }
}

/// Texto vazio grava-se como NULL (como no Access); o resto como texto.
/// O SQLite converte para número se a coluna tiver afinidade numérica.
pub(crate) fn valor(t: &str) -> Value {
    if t.is_empty() {
        Value::Null
    } else {
        Value::Text(t.to_string())
    }
}

fn celula(v: ValueRef) -> String {
    match v {
        ValueRef::Null => String::new(),
        ValueRef::Integer(x) => x.to_string(),
        ValueRef::Real(x) => x.to_string(),
        ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
        ValueRef::Blob(_) => "[BLOB]".to_string(),
    }
}

fn linha(r: &Row, de: usize, ate: usize) -> rusqlite::Result<Vec<String>> {
    let mut v = Vec::with_capacity(ate - de);
    for i in de..ate {
        v.push(celula(r.get_ref(i)?));
    }
    Ok(v)
}

/// Operadores dos filtros da folha de dados.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Op {
    Igual,
    Diferente,
    Contem,
    NaoContem,
}

/// Uma condição de filtro sobre uma coluna.
#[derive(Clone)]
pub(crate) struct Condicao {
    pub(crate) col: String,
    pub(crate) op: Op,
    pub(crate) valor: String,
}

fn escapar_like(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

impl Condicao {
    pub(crate) fn descricao(&self) -> String {
        let v = if self.valor.is_empty() { "(vazio)".to_string() } else { self.valor.clone() };
        match self.op {
            Op::Igual => format!("{} = «{}»", self.col, v),
            Op::Diferente => format!("{} ≠ «{}»", self.col, v),
            Op::Contem => format!("{} contém «{}»", self.col, v),
            Op::NaoContem => format!("{} não contém «{}»", self.col, v),
        }
    }

    /// Texto SQL da condição; os valores vão em `params` (um `?` por valor).
    fn sql(&self, params: &mut Vec<String>) -> String {
        let c = quote(&self.col);
        match self.op {
            Op::Igual => {
                if self.valor.is_empty() {
                    format!("({c} IS NULL OR CAST({c} AS TEXT) = '')")
                } else {
                    params.push(self.valor.clone());
                    format!("CAST({c} AS TEXT) = ?")
                }
            }
            Op::Diferente => {
                if self.valor.is_empty() {
                    format!("({c} IS NOT NULL AND CAST({c} AS TEXT) <> '')")
                } else {
                    params.push(self.valor.clone());
                    format!("({c} IS NULL OR CAST({c} AS TEXT) <> ?)")
                }
            }
            Op::Contem => {
                params.push(format!("%{}%", escapar_like(&self.valor)));
                format!("CAST({c} AS TEXT) LIKE ? ESCAPE '\\'")
            }
            Op::NaoContem => {
                params.push(format!("%{}%", escapar_like(&self.valor)));
                format!("({c} IS NULL OR CAST({c} AS TEXT) NOT LIKE ? ESCAPE '\\')")
            }
        }
    }
}

/// Lê o resultado de uma instrução já preparada (ignora as primeiras `skip` colunas).
fn ler(stmt: &mut Statement, skip: usize, com_rowid: bool, limite: usize, params: &[String]) -> rusqlite::Result<TableData> {
    let total = stmt.column_count();
    let nomes: Vec<String> = stmt.column_names().into_iter().skip(skip).map(String::from).collect();
    let mut rows_out: Vec<Vec<String>> = Vec::new();
    let mut rowids: Vec<i64> = Vec::new();
    let mut truncado = false;
    let mut rows = stmt.query(params_from_iter(params.iter()))?;
    while let Some(r) = rows.next()? {
        if rows_out.len() >= limite {
            truncado = true;
            break;
        }
        if com_rowid {
            rowids.push(r.get::<_, i64>(0)?);
        }
        rows_out.push(linha(r, skip, total)?);
    }
    // largura de cada coluna: cabeçalho + até 200 linhas de amostra
    let cols = nomes
        .into_iter()
        .enumerate()
        .map(|(i, nome)| {
            let mut max = nome.chars().count();
            for row in rows_out.iter().take(200) {
                max = max.max(row[i].chars().count());
            }
            (nome, (max as f32 * 7.2 + 26.0).clamp(70.0, 320.0))
        })
        .collect();
    Ok(TableData { cols, tipos: Vec::new(), obrig: Vec::new(), rows: rows_out, rowids, editable: com_rowid, truncado })
}

#[cfg(test)]
mod testes {
    use super::*;

    fn tabela_teste() -> Database {
        let db = Database::em_memoria().unwrap();
        db.criar_tabela(
            "Clientes",
            &[
                Campo { nome: "ID".into(), tipo: "INTEGER".into(), pk: true, ..Default::default() },
                Campo { nome: "Nome".into(), tipo: "TEXT".into(), ..Default::default() },
            ],
        )
        .unwrap();
        db
    }

    #[test]
    fn metadados_da_tabela_chegam_a_folha_de_dados() {
        let db = tabela_teste();
        let (campos, _) = db.info_tabela("Clientes").unwrap();
        let id = campos.iter().find(|c| c.nome == "ID").unwrap();
        assert!(id.tipo.contains("INT"), "tipo de ID: {}", id.tipo);
        let dados = db.load("Clientes", true).unwrap();
        assert_eq!(dados.tipos.len(), dados.cols.len());
        assert_eq!(dados.obrig.len(), dados.cols.len());
        let i = dados.cols.iter().position(|c| c.0 == "ID").unwrap();
        assert!(dados.tipos[i].contains("INT"));
    }

    #[test]
    fn alterar_campo_preserva_dados_indices_e_integridade() {
        let db = tabela_teste();
        db.conn
            .execute("INSERT INTO \"Clientes\" (\"Nome\") VALUES ('42')", [])
            .unwrap();
        db.conn.execute("CREATE INDEX \"ix_nome\" ON \"Clientes\" (\"Nome\")", []).unwrap();

        let novo = Campo { tipo: "INTEGER".into(), obrigatorio: true, omissao: "0".into(), ..Default::default() };
        db.alterar_campo("Clientes", "Nome", &novo).unwrap();

        let (campos, _) = db.info_tabela("Clientes").unwrap();
        let nome = campos.iter().find(|c| c.nome == "Nome").unwrap();
        assert_eq!(nome.tipo, "INTEGER");
        assert!(nome.obrigatorio);
        assert_eq!(nome.omissao, "0");
        assert!(campos.iter().find(|c| c.nome == "ID").unwrap().pk);

        let dados = db.load("Clientes", true).unwrap();
        let i = dados.cols.iter().position(|c| c.0 == "Nome").unwrap();
        assert_eq!(dados.rows[0][i], "42");
        let indice: i64 = db
            .conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'index' AND name = 'ix_nome'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(indice, 1, "o índice deve sobreviver à recriação");
        let integridade: String = db.conn.query_row("PRAGMA integrity_check", [], |r| r.get(0)).unwrap();
        assert_eq!(integridade, "ok");
    }

    #[test]
    fn alterar_campo_com_nulos_falha_sem_mudar_a_tabela() {
        let db = tabela_teste();
        db.conn.execute("INSERT INTO \"Clientes\" (\"Nome\") VALUES (NULL)", []).unwrap();
        let (antes, _) = db.info_tabela("Clientes").unwrap();

        let novo = Campo { tipo: "TEXT".into(), obrigatorio: true, ..Default::default() };
        let erro = db.alterar_campo("Clientes", "Nome", &novo).unwrap_err();
        assert!(erro.contains("obrigatório"), "mensagem: {}", erro);

        let (depois, _) = db.info_tabela("Clientes").unwrap();
        assert!(!depois.iter().find(|c| c.nome == "Nome").unwrap().obrigatorio);
        assert_eq!(antes.len(), depois.len());
        assert_eq!(db.contar_registos("Clientes").unwrap(), 1);
        let integridade: String = db.conn.query_row("PRAGMA integrity_check", [], |r| r.get(0)).unwrap();
        assert_eq!(integridade, "ok");
    }

    #[test]
    fn alterar_campo_preserva_acoes_das_relacoes() {
        let db = Database::em_memoria().unwrap();
        db.conn
            .execute_batch(
                "CREATE TABLE \"Pais\" (\"ID\" INTEGER PRIMARY KEY);\n\
                 CREATE TABLE \"Filhos\" (\
                     \"ID\" INTEGER PRIMARY KEY, \
                     \"Pai\" INTEGER REFERENCES \"Pais\"(\"ID\") ON DELETE CASCADE\
                 );\n\
                 INSERT INTO \"Pais\" (\"ID\") VALUES (1);\n\
                 INSERT INTO \"Filhos\" (\"ID\", \"Pai\") VALUES (1, 1);",
            )
            .unwrap();

        let novo = Campo { tipo: "INTEGER".into(), ..Default::default() };
        db.alterar_campo("Filhos", "Pai", &novo).unwrap();

        let mut stmt = db.conn.prepare("PRAGMA foreign_key_list('Filhos')").unwrap();
        let acoes: Vec<(String, String)> = stmt
            .query_map([], |r| Ok((r.get::<_, String>(3)?, r.get::<_, String>(6)?)))
            .unwrap()
            .flatten()
            .collect();
        let acao = acoes.iter().find(|(c, _)| c == "Pai").map(|(_, a)| a.as_str());
        assert_eq!(acao, Some("CASCADE"), "a ação ON DELETE deve sobreviver");
    }

    #[test]
    fn criar_tabela_com_unique_check_e_autoincrement() {
        let db = Database::em_memoria().unwrap();
        let campos = [
            Campo { nome: "ID".into(), tipo: "INTEGER".into(), pk: true, autoincrement: true, ..Default::default() },
            Campo {
                nome: "Nome".into(),
                tipo: "TEXT".into(),
                unico: true,
                check: "length(\"Nome\") > 0".into(),
                ..Default::default()
            },
        ];
        db.criar_tabela("Pessoas", &campos).unwrap();

        let (lidos, sql) = db.info_tabela("Pessoas").unwrap();
        assert!(lidos[0].autoincrement, "AUTOINCREMENT deve ser lido do catálogo");
        assert!(lidos[1].unico, "UNIQUE deve ser lido do catálogo");
        assert_eq!(lidos[1].check, "length(\"Nome\") > 0");
        assert!(sql.contains("AUTOINCREMENT") && sql.contains("UNIQUE") && sql.contains("CHECK"));

        db.conn.execute("INSERT INTO \"Pessoas\" (\"Nome\") VALUES ('Ana')", []).unwrap();
        let erro = db.conn.execute("INSERT INTO \"Pessoas\" (\"Nome\") VALUES ('Ana')", []).unwrap_err();
        assert!(erro.to_string().contains("UNIQUE"), "UNIQUE deve ser aplicado: {}", erro);
        let erro = db.conn.execute("INSERT INTO \"Pessoas\" (\"Nome\") VALUES ('')", []).unwrap_err();
        assert!(erro.to_string().contains("CHECK"), "CHECK deve ser aplicado: {}", erro);
    }

    #[test]
    fn alterar_campo_preserva_restricoes_inline_e_de_tabela() {
        let db = Database::em_memoria().unwrap();
        db.conn
            .execute_batch(
                "CREATE TABLE \"P\" (\
                     \"ID\" INTEGER PRIMARY KEY AUTOINCREMENT, \
                     \"Nome\" TEXT UNIQUE CHECK(length(\"Nome\") > 0), \
                     \"Cidade\" TEXT, \
                     CONSTRAINT \"uq_nome_cidade\" UNIQUE (\"Nome\", \"Cidade\"), \
                     CHECK (\"Cidade\" <> 'x')\
                 );\
                 INSERT INTO \"P\" (\"Nome\", \"Cidade\") VALUES ('Ana', 'Porto');",
            )
            .unwrap();

        let (campos, _) = db.info_tabela("P").unwrap();
        let nome = campos.iter().find(|c| c.nome == "Nome").unwrap();
        assert!(nome.unico && !nome.autoincrement);
        assert_eq!(nome.check, "length(\"Nome\") > 0");

        let novo = Campo {
            tipo: "TEXT".into(),
            unico: true,
            check: "length(\"Nome\") > 0".into(),
            ..Default::default()
        };
        db.alterar_campo("P", "Nome", &novo).unwrap();

        let sql: String = db
            .conn
            .query_row("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'P'", [], |r| r.get(0))
            .unwrap();
        assert!(sql.contains("AUTOINCREMENT"), "sql: {sql}");
        assert!(sql.contains("uq_nome_cidade"), "a restrição de tabela com nome deve sobreviver: {sql}");
        assert!(sql.contains("UNIQUE (\"Nome\", \"Cidade\")"), "sql: {sql}");
        assert!(sql.contains("\"Cidade\" <> 'x'"), "sql: {sql}");
        assert_eq!(db.contar_registos("P").unwrap(), 1);

        let erro = db.conn.execute("INSERT INTO \"P\" (\"Nome\", \"Cidade\") VALUES ('Ana', 'Porto')", []).unwrap_err();
        assert!(erro.to_string().contains("UNIQUE"), "sql: {sql}");
        let integridade: String = db.conn.query_row("PRAGMA integrity_check", [], |r| r.get(0)).unwrap();
        assert_eq!(integridade, "ok");
    }

    #[test]
    fn acrescentar_campo_aceita_check_mas_recusa_unique_e_autoincrement() {
        let db = tabela_teste();
        let erro = db
            .acrescentar_campo("Clientes", &Campo { nome: "x".into(), tipo: "TEXT".into(), unico: true, ..Default::default() })
            .unwrap_err();
        assert!(erro.contains("UNIQUE"), "mensagem: {}", erro);
        let erro = db
            .acrescentar_campo(
                "Clientes",
                &Campo { nome: "x".into(), tipo: "INTEGER".into(), autoincrement: true, ..Default::default() },
            )
            .unwrap_err();
        assert!(erro.contains("numeração automática"), "mensagem: {}", erro);

        db.acrescentar_campo(
            "Clientes",
            &Campo { nome: "x".into(), tipo: "INTEGER".into(), check: "x > 0".into(), ..Default::default() },
        )
        .unwrap();
        let sql: String = db
            .conn
            .query_row("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'Clientes'", [], |r| r.get(0))
            .unwrap();
        assert!(sql.contains("CHECK (x > 0)"), "sql: {sql}");
    }

    #[test]
    fn parser_respeita_virgulas_e_palavras_dentro_de_textos() {
        let sql = "CREATE TABLE t (a TEXT, b TEXT CHECK (b IN ('x,y', 'UNIQUE')), UNIQUE (a, b))";
        let segs = segmentos_da_tabela(sql);
        assert_eq!(segs.len(), 3, "segmentos: {segs:?}");
        assert!(segs[1].contains("'x,y'"));
        assert_eq!(restricoes_tabela(sql).len(), 1);
        let mut campos = vec![
            Campo { nome: "a".into(), tipo: "TEXT".into(), ..Default::default() },
            Campo { nome: "b".into(), tipo: "TEXT".into(), ..Default::default() },
        ];
        aplicar_restricoes_inline(&mut campos, sql);
        assert!(!campos[0].unico, "UNIQUE de tabela não deve virar UNIQUE da coluna");
        assert!(campos[1].check.contains("'UNIQUE'"), "check: {}", campos[1].check);
    }
}

/// Divide um script em instruções, respeitando textos, identificadores e comentários.
fn dividir(sql: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut conteudo = false;
    let mut it = sql.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\'' | '"' | '`' => {
                cur.push(c);
                conteudo = true;
                for d in it.by_ref() {
                    cur.push(d);
                    if d == c {
                        break;
                    }
                }
            }
            '[' => {
                cur.push(c);
                conteudo = true;
                for d in it.by_ref() {
                    cur.push(d);
                    if d == ']' {
                        break;
                    }
                }
            }
            '-' if it.peek() == Some(&'-') => {
                cur.push(c);
                while let Some(&d) = it.peek() {
                    if d == '\n' {
                        break;
                    }
                    cur.push(d);
                    it.next();
                }
            }
            '/' if it.peek() == Some(&'*') => {
                cur.push(c);
                if let Some(d) = it.next() {
                    cur.push(d);
                }
                let mut prev = ' ';
                for d in it.by_ref() {
                    cur.push(d);
                    if prev == '*' && d == '/' {
                        break;
                    }
                    prev = d;
                }
            }
            ';' => {
                if conteudo {
                    out.push(cur.trim().to_string());
                }
                cur.clear();
                conteudo = false;
            }
            c if !c.is_whitespace() => {
                cur.push(c);
                conteudo = true;
            }
            c => cur.push(c),
        }
    }
    if conteudo {
        out.push(cur.trim().to_string());
    }
    out
}

/// De `CREATE VIEW "x" AS SELECT ...` devolve só o `SELECT ...`.
fn corpo_vista(sql: &str) -> String {
    let b = sql.as_bytes();
    let lower = sql.to_ascii_lowercase();
    let lb = lower.as_bytes();
    let mut aspas: Option<u8> = None;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = aspas {
            if c == q {
                aspas = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => aspas = Some(c),
            b'[' => aspas = Some(b']'),
            _ => {
                if i > 0
                    && b[i - 1].is_ascii_whitespace()
                    && lb[i..].starts_with(b"as")
                    && i + 2 < b.len()
                    && b[i + 2].is_ascii_whitespace()
                {
                    return sql[i + 2..].trim().to_string();
                }
            }
        }
        i += 1;
    }
    sql.to_string()
}

/// Valor por omissão escrito pelo utilizador -> literal SQL.
fn literal_omissao(t: &str) -> String {
    let t = t.trim();
    let up = t.to_ascii_uppercase();
    if matches!(up.as_str(), "NULL" | "CURRENT_TIMESTAMP" | "CURRENT_DATE" | "CURRENT_TIME" | "TRUE" | "FALSE") {
        return up;
    }
    if t.chars().all(|c| c.is_ascii_digit() || "+-.eE".contains(c)) && t.parse::<f64>().is_ok() {
        return t.to_string();
    }
    format!("'{}'", t.replace('\'', "''"))
}

/// Divide o interior de um `CREATE TABLE (...)` por vírgulas de topo, ignorando
/// textos, identificadores e parênteses internos (expressões CHECK, por exemplo).
fn segmentos_da_tabela(sql: &str) -> Vec<String> {
    let b = sql.as_bytes();
    let mut i = 0;
    let mut aspas: Option<u8> = None;
    while i < b.len() {
        if let Some(q) = aspas {
            if b[i] == q {
                if i + 1 < b.len() && b[i + 1] == q {
                    i += 2;
                    continue;
                }
                aspas = None;
            }
            i += 1;
            continue;
        }
        match b[i] {
            b'\'' | b'"' | b'`' => {
                aspas = Some(b[i]);
                i += 1;
            }
            b'[' => {
                aspas = Some(b']');
                i += 1;
            }
            b'(' => break,
            _ => i += 1,
        }
    }
    if i >= b.len() {
        return Vec::new();
    }
    i += 1; // abre o parêntese da lista de campos
    let mut out = Vec::new();
    let mut atual: Vec<u8> = Vec::new();
    let mut nivel = 1_i32;
    aspas = None;
    while i < b.len() {
        if let Some(q) = aspas {
            atual.push(b[i]);
            if b[i] == q {
                if i + 1 < b.len() && b[i + 1] == q {
                    atual.push(b[i + 1]);
                    i += 2;
                    continue;
                }
                aspas = None;
            }
            i += 1;
            continue;
        }
        match b[i] {
            b'\'' | b'"' | b'`' => {
                aspas = Some(b[i]);
                atual.push(b[i]);
                i += 1;
            }
            b'[' => {
                aspas = Some(b']');
                atual.push(b[i]);
                i += 1;
            }
            b'(' => {
                nivel += 1;
                atual.push(b[i]);
                i += 1;
            }
            b')' => {
                nivel -= 1;
                if nivel == 0 {
                    if !atual.iter().all(u8::is_ascii_whitespace) {
                        out.push(String::from_utf8_lossy(&atual).trim().to_string());
                    }
                    break;
                }
                atual.push(b[i]);
                i += 1;
            }
            b',' if nivel == 1 => {
                if !atual.iter().all(u8::is_ascii_whitespace) {
                    out.push(String::from_utf8_lossy(&atual).trim().to_string());
                }
                atual.clear();
                i += 1;
            }
            _ => {
                atual.push(b[i]);
                i += 1;
            }
        }
    }
    out
}

/// Primeiro identificador de um segmento (`"nome" INTEGER ...`) e o que vem a seguir.
fn primeiro_ident(s: &str) -> (String, String) {
    let t = s.trim_start();
    let mut it = t.chars().peekable();
    let Some(&primeiro) = it.peek() else {
        return (String::new(), String::new());
    };
    let mut ident = String::new();
    if matches!(primeiro, '"' | '`' | '\'') {
        ident.push(it.next().unwrap());
        while let Some(c) = it.next() {
            ident.push(c);
            if c == primeiro {
                if it.peek() == Some(&primeiro) {
                    ident.push(it.next().unwrap());
                    continue;
                }
                break;
            }
        }
    } else if primeiro == '[' {
        ident.push(it.next().unwrap());
        for c in it.by_ref() {
            ident.push(c);
            if c == ']' {
                break;
            }
        }
    } else {
        while let Some(&c) = it.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                it.next();
            } else {
                break;
            }
        }
    }
    (ident, it.collect())
}

/// Nome de um identificador SQL, sem as aspas envolventes.
fn dequote_ident(s: &str) -> String {
    let (ident, _) = primeiro_ident(s);
    let t = ident.trim();
    if t.len() >= 2 {
        let primeiro = t.chars().next().unwrap();
        let ultimo = t.chars().last().unwrap();
        if matches!(primeiro, '"' | '`' | '\'') && ultimo == primeiro {
            let duplo = format!("{primeiro}{primeiro}");
            return t[1..t.len() - 1].replace(&duplo, &primeiro.to_string());
        }
        if primeiro == '[' && ultimo == ']' {
            return t[1..t.len() - 1].to_string();
        }
    }
    t.to_string()
}

/// `true` se `s` começa pela palavra `kw` (comparação sem diferenciar maiúsculas).
fn palavra_no_inicio(s: &str, kw: &str) -> bool {
    let t = s.trim_start();
    let n = kw.len();
    if t.len() < n || !t.as_bytes()[..n].eq_ignore_ascii_case(kw.as_bytes()) {
        return false;
    }
    match t.as_bytes().get(n) {
        Some(c) => !(c.is_ascii_alphanumeric() || *c == b'_'),
        None => true,
    }
}

/// Remove um `CONSTRAINT <nome>` inicial, para reconhecer o tipo de restrição.
fn sem_constraint(s: &str) -> String {
    let t = s.trim_start();
    if !palavra_no_inicio(t, "CONSTRAINT") {
        return t.to_string();
    }
    let (_, resto) = primeiro_ident(&t["CONSTRAINT".len()..]);
    resto.trim_start().to_string()
}

/// Procura uma palavra-chave fora de textos e de parênteses (depth zero).
fn contem_palavra_fora(s: &str, kw: &str) -> bool {
    let b = s.as_bytes();
    let k = kw.as_bytes();
    let mut i = 0;
    let mut nivel = 0_i32;
    let mut aspas: Option<u8> = None;
    while i < b.len() {
        if let Some(q) = aspas {
            if b[i] == q {
                if i + 1 < b.len() && b[i + 1] == q {
                    i += 2;
                    continue;
                }
                aspas = None;
            }
            i += 1;
            continue;
        }
        match b[i] {
            b'\'' | b'"' | b'`' => {
                aspas = Some(b[i]);
                i += 1;
                continue;
            }
            b'[' => {
                aspas = Some(b']');
                i += 1;
                continue;
            }
            b'(' => {
                nivel += 1;
                i += 1;
                continue;
            }
            b')' => {
                nivel -= 1;
                i += 1;
                continue;
            }
            _ => {}
        }
        if nivel == 0 && i + k.len() <= b.len() && b[i..i + k.len()].eq_ignore_ascii_case(k) {
            let antes_ok = i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_');
            let fim = i + k.len();
            let depois_ok = fim >= b.len() || !(b[fim].is_ascii_alphanumeric() || b[fim] == b'_');
            if antes_ok && depois_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// De um segmento com `CHECK (...)`, devolve só a expressão entre parênteses.
fn extrair_check(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let k = b"CHECK";
    let mut i = 0;
    let mut nivel = 0_i32;
    let mut aspas: Option<u8> = None;
    while i < b.len() {
        if let Some(q) = aspas {
            if b[i] == q {
                if i + 1 < b.len() && b[i + 1] == q {
                    i += 2;
                    continue;
                }
                aspas = None;
            }
            i += 1;
            continue;
        }
        match b[i] {
            b'\'' | b'"' | b'`' => {
                aspas = Some(b[i]);
                i += 1;
                continue;
            }
            b'[' => {
                aspas = Some(b']');
                i += 1;
                continue;
            }
            b'(' => {
                nivel += 1;
                i += 1;
                continue;
            }
            b')' => {
                nivel -= 1;
                i += 1;
                continue;
            }
            _ => {}
        }
        if nivel != 0 || i + k.len() > b.len() || !b[i..i + k.len()].eq_ignore_ascii_case(k) {
            i += 1;
            continue;
        }
        let antes_ok = i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_');
        let fim = i + k.len();
        let depois_ok = fim >= b.len() || !(b[fim].is_ascii_alphanumeric() || b[fim] == b'_');
        if !antes_ok || !depois_ok {
            i += 1;
            continue;
        }
        let mut j = fim;
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        if b.get(j) != Some(&b'(') {
            return None;
        }
        let ini = j + 1;
        let mut d = 1_i32;
        aspas = None;
        j = ini;
        while j < b.len() {
            if let Some(q) = aspas {
                if b[j] == q {
                    if j + 1 < b.len() && b[j + 1] == q {
                        j += 2;
                        continue;
                    }
                    aspas = None;
                }
                j += 1;
                continue;
            }
            match b[j] {
                b'\'' | b'"' | b'`' => aspas = Some(b[j]),
                b'[' => aspas = Some(b']'),
                b'(' => d += 1,
                b')' => {
                    d -= 1;
                    if d == 0 {
                        return Some(s[ini..j].to_string());
                    }
                }
                _ => {}
            }
            j += 1;
        }
        return None;
    }
    None
}

/// Lê do `CREATE TABLE` original as restrições UNIQUE/CHECK/AUTOINCREMENT
/// definidas ao nível de cada coluna e completa os campos do catálogo.
fn aplicar_restricoes_inline(campos: &mut [Campo], sql: &str) {
    for seg in segmentos_da_tabela(sql) {
        let t = seg.trim();
        // CONSTRAINT/PRIMARY/UNIQUE/CHECK/FOREIGN no início = restrição de tabela.
        if palavra_no_inicio(t, "CONSTRAINT")
            || palavra_no_inicio(t, "PRIMARY")
            || palavra_no_inicio(t, "UNIQUE")
            || palavra_no_inicio(t, "CHECK")
            || palavra_no_inicio(t, "FOREIGN")
        {
            continue;
        }
        let nome = dequote_ident(t);
        let (_, resto) = primeiro_ident(t);
        let Some(c) = campos.iter_mut().find(|c| c.nome.eq_ignore_ascii_case(&nome)) else {
            continue;
        };
        if contem_palavra_fora(&resto, "AUTOINCREMENT") {
            c.autoincrement = true;
            c.pk = true;
        }
        if contem_palavra_fora(&resto, "UNIQUE") {
            c.unico = true;
        }
        if let Some(expr) = extrair_check(&resto) {
            c.check = expr;
        }
    }
}

/// Restrições de tabela `UNIQUE (...)` e `CHECK (...)` do `CREATE TABLE` original.
/// As chaves primárias e as relações são regeneradas a partir do catálogo.
pub(crate) fn restricoes_tabela(sql: &str) -> Vec<String> {
    segmentos_da_tabela(sql)
        .into_iter()
        .filter(|seg| {
            let t = seg.trim();
            let alvo = sem_constraint(t);
            palavra_no_inicio(&alvo, "UNIQUE") || palavra_no_inicio(&alvo, "CHECK")
        })
        .map(|seg| seg.trim().to_string())
        .collect()
}

/// Definição SQL de uma coluna:
/// `"nome" TIPO [PRIMARY KEY] [AUTOINCREMENT] [NOT NULL] [UNIQUE] [DEFAULT x] [CHECK (...)] [REFERENCES ...]`.
fn definicao(c: &Campo, pk_inline: bool) -> String {
    let mut s = format!("{} {}", quote(c.nome.trim()), c.tipo);
    if pk_inline {
        s.push_str(" PRIMARY KEY");
        if c.autoincrement {
            s.push_str(" AUTOINCREMENT");
        }
    }
    if c.obrigatorio {
        s.push_str(" NOT NULL");
    }
    if c.unico {
        s.push_str(" UNIQUE");
    }
    if !c.omissao.trim().is_empty() {
        s.push_str(&format!(" DEFAULT {}", literal_omissao(&c.omissao)));
    }
    if !c.check.trim().is_empty() {
        s.push_str(&format!(" CHECK ({})", c.check.trim()));
    }
    if !c.ref_tabela.trim().is_empty() {
        if c.ref_coluna.trim().is_empty() {
            s.push_str(&format!(" REFERENCES {}", quote(c.ref_tabela.trim())));
        } else {
            s.push_str(&format!(" REFERENCES {}({})", quote(c.ref_tabela.trim()), quote(c.ref_coluna.trim())));
        }
    }
    s
}

fn validar_campo(c: &Campo) -> Result<(), String> {
    if c.nome.trim().is_empty() {
        return Err("Todos os campos precisam de um nome".into());
    }
    if !TIPOS.contains(&c.tipo.as_str()) {
        return Err(format!("Tipo de dados desconhecido no campo «{}»", c.nome.trim()));
    }
    if c.autoincrement && !(c.pk && c.tipo.eq_ignore_ascii_case("INTEGER")) {
        return Err(format!(
            "A numeração automática do campo «{}» só é possível numa chave primária INTEGER",
            c.nome.trim()
        ));
    }
    if !c.check.trim().is_empty() && !parenteses_balanceados(c.check.trim()) {
        return Err(format!("A verificação do campo «{}» tem parênteses desfechados", c.nome.trim()));
    }
    Ok(())
}

/// Confirma que os parênteses de uma expressão CHECK estão fechados (ignora texto entre aspas).
fn parenteses_balanceados(s: &str) -> bool {
    let mut nivel = 0_i32;
    let mut aspas: Option<char> = None;
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if let Some(q) = aspas {
            if c == q {
                if it.peek() == Some(&q) {
                    it.next();
                } else {
                    aspas = None;
                }
            }
            continue;
        }
        match c {
            '\'' | '"' | '`' => aspas = Some(c),
            '(' => nivel += 1,
            ')' => {
                nivel -= 1;
                if nivel < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    nivel == 0 && aspas.is_none()
}

/// Ações ON UPDATE / ON DELETE das relações de uma tabela, por coluna de origem.
fn acoes_fk(conn: &Connection, tabela: &str) -> HashMap<String, (String, String)> {
    let mut out = HashMap::new();
    let Ok(mut stmt) = conn.prepare(&format!("PRAGMA foreign_key_list({})", quote(tabela))) else {
        return out;
    };
    let Ok(it) = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(3)?, // coluna de origem
            r.get::<_, String>(5)?, // on_update
            r.get::<_, String>(6)?, // on_delete
        ))
    }) else {
        return out;
    };
    for (coluna, upd, del) in it.flatten() {
        out.insert(coluna, (upd, del));
    }
    out
}

/// SQL dos índices e gatilhos da tabela: são apagados com o `DROP TABLE`.
fn objetos_da_tabela(conn: &Connection, tabela: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT sql FROM sqlite_master WHERE tbl_name = ?1 AND type IN ('index', 'trigger') AND sql IS NOT NULL ORDER BY type, name")
        .map_err(traduz)?;
    let objetos = stmt
        .query_map([tabela], |r| r.get::<_, String>(0))
        .map_err(traduz)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(traduz);
    objetos
}

impl Database {
    /// Abre um ficheiro existente. Não cria nada se o ficheiro não existir
    /// e recusa ficheiros que não sejam SQLite.
    pub(crate) fn open(path: &Path) -> Result<Self, String> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(traduz)?;
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0))
            .map_err(traduz)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;").map_err(traduz)?;
        Ok(Self { conn })
    }

    /// Cria um ficheiro novo e vazio. Recusa se já existir um ficheiro com esse nome.
    pub(crate) fn create(path: &Path) -> Result<Self, String> {
        if path.exists() {
            return Err(format!("Já existe um ficheiro com esse nome: {}", path.display()));
        }
        let conn = Connection::open(path).map_err(traduz)?;
        conn.execute_batch("PRAGMA user_version = 1; PRAGMA foreign_keys = ON;").map_err(traduz)?;
        Ok(Self { conn })
    }

    /// Base de dados em memória, usada nos testes unitários.
    #[cfg(test)]
    fn em_memoria() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(traduz)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;").map_err(traduz)?;
        Ok(Self { conn })
    }

    /// Grava uma cópia compacta da base de dados em `path` (que não pode existir).
    pub(crate) fn copy_to(&self, path: &Path) -> Result<(), String> {
        let p = path.to_string_lossy().replace('\'', "''");
        self.conn.execute_batch(&format!("VACUUM INTO '{}'", p)).map_err(traduz)
    }

    /// Verifica a integridade e compacta o ficheiro. Devolve o resultado da verificação.
    pub(crate) fn compact(&self) -> Result<String, String> {
        let r: String = self
            .conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .map_err(traduz)?;
        self.conn.execute_batch("VACUUM").map_err(traduz)?;
        Ok(r)
    }

    fn nomes(&self, tipo: &str) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master \
             WHERE type = ?1 AND name NOT LIKE 'sqlite_%' \
             ORDER BY name COLLATE NOCASE",
        )?;
        let it = stmt.query_map([tipo], |r| r.get::<_, String>(0))?;
        it.collect()
    }

    pub(crate) fn list_tables(&self) -> rusqlite::Result<Vec<String>> {
        self.nomes("table")
    }

    pub(crate) fn list_views(&self) -> rusqlite::Result<Vec<String>> {
        self.nomes("view")
    }

    /// Lê uma tabela ou consulta para memória. Em tabelas lê também o `rowid`
    /// (necessário para editar); se não houver `rowid` fica só de leitura.
    pub(crate) fn load(&self, name: &str, tabela: bool) -> rusqlite::Result<TableData> {
        self.load_filtrado(name, tabela, None, &[])
    }

    /// Como `load`, com ordenação (`coluna`, `ascendente`) e filtros opcionais.
    pub(crate) fn load_filtrado(
        &self,
        name: &str,
        tabela: bool,
        ordem: Option<(&str, bool)>,
        filtros: &[Condicao],
    ) -> rusqlite::Result<TableData> {
        if tabela {
            if let Ok(d) = self.load_inner(name, true, ordem, filtros) {
                return Ok(d);
            }
        }
        self.load_inner(name, false, ordem, filtros)
    }

    fn load_inner(
        &self,
        name: &str,
        com_rowid: bool,
        ordem: Option<(&str, bool)>,
        filtros: &[Condicao],
    ) -> rusqlite::Result<TableData> {
        let mut sql = if com_rowid {
            format!("SELECT rowid, * FROM {}", quote(name))
        } else {
            format!("SELECT * FROM {}", quote(name))
        };
        let mut params: Vec<String> = Vec::new();
        if !filtros.is_empty() {
            let conds: Vec<String> = filtros.iter().map(|f| f.sql(&mut params)).collect();
            sql.push_str(" WHERE ");
            sql.push_str(&conds.join(" AND "));
        }
        if let Some((col, asc)) = ordem {
            sql.push_str(&format!(" ORDER BY {} COLLATE NOCASE {}", quote(col), if asc { "ASC" } else { "DESC" }));
        }
        let mut stmt = self.conn.prepare(&sql)?;
        let mut dados = ler(&mut stmt, if com_rowid { 1 } else { 0 }, com_rowid, usize::MAX, &params)?;
        // Tipos e obrigatoriedade vêm do catálogo da tabela; em consultas e vistas
        // ficam vazios (essas grelhas não são editáveis).
        if com_rowid {
            if let Ok((campos, _)) = self.info_tabela(name) {
                let achar = |n: &str| campos.iter().find(|c| c.nome.eq_ignore_ascii_case(n));
                dados.tipos = dados.cols.iter().map(|(n, _)| achar(n).map(|c| c.tipo.clone()).unwrap_or_default()).collect();
                dados.obrig = dados.cols.iter().map(|(n, _)| achar(n).map(|c| c.obrigatorio).unwrap_or(false)).collect();
            }
        }
        Ok(dados)
    }

    /// Executa um script SQL (uma ou mais instruções). Mostra o resultado da última
    /// instrução que devolve linhas; as restantes só contam as linhas afetadas.
    pub(crate) fn executar(&self, sql: &str) -> Result<ResultadoSql, String> {
        let mut res = ResultadoSql { dados: None, afetadas: 0, instrucoes: 0 };
        if sql.to_ascii_lowercase().contains("trigger") {
            // os gatilhos têm ';' dentro do corpo: executar o script inteiro de uma vez
            self.conn.execute_batch(sql).map_err(traduz)?;
            res.instrucoes = 1;
            return Ok(res);
        }
        for (i, s) in dividir(sql).iter().enumerate() {
            let n = i + 1;
            let erro = |e: rusqlite::Error| format!("Instrução {}: {}", n, traduz(e));
            let mut stmt = self.conn.prepare(s).map_err(erro)?;
            if stmt.column_count() > 0 {
                res.dados = Some(ler(&mut stmt, 0, false, LIMITE_SQL, &[]).map_err(erro)?);
            } else {
                res.afetadas += stmt.execute([]).map_err(erro)?;
            }
            res.instrucoes = n;
        }
        Ok(res)
    }

    /// Texto `SELECT ...` de uma consulta guardada.
    pub(crate) fn view_sql(&self, nome: &str) -> Result<String, String> {
        let sql: String = self
            .conn
            .query_row("SELECT sql FROM sqlite_master WHERE type = 'view' AND name = ?1", [nome], |r| r.get(0))
            .map_err(traduz)?;
        Ok(corpo_vista(&sql))
    }

    /// Guarda uma consulta (`SELECT`) como `VIEW`. Com `substituir` troca uma existente.
    pub(crate) fn guardar_vista(&self, nome: &str, sql: &str, substituir: bool) -> Result<(), String> {
        let nome = nome.trim();
        if nome.is_empty() {
            return Err("Indique um nome para a consulta".into());
        }
        let corpo = sql.trim().trim_end_matches(';').trim();
        if corpo.is_empty() {
            return Err("A consulta está vazia".into());
        }
        if dividir(corpo).len() != 1 {
            return Err("Só é possível guardar uma instrução SELECT".into());
        }
        {
            let stmt = self.conn.prepare(corpo).map_err(traduz)?;
            let primeiro = corpo
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_start_matches('(')
                .to_ascii_uppercase();
            if (primeiro != "SELECT" && primeiro != "WITH") || stmt.column_count() == 0 {
                return Err("Só consultas SELECT podem ser guardadas como Consulta".into());
            }
        }
        let existe: i64 = self
            .conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name = ?1 COLLATE NOCASE AND type IN ('table', 'view')",
                [nome],
                |r| r.get(0),
            )
            .map_err(traduz)?;
        if existe > 0 && !substituir {
            return Err(format!("Já existe um objeto com o nome «{}»", nome));
        }
        let n = quote(nome);
        let script = format!("BEGIN;\nDROP VIEW IF EXISTS {n};\nCREATE VIEW {n} AS {corpo}\n;\nCOMMIT;");
        if let Err(e) = self.conn.execute_batch(&script) {
            let _ = self.conn.execute_batch("ROLLBACK;");
            return Err(traduz(e));
        }
        Ok(())
    }

    /// Lê uma única linha (para mostrar o valor tal como ficou gravado).
    pub(crate) fn fetch_row(&self, tabela: &str, rowid: i64) -> rusqlite::Result<Option<Vec<String>>> {
        let sql = format!("SELECT * FROM {} WHERE rowid = ?1", quote(tabela));
        let mut stmt = self.conn.prepare(&sql)?;
        let n = stmt.column_count();
        let mut rows = stmt.query([rowid])?;
        match rows.next()? {
            Some(r) => Ok(Some(linha(r, 0, n)?)),
            None => Ok(None),
        }
    }

    pub(crate) fn update_cell(&self, tabela: &str, col: &str, rowid: i64, texto: &str) -> rusqlite::Result<()> {
        let sql = format!("UPDATE {} SET {} = ?1 WHERE rowid = ?2", quote(tabela), quote(col));
        self.conn.execute(&sql, params![valor(texto), rowid])?;
        Ok(())
    }

    /// Insere uma linha nova só com um campo preenchido; devolve o `rowid` novo.
    pub(crate) fn insert_cell(&self, tabela: &str, col: &str, texto: &str) -> rusqlite::Result<i64> {
        let sql = format!("INSERT INTO {} ({}) VALUES (?1)", quote(tabela), quote(col));
        self.conn.execute(&sql, params![valor(texto)])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub(crate) fn delete_row(&self, tabela: &str, rowid: i64) -> rusqlite::Result<()> {
        let sql = format!("DELETE FROM {} WHERE rowid = ?1", quote(tabela));
        self.conn.execute(&sql, params![rowid])?;
        Ok(())
    }

    /// Campos de uma tabela e o texto `CREATE TABLE` original.
    pub(crate) fn info_tabela(&self, nome: &str) -> Result<(Vec<Campo>, String), String> {
        let mut stmt = self
            .conn
            .prepare(&format!("PRAGMA table_info({})", quote(nome)))
            .map_err(traduz)?;
        let campos = stmt
            .query_map([], |r| {
                Ok(Campo {
                    nome: r.get::<_, String>(1)?,
                    tipo: r.get::<_, String>(2)?,
                    obrigatorio: r.get::<_, i64>(3)? != 0,
                    omissao: r.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    pk: r.get::<_, i64>(5)? > 0,
                    ref_tabela: String::new(),
                    ref_coluna: String::new(),
                    unico: false,
                    check: String::new(),
                    autoincrement: false,
                })
            })
            .map_err(traduz)?
            .collect::<rusqlite::Result<Vec<Campo>>>()
            .map_err(traduz)?;
        let mut campos = campos;
        if let Ok(mut fk) = self.conn.prepare(&format!("PRAGMA foreign_key_list({})", quote(nome))) {
            if let Ok(it) = fk.query_map([], |r| {
                Ok((r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, Option<String>>(4)?))
            }) {
                for (tab, de, para) in it.flatten() {
                    if let Some(c) = campos.iter_mut().find(|c| c.nome == de) {
                        c.ref_tabela = tab;
                        c.ref_coluna = para.unwrap_or_default();
                    }
                }
            }
        }
        let sql: String = self
            .conn
            .query_row("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1", [nome], |r| r.get(0))
            .unwrap_or_default();
        aplicar_restricoes_inline(&mut campos, &sql);
        Ok((campos, sql))
    }

    /// Cria uma tabela nova a partir da Vista Estrutura, sem restrições de tabela.
    pub(crate) fn criar_tabela(&self, nome: &str, campos: &[Campo]) -> Result<(), String> {
        self.criar_tabela_com_restricoes(nome, campos, &[])
    }

    /// Como `criar_tabela`, com restrições de tabela `UNIQUE (...)` / `CHECK (...)`.
    pub(crate) fn criar_tabela_com_restricoes(
        &self,
        nome: &str,
        campos: &[Campo],
        restricoes: &[String],
    ) -> Result<(), String> {
        let nome = nome.trim();
        if nome.is_empty() {
            return Err("Indique um nome para a tabela".into());
        }
        if campos.is_empty() {
            return Err("A tabela precisa de pelo menos um campo".into());
        }
        let mut vistos: Vec<String> = Vec::new();
        for c in campos {
            validar_campo(c)?;
            let k = c.nome.trim().to_lowercase();
            if vistos.contains(&k) {
                return Err(format!("O campo «{}» está repetido", c.nome.trim()));
            }
            vistos.push(k);
        }
        let existe: i64 = self
            .conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name = ?1 COLLATE NOCASE AND type IN ('table', 'view')",
                [nome],
                |r| r.get(0),
            )
            .map_err(traduz)?;
        if existe > 0 {
            return Err(format!("Já existe um objeto com o nome «{}»", nome));
        }
        let pks: Vec<&Campo> = campos.iter().filter(|c| c.pk).collect();
        if pks.iter().any(|c| c.autoincrement) && pks.len() != 1 {
            return Err("A numeração automática só é possível com uma chave primária simples".into());
        }
        let inline = pks.len() == 1 && pks[0].tipo == "INTEGER";
        let mut defs: Vec<String> = campos.iter().map(|c| definicao(c, inline && c.pk)).collect();
        if !pks.is_empty() && !inline {
            let lista: Vec<String> = pks.iter().map(|c| quote(c.nome.trim())).collect();
            defs.push(format!("PRIMARY KEY ({})", lista.join(", ")));
        }
        for r in restricoes {
            let alvo = sem_constraint(r);
            if !(palavra_no_inicio(&alvo, "UNIQUE") || palavra_no_inicio(&alvo, "CHECK")) {
                return Err(format!("Restrição de tabela inválida: «{}»", r.trim()));
            }
            defs.push(r.trim().to_string());
        }
        let sql = format!("CREATE TABLE {} (\n  {}\n)", quote(nome), defs.join(",\n  "));
        self.conn.execute_batch(&sql).map_err(traduz)
    }

    pub(crate) fn acrescentar_campo(&self, tabela: &str, c: &Campo) -> Result<(), String> {
        validar_campo(c)?;
        if c.pk {
            return Err("Não é possível acrescentar uma chave primária a uma tabela existente".into());
        }
        if c.autoincrement {
            return Err("Não é possível acrescentar uma numeração automática a uma tabela existente".into());
        }
        if c.unico {
            return Err("Não é possível acrescentar um campo UNIQUE a uma tabela existente (crie um índice único)".into());
        }
        if c.obrigatorio && c.omissao.trim().is_empty() {
            return Err("Um campo obrigatório precisa de um valor por omissão ao ser acrescentado a uma tabela existente".into());
        }
        let sql = format!("ALTER TABLE {} ADD COLUMN {}", quote(tabela), definicao(c, false));
        self.conn.execute_batch(&sql).map_err(traduz)
    }

    pub(crate) fn renomear_campo(&self, tabela: &str, de: &str, para: &str) -> Result<(), String> {
        let para = para.trim();
        if para.is_empty() {
            return Err("Indique o novo nome do campo".into());
        }
        let sql = format!("ALTER TABLE {} RENAME COLUMN {} TO {}", quote(tabela), quote(de), quote(para));
        self.conn.execute_batch(&sql).map_err(traduz)
    }

    pub(crate) fn eliminar_campo(&self, tabela: &str, campo: &str) -> Result<(), String> {
        let sql = format!("ALTER TABLE {} DROP COLUMN {}", quote(tabela), quote(campo));
        self.conn.execute_batch(&sql).map_err(traduz)
    }

    /// Altera o tipo, a obrigatoriedade, o valor por omissão e as restrições
    /// (`UNIQUE`, `CHECK`, `AUTOINCREMENT`) de um campo existente.
    ///
    /// O SQLite não tem `ALTER COLUMN`: a tabela é recriada dentro de uma transação
    /// e os dados são copiados. Índices, gatilhos, chaves primárias, relações,
    /// restrições de tabela e vistas ficam intactos. Se qualquer passo falhar,
    /// a transação é anulada e a tabela original permanece.
    pub(crate) fn alterar_campo(&self, tabela: &str, nome_campo: &str, novo: &Campo) -> Result<(), String> {
        let (mut campos, sql) = self.info_tabela(tabela)?;
        let Some(i) = campos.iter().position(|c| c.nome.eq_ignore_ascii_case(nome_campo)) else {
            return Err(format!("O campo «{}» não existe na tabela «{}»", nome_campo, tabela));
        };
        let atual = campos[i].clone();
        let alterado = Campo {
            nome: atual.nome.clone(),
            tipo: novo.tipo.clone(),
            obrigatorio: novo.obrigatorio,
            omissao: novo.omissao.clone(),
            pk: atual.pk,
            ref_tabela: atual.ref_tabela.clone(),
            ref_coluna: atual.ref_coluna.clone(),
            unico: novo.unico,
            check: novo.check.clone(),
            autoincrement: novo.autoincrement,
        };
        validar_campo(&alterado)?;
        campos[i] = alterado;
        let pks: Vec<&Campo> = campos.iter().filter(|c| c.pk).collect();
        if pks.iter().any(|c| c.autoincrement) && pks.len() != 1 {
            return Err("A numeração automática só é possível com uma chave primária simples".into());
        }

        // As ações ON DELETE/ON UPDATE não estão em `Campo`: lê-se do catálogo
        // para não mudar o comportamento das relações ao recriar a tabela.
        let acoes = acoes_fk(&self.conn, tabela);
        let objetos = objetos_da_tabela(&self.conn, tabela)?;
        let restricoes = restricoes_tabela(&sql);
        let inline = pks.len() == 1 && pks[0].tipo.eq_ignore_ascii_case("INTEGER");
        let mut defs: Vec<String> = campos
            .iter()
            .map(|c| {
                let mut d = definicao(c, inline && c.pk);
                if !c.ref_tabela.trim().is_empty() {
                    if let Some((upd, del)) = acoes.get(&c.nome) {
                        if del != "NO ACTION" {
                            d.push_str(&format!(" ON DELETE {}", del));
                        }
                        if upd != "NO ACTION" {
                            d.push_str(&format!(" ON UPDATE {}", upd));
                        }
                    }
                }
                d
            })
            .collect();
        if !pks.is_empty() && !inline {
            let lista: Vec<String> = pks.iter().map(|c| quote(c.nome.trim())).collect();
            defs.push(format!("PRIMARY KEY ({})", lista.join(", ")));
        }
        defs.extend(restricoes);

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tmp = format!("__accessql_rebuild_{}", nanos);
        let colunas: Vec<String> = campos.iter().map(|c| quote(&c.nome)).collect();
        let lista = colunas.join(", ");
        let mut script = format!(
            "BEGIN IMMEDIATE;\nCREATE TABLE {} (\n  {}\n);\nINSERT INTO {} ({}) SELECT {} FROM {};\nDROP TABLE {};\nALTER TABLE {} RENAME TO {};\n",
            quote(&tmp),
            defs.join(",\n  "),
            quote(&tmp),
            lista,
            lista,
            quote(tabela),
            quote(tabela),
            quote(&tmp),
            quote(tabela),
        );
        for sql in objetos {
            script.push_str(&sql);
            script.push_str(";\n");
        }
        script.push_str("COMMIT;");

        // PRAGMA foreign_keys não pode mudar dentro de uma transação.
        let fk: i64 = self.conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0)).unwrap_or(0);
        if fk != 0 {
            let _ = self.conn.execute_batch("PRAGMA foreign_keys = OFF;");
        }
        let resultado = self.conn.execute_batch(&script).map_err(traduz);
        if resultado.is_err() {
            let _ = self.conn.execute_batch("ROLLBACK;");
        }
        if fk != 0 {
            let _ = self.conn.execute_batch("PRAGMA foreign_keys = ON;");
        }
        resultado
    }

    /// Muda o nome de uma tabela (`ALTER TABLE ... RENAME TO`) ou de uma consulta
    /// (o SQLite não tem `ALTER VIEW`: recria-se a `VIEW` com o novo nome).
    pub(crate) fn renomear_objeto(&self, tabela: bool, de: &str, para: &str) -> Result<(), String> {
        let para = para.trim();
        if para.is_empty() {
            return Err("Indique o novo nome".into());
        }
        if !para.eq_ignore_ascii_case(de) {
            let existe: i64 = self
                .conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE name = ?1 COLLATE NOCASE AND type IN ('table', 'view')",
                    [para],
                    |r| r.get(0),
                )
                .map_err(traduz)?;
            if existe > 0 {
                return Err(format!("Já existe um objeto com o nome «{}»", para));
            }
        }
        if tabela {
            let sql = format!("ALTER TABLE {} RENAME TO {}", quote(de), quote(para));
            self.conn.execute_batch(&sql).map_err(traduz)
        } else {
            let corpo = self.view_sql(de)?;
            let script = format!(
                "BEGIN;\nDROP VIEW {};\nCREATE VIEW {} AS {}\n;\nCOMMIT;",
                quote(de),
                quote(para),
                corpo
            );
            if let Err(e) = self.conn.execute_batch(&script) {
                let _ = self.conn.execute_batch("ROLLBACK;");
                return Err(traduz(e));
            }
            Ok(())
        }
    }

    /// Insere linhas numa tabela, numa só transação (tudo ou nada).
    /// `mapa`: (coluna do ficheiro, campo de destino, campo numérico).
    pub(crate) fn inserir_linhas(
        &self,
        tabela: &str,
        mapa: &[(usize, String, bool)],
        linhas: &[Vec<String>],
        virgula: bool,
    ) -> Result<usize, String> {
        self.inserir_linhas_ids(tabela, mapa, linhas, virgula).map(|v| v.len())
    }

    /// Como `inserir_linhas`, mas devolve o `rowid` de cada registo criado.
    pub(crate) fn inserir_linhas_ids(
        &self,
        tabela: &str,
        mapa: &[(usize, String, bool)],
        linhas: &[Vec<String>],
        virgula: bool,
    ) -> Result<Vec<i64>, String> {
        if mapa.is_empty() {
            return Err("Nenhuma coluna do ficheiro corresponde a campos da tabela".into());
        }
        let cols: Vec<String> = mapa.iter().map(|m| quote(&m.1)).collect();
        let marcas = vec!["?"; mapa.len()].join(", ");
        let sql = format!("INSERT INTO {} ({}) VALUES ({})", quote(tabela), cols.join(", "), marcas);
        let tx = self.conn.unchecked_transaction().map_err(traduz)?;
        let mut ids: Vec<i64> = Vec::new();
        {
            let mut stmt = tx.prepare(&sql).map_err(traduz)?;
            for (k, l) in linhas.iter().enumerate() {
                let vals: Vec<Value> = mapa
                    .iter()
                    .map(|(ci, _, num)| {
                        let v = l.get(*ci).map(|s| s.as_str()).unwrap_or("");
                        if v.trim().is_empty() {
                            Value::Null
                        } else if *num {
                            let t = v.trim();
                            Value::Text(if virgula { t.replace(',', ".") } else { t.to_string() })
                        } else {
                            Value::Text(v.to_string())
                        }
                    })
                    .collect();
                stmt.execute(params_from_iter(vals.iter()))
                    .map_err(|e| format!("Linha {} de dados: {}", k + 1, traduz(e)))?;
                ids.push(tx.last_insert_rowid());
            }
        }
        tx.commit().map_err(traduz)?;
        Ok(ids)
    }

    pub(crate) fn eliminar_tabela(&self, nome: &str) -> Result<(), String> {
        self.conn.execute_batch(&format!("DROP TABLE {}", quote(nome))).map_err(traduz)
    }

    /// Valor guardado numa célula, tal como está na base de dados (para desfazer).
    pub(crate) fn valor_bruto(&self, tabela: &str, col: &str, rowid: i64) -> Result<Value, String> {
        self.conn
            .query_row(
                &format!("SELECT {} FROM {} WHERE rowid = ?1", quote(col), quote(tabela)),
                [rowid],
                |r| r.get::<_, Value>(0),
            )
            .map_err(traduz)
    }

    /// Registo completo (nomes e valores) tal como está na base de dados.
    pub(crate) fn registo_bruto(&self, tabela: &str, rowid: i64) -> Result<(Vec<String>, Vec<Value>), String> {
        let mut stmt = self
            .conn
            .prepare(&format!("SELECT * FROM {} WHERE rowid = ?1", quote(tabela)))
            .map_err(traduz)?;
        let nomes: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();
        let mut rows = stmt.query([rowid]).map_err(traduz)?;
        match rows.next().map_err(traduz)? {
            Some(r) => {
                let mut v = Vec::with_capacity(nomes.len());
                for i in 0..nomes.len() {
                    v.push(r.get::<_, Value>(i).map_err(traduz)?);
                }
                Ok((nomes, v))
            }
            None => Err("Registo não encontrado".into()),
        }
    }

    pub(crate) fn definir_celula(&self, tabela: &str, col: &str, rowid: i64, v: &Value) -> Result<(), String> {
        let sql = format!("UPDATE {} SET {} = ?1 WHERE rowid = ?2", quote(tabela), quote(col));
        self.conn.execute(&sql, params![v, rowid]).map_err(traduz)?;
        Ok(())
    }

    /// Repõe um registo com o mesmo `rowid` (desfazer uma eliminação).
    pub(crate) fn inserir_bruto(&self, tabela: &str, rowid: i64, colunas: &[String], valores: &[Value]) -> Result<(), String> {
        // se o campo de chave é INTEGER PRIMARY KEY já faz parte das colunas e dá o rowid
        let alias = self.info_tabela(tabela).ok().map_or(false, |(campos, _)| {
            let pks: Vec<&Campo> = campos.iter().filter(|c| c.pk).collect();
            pks.len() == 1 && pks[0].tipo.eq_ignore_ascii_case("INTEGER") && colunas.iter().any(|c| *c == pks[0].nome)
        });
        let mut nomes: Vec<String> = colunas.iter().map(|c| quote(c)).collect();
        let mut todos: Vec<Value> = valores.to_vec();
        if !alias {
            nomes.insert(0, "rowid".to_string());
            todos.insert(0, Value::Integer(rowid));
        }
        let marcas = vec!["?"; todos.len()].join(", ");
        let sql = format!("INSERT INTO {} ({}) VALUES ({})", quote(tabela), nomes.join(", "), marcas);
        self.conn.execute(&sql, params_from_iter(todos.iter())).map_err(traduz)?;
        Ok(())
    }

    pub(crate) fn lista_indices(&self, tabela: &str) -> Result<Vec<Indice>, String> {
        let mut stmt = self
            .conn
            .prepare(&format!("PRAGMA index_list({})", quote(tabela)))
            .map_err(traduz)?;
        let base: Vec<(String, bool, String)> = stmt
            .query_map([], |r| Ok((r.get::<_, String>(1)?, r.get::<_, i64>(2)? != 0, r.get::<_, String>(3)?)))
            .map_err(traduz)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(traduz)?;
        let mut out = Vec::new();
        for (nome, unico, origem) in base {
            let mut st = self
                .conn
                .prepare(&format!("PRAGMA index_info({})", quote(&nome)))
                .map_err(traduz)?;
            let colunas: Vec<String> = st
                .query_map([], |r| r.get::<_, Option<String>>(2))
                .map_err(traduz)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(traduz)?
                .into_iter()
                .map(|c| c.unwrap_or_else(|| "(expressão)".to_string()))
                .collect();
            out.push(Indice { nome, unico, origem, colunas });
        }
        Ok(out)
    }

    pub(crate) fn criar_indice(&self, nome: &str, tabela: &str, colunas: &[String], unico: bool) -> Result<(), String> {
        if colunas.is_empty() {
            return Err("Escolha pelo menos um campo para o índice".into());
        }
        let nome = nome.trim();
        if nome.is_empty() {
            return Err("Indique um nome para o índice".into());
        }
        let existe: i64 = self
            .conn
            .query_row("SELECT count(*) FROM sqlite_master WHERE name = ?1 COLLATE NOCASE", [nome], |r| r.get(0))
            .map_err(traduz)?;
        if existe > 0 {
            return Err(format!("Já existe um objeto com o nome «{}»", nome));
        }
        let lista: Vec<String> = colunas.iter().map(|c| quote(c)).collect();
        let sql = format!(
            "CREATE {}INDEX {} ON {} ({})",
            if unico { "UNIQUE " } else { "" },
            quote(nome),
            quote(tabela),
            lista.join(", ")
        );
        self.conn.execute_batch(&sql).map_err(traduz)
    }

    pub(crate) fn eliminar_indice(&self, nome: &str) -> Result<(), String> {
        self.conn.execute_batch(&format!("DROP INDEX {}", quote(nome))).map_err(traduz)
    }

    pub(crate) fn foreign_keys_ativas(&self) -> bool {
        self.conn.query_row("PRAGMA foreign_keys", [], |r| r.get::<_, i64>(0)).map_or(false, |v| v != 0)
    }

    pub(crate) fn definir_foreign_keys(&self, ligar: bool) -> Result<(), String> {
        self.conn
            .execute_batch(if ligar { "PRAGMA foreign_keys = ON;" } else { "PRAGMA foreign_keys = OFF;" })
            .map_err(traduz)
    }

    /// Todas as chaves estrangeiras da base de dados (as compostas aparecem numa só linha).
    pub(crate) fn lista_relacoes(&self) -> Result<Vec<Relacao>, String> {
        let mut out: Vec<Relacao> = Vec::new();
        for tabela in self.list_tables().map_err(traduz)? {
            let mut stmt = self
                .conn
                .prepare(&format!("PRAGMA foreign_key_list({})", quote(&tabela)))
                .map_err(traduz)?;
            let linhas: Vec<(i64, String, String, Option<String>, String, String)> = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, Option<String>>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, String>(6)?,
                    ))
                })
                .map_err(traduz)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(traduz)?;
            let mut ids: Vec<i64> = Vec::new();
            let mut parcial: Vec<Relacao> = Vec::new();
            for (id, ref_t, de, para, on_upd, on_del) in linhas {
                let para = para.unwrap_or_else(|| "(chave primária)".to_string());
                if let Some(p) = ids.iter().position(|x| *x == id) {
                    parcial[p].colunas.push_str(&format!(", {}", de));
                    parcial[p].ref_colunas.push_str(&format!(", {}", para));
                } else {
                    ids.push(id);
                    parcial.push(Relacao {
                        tabela: tabela.clone(),
                        colunas: de,
                        ref_tabela: ref_t,
                        ref_colunas: para,
                        ao_apagar: on_del,
                        ao_atualizar: on_upd,
                    });
                }
            }
            out.extend(parcial);
        }
        Ok(out)
    }

    /// Registos que referem valores inexistentes (`PRAGMA foreign_key_check`).
    pub(crate) fn verificar_relacoes(&self) -> Result<Vec<String>, String> {
        let mut stmt = self.conn.prepare("PRAGMA foreign_key_check").map_err(traduz)?;
        let v: Vec<String> = stmt
            .query_map([], |r| {
                Ok(format!(
                    "{}: o registo {} refere um valor que não existe em {}",
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<i64>>(1)?.map_or("?".to_string(), |n| n.to_string()),
                    r.get::<_, String>(2)?
                ))
            })
            .map_err(traduz)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(traduz)?;
        Ok(v.into_iter().take(200).collect())
    }

    /// Campos a que se pode ligar uma chave estrangeira: as chaves primárias simples.
    pub(crate) fn opcoes_fk(&self) -> Result<Vec<(String, String)>, String> {
        let mut out = Vec::new();
        for t in self.list_tables().map_err(traduz)? {
            if let Ok((campos, _)) = self.info_tabela(&t) {
                let pks: Vec<&Campo> = campos.iter().filter(|c| c.pk).collect();
                if pks.len() == 1 {
                    out.push((t.clone(), pks[0].nome.clone()));
                }
            }
        }
        Ok(out)
    }

    pub(crate) fn contar_registos(&self, tabela: &str) -> Result<i64, String> {
        self.conn
            .query_row(&format!("SELECT count(*) FROM {}", quote(tabela)), [], |r| r.get::<_, i64>(0))
            .map_err(traduz)
    }

    /// Consultas guardadas cujo texto menciona `nome` (deixam de funcionar se ele desaparecer).
    pub(crate) fn consultas_que_mencionam(&self, nome: &str) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, sql FROM sqlite_master WHERE type = 'view' AND name <> ?1 COLLATE NOCASE ORDER BY name COLLATE NOCASE")
            .map_err(traduz)?;
        let it = stmt
            .query_map([nome], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default())))
            .map_err(traduz)?;
        Ok(it.flatten().filter(|(_, sql)| menciona(sql, nome)).map(|(n, _)| n).collect())
    }

    /// Elimina uma tabela (com os seus índices) ou uma consulta guardada.
    pub(crate) fn eliminar_objeto(&self, tabela: bool, nome: &str) -> Result<(), String> {
        let sql = format!("DROP {} {}", if tabela { "TABLE" } else { "VIEW" }, quote(nome));
        self.conn.execute_batch(&sql).map_err(traduz)
    }
}

/// `true` se `sql` contém `nome` como palavra inteira (sem letras, dígitos ou `_` ao lado).
fn menciona(sql: &str, nome: &str) -> bool {
    let s = sql.to_lowercase();
    let n = nome.to_lowercase();
    if n.is_empty() {
        return false;
    }
    let mut desde = 0;
    while let Some(p) = s[desde..].find(&n) {
        let ini = desde + p;
        let fim = ini + n.len();
        let antes_ok = s[..ini].chars().next_back().map_or(true, |c| !(c.is_alphanumeric() || c == '_'));
        let depois_ok = s[fim..].chars().next().map_or(true, |c| !(c.is_alphanumeric() || c == '_'));
        if antes_ok && depois_ok {
            return true;
        }
        desde = fim;
    }
    false
}
