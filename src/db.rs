//! Camada de dados do AccesSQL (SQLite local, via rusqlite).

use rusqlite::{params_from_iter, types::ValueRef, Connection};
use std::path::Path;

/// Conteúdo de uma tabela ou consulta já carregado em memória.
/// Cada coluna: (nome, largura em píxeis para a folha de dados).
pub(crate) struct TableData {
    pub(crate) cols: Vec<(String, f32)>,
    pub(crate) rows: Vec<Vec<String>>,
}

pub(crate) struct Database {
    conn: Connection,
}

/// Escapa um identificador SQL: "nome" (aspas duplicadas dentro do nome).
pub(crate) fn quote(id: &str) -> String {
    format!("\"{}\"", id.replace('"', "\"\""))
}

impl Database {
    /// Abre (ou cria, se não existir) um ficheiro de base de dados.
    #[allow(dead_code)] // usado no passo 3 (Ficheiro → Novo/Abrir)
    pub(crate) fn open(path: &Path) -> rusqlite::Result<Self> {
        Ok(Self { conn: Connection::open(path)? })
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
                    let tipo = if *c == "ID" || *c == "Stock" { "INTEGER" } else { "TEXT" };
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

    /// Lê uma tabela ou consulta (vista) completa para memória.
    pub(crate) fn load(&self, name: &str) -> rusqlite::Result<TableData> {
        let sql = format!("SELECT * FROM {}", quote(name));
        let mut stmt = self.conn.prepare(&sql)?;
        let nomes: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();
        let n = nomes.len();
        let mut rows_out: Vec<Vec<String>> = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            let mut v = Vec::with_capacity(n);
            for i in 0..n {
                v.push(match r.get_ref(i)? {
                    ValueRef::Null => String::new(),
                    ValueRef::Integer(x) => x.to_string(),
                    ValueRef::Real(x) => x.to_string(),
                    ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
                    ValueRef::Blob(_) => "[BLOB]".to_string(),
                });
            }
            rows_out.push(v);
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
        Ok(TableData { cols, rows: rows_out })
    }
}
