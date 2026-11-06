//! Camada de dados do AccesSQL (SQLite local, via rusqlite).

use rusqlite::{params, params_from_iter, types::Value, types::ValueRef, Connection, OpenFlags, Row};
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
        let skip = if com_rowid { 1 } else { 0 };
        let mut stmt = self.conn.prepare(&sql)?;
        let total = stmt.column_count();
        let nomes: Vec<String> = stmt.column_names().into_iter().skip(skip).map(String::from).collect();
        let mut rows_out: Vec<Vec<String>> = Vec::new();
        let mut rowids: Vec<i64> = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
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
        Ok(TableData { cols, rows: rows_out, rowids, editable: com_rowid })
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
