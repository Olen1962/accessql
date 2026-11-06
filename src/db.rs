//! Camada de dados do AccesSQL (SQLite local, via rusqlite).

use rusqlite::{params, params_from_iter, types::Value, types::ValueRef, Connection, OpenFlags, Row, Statement};
use std::path::Path;

/// Conteúdo de uma tabela ou consulta já carregado em memória.
#[derive(Default)]
pub(crate) struct TableData {
    /// Cada coluna: (nome, largura em píxeis na folha de dados).
    pub(crate) cols: Vec<(String, f32)>,
    pub(crate) rows: Vec<Vec<String>>,
    /// `rowid` de cada linha (só em tabelas editáveis; vazio em consultas).
    pub(crate) rowids: Vec<i64>,
    /// `true` se for uma tabela com `rowid` (permite editar, inserir e eliminar).
    pub(crate) editable: bool,
    /// `true` se o resultado foi cortado por ter demasiadas linhas.
    pub(crate) truncado: bool,
}

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
fn traduz(e: rusqlite::Error) -> String {
    let s = e.to_string();
    if s.contains("not a database") || s.contains("encrypted") {
        "O ficheiro não é uma base de dados SQLite válida".into()
    } else if s.contains("unable to open") {
        "Não foi possível abrir o ficheiro (confirme o caminho e as permissões)".into()
    } else {
        s
    }
}

/// Texto vazio grava-se como NULL (como no Access); o resto como texto.
/// O SQLite converte para número se a coluna tiver afinidade numérica.
fn valor(t: &str) -> Value {
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

/// Lê o resultado de uma instrução já preparada (ignora as primeiras `skip` colunas).
fn ler(stmt: &mut Statement, skip: usize, com_rowid: bool, limite: usize) -> rusqlite::Result<TableData> {
    let total = stmt.column_count();
    let nomes: Vec<String> = stmt.column_names().into_iter().skip(skip).map(String::from).collect();
    let mut rows_out: Vec<Vec<String>> = Vec::new();
    let mut rowids: Vec<i64> = Vec::new();
    let mut truncado = false;
    let mut rows = stmt.query([])?;
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
    Ok(TableData { cols, rows: rows_out, rowids, editable: com_rowid, truncado })
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
        Ok(Self { conn })
    }

    /// Cria um ficheiro novo e vazio. Recusa se já existir um ficheiro com esse nome.
    pub(crate) fn create(path: &Path) -> Result<Self, String> {
        if path.exists() {
            return Err(format!("Já existe um ficheiro com esse nome: {}", path.display()));
        }
        let conn = Connection::open(path).map_err(traduz)?;
        conn.execute_batch("PRAGMA user_version = 1;").map_err(traduz)?;
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
        if tabela {
            if let Ok(d) = self.load_inner(name, true) {
                return Ok(d);
            }
        }
        self.load_inner(name, false)
    }

    fn load_inner(&self, name: &str, com_rowid: bool) -> rusqlite::Result<TableData> {
        let sql = if com_rowid {
            format!("SELECT rowid, * FROM {}", quote(name))
        } else {
            format!("SELECT * FROM {}", quote(name))
        };
        let mut stmt = self.conn.prepare(&sql)?;
        ler(&mut stmt, if com_rowid { 1 } else { 0 }, com_rowid, usize::MAX)
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
                res.dados = Some(ler(&mut stmt, 0, false, LIMITE_SQL).map_err(erro)?);
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
            if stmt.column_count() == 0 {
                return Err("Só consultas que devolvem dados (SELECT) podem ser guardadas".into());
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
}
