//! Camada de dados do AccesSQL (SQLite local, via rusqlite).

use rusqlite::{params, params_from_iter, types::Value, types::ValueRef, Connection, OpenFlags, Row, Statement};
use std::path::Path;

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

    #[test]
    fn metadados_da_tabela_chegam_a_folha_de_dados() {
        let db = Database::demo().unwrap();
        let (campos, _) = db.info_tabela("Clientes").unwrap();
        let id = campos.iter().find(|c| c.nome == "ID").unwrap();
        assert!(id.tipo.contains("INT"), "tipo de ID: {}", id.tipo);
        let dados = db.load("Clientes", true).unwrap();
        assert_eq!(dados.tipos.len(), dados.cols.len());
        assert_eq!(dados.obrig.len(), dados.cols.len());
        let i = dados.cols.iter().position(|c| c.0 == "ID").unwrap();
        assert!(dados.tipos[i].contains("INT"));
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

/// Definição SQL de uma coluna: `"nome" TIPO [PRIMARY KEY] [NOT NULL] [DEFAULT x]`.
fn definicao(c: &Campo, pk_inline: bool) -> String {
    let mut s = format!("{} {}", quote(c.nome.trim()), c.tipo);
    if pk_inline {
        s.push_str(" PRIMARY KEY");
    }
    if c.obrigatorio {
        s.push_str(" NOT NULL");
    }
    if !c.omissao.trim().is_empty() {
        s.push_str(&format!(" DEFAULT {}", literal_omissao(&c.omissao)));
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
    Ok(())
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

    /// Base de dados em memória com os dados do modelo visual.
    /// Temporária: serve só até Ficheiro → Novo/Abrir estar ligado.
    pub(crate) fn demo() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        for nome in ["Clientes", "Encomendas", "Produtos"] {
            let (cols, rows) = crate::dados_exemplo::dataset(nome);
            let defs: Vec<String> = cols
                .iter()
                .map(|(c, _)| {
                    let tipo = match *c {
                        "ID" => "INTEGER PRIMARY KEY",
                        "Stock" => "INTEGER",
                        _ => "TEXT",
                    };
                    format!("{} {}", quote(c), tipo)
                })
                .collect();
            conn.execute(&format!("CREATE TABLE {} ({})", quote(nome), defs.join(", ")), [])?;
            let marcas = vec!["?"; cols.len()].join(", ");
            let ins = format!("INSERT INTO {} VALUES ({})", quote(nome), marcas);
            for r in &rows {
                conn.execute(&ins, params_from_iter(r.iter().copied()))?;
            }
        }
        conn.execute_batch(
            "CREATE VIEW \"Encomendas por Cliente\" AS \
                 SELECT Cliente, COUNT(*) AS Encomendas FROM Encomendas \
                 GROUP BY Cliente ORDER BY Cliente;
             CREATE VIEW \"Produtos em Falta\" AS \
                 SELECT * FROM Produtos WHERE Stock < 20;",
        )?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(Self { conn })
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
        Ok((campos, sql))
    }

    /// Cria uma tabela nova a partir da Vista Estrutura.
    pub(crate) fn criar_tabela(&self, nome: &str, campos: &[Campo]) -> Result<(), String> {
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
        let inline = pks.len() == 1 && pks[0].tipo == "INTEGER";
        let mut defs: Vec<String> = campos.iter().map(|c| definicao(c, inline && c.pk)).collect();
        if !pks.is_empty() && !inline {
            let lista: Vec<String> = pks.iter().map(|c| quote(c.nome.trim())).collect();
            defs.push(format!("PRIMARY KEY ({})", lista.join(", ")));
        }
        let sql = format!("CREATE TABLE {} (\n  {}\n)", quote(nome), defs.join(",\n  "));
        self.conn.execute_batch(&sql).map_err(traduz)
    }

    pub(crate) fn acrescentar_campo(&self, tabela: &str, c: &Campo) -> Result<(), String> {
        validar_campo(c)?;
        if c.pk {
            return Err("Não é possível acrescentar uma chave primária a uma tabela existente".into());
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
