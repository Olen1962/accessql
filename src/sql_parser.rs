//! Parser SQL -> QBE. Placeholder.
use crate::db::Database;
use crate::qbe::{ParsedQbe, QbeTable, QbeJoin, GridRow, QbeField};

pub(crate) fn parse_sql_to_qbe(_sql: &str, _db: &Database) -> Result<ParsedQbe, String> {
    Err("Abrir consulta existente em Vista Estrutura (QBE) ainda não suportado. Use a Vista SQL para editar.".into())
}