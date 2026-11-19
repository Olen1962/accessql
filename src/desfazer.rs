//! Desfazer e refazer alterações aos dados (células, registos criados e eliminados).

use crate::{db::Database, estado::*};
use rusqlite::types::Value;

/// Uma alteração feita aos dados, com o que é preciso para a desfazer e refazer.
pub(crate) enum Acao {
    /// Célula alterada. `rowid_depois` só difere de `rowid` se se mudou o campo de chave.
    Celula { tabela: String, rowid: i64, rowid_depois: i64, col: String, antigo: Value, novo: Value },
    /// Registo criado (guarda-se o conteúdo para o poder refazer).
    Inserido { tabela: String, rowid: i64, colunas: Vec<String>, valores: Vec<Value> },
    /// Registo eliminado (guarda-se o conteúdo para o poder repor).
    Apagado { tabela: String, rowid: i64, colunas: Vec<String>, valores: Vec<Value> },
}

impl Acao {
    pub(crate) fn tabela(&self) -> &str {
        match self {
            Acao::Celula { tabela, .. } | Acao::Inserido { tabela, .. } | Acao::Apagado { tabela, .. } => tabela,
        }
    }

    fn descricao(&self) -> &'static str {
        match self {
            Acao::Celula { .. } => "alteração de uma célula",
            Acao::Inserido { .. } => "criação de um registo",
            Acao::Apagado { .. } => "eliminação de um registo",
        }
    }

    fn desfazer(&self, db: &Database) -> Result<(), String> {
        match self {
            Acao::Celula { tabela, rowid_depois, col, antigo, .. } => db.definir_celula(tabela, col, *rowid_depois, antigo),
            Acao::Inserido { tabela, rowid, .. } => db.delete_row(tabela, *rowid).map_err(|e| e.to_string()),
            Acao::Apagado { tabela, rowid, colunas, valores } => db.inserir_bruto(tabela, *rowid, colunas, valores),
        }
    }

    fn refazer(&self, db: &Database) -> Result<(), String> {
        match self {
            Acao::Celula { tabela, rowid, col, novo, .. } => db.definir_celula(tabela, col, *rowid, novo),
            Acao::Inserido { tabela, rowid, colunas, valores } => db.inserir_bruto(tabela, *rowid, colunas, valores),
            Acao::Apagado { tabela, rowid, .. } => db.delete_row(tabela, *rowid).map_err(|e| e.to_string()),
        }
    }
}

impl AccessApp {
    /// Fecha o passo em curso: tudo o que se gravou desde o último passo passa a ser
    /// uma única entrada do histórico (uma colagem grande desfaz-se de uma vez).
    pub(crate) fn fechar_passo(&mut self) {
        if self.hist_pend.is_empty() {
            return;
        }
        let p = std::mem::take(&mut self.hist_pend);
        self.undo.push(p);
        self.redo.clear();
        if self.undo.len() > 200 {
            self.undo.remove(0);
        }
    }

    /// Esquece o histórico (a estrutura ou a base de dados mudaram por outra via).
    pub(crate) fn limpar_historico(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.hist_pend.clear();
    }

    pub(crate) fn desfazer(&mut self) {
        self.passo(true);
    }

    pub(crate) fn refazer(&mut self) {
        self.passo(false);
    }

    fn passo(&mut self, desfazer: bool) {
        self.fechar_passo();
        let passo = if desfazer { self.undo.pop() } else { self.redo.pop() };
        let Some(passo) = passo else {
            self.msg = if desfazer { "Não há nada para desfazer" } else { "Não há nada para refazer" }.into();
            return;
        };
        let mut erro: Option<String> = None;
        let mut tabelas: Vec<String> = Vec::new();
        if let Some(db) = self.db.as_ref() {
            let it: Box<dyn Iterator<Item = &Acao> + '_> = if desfazer { Box::new(passo.iter().rev()) } else { Box::new(passo.iter()) };
            for a in it {
                let r = if desfazer { a.desfazer(db) } else { a.refazer(db) };
                if let Err(e) = r {
                    erro = Some(e);
                    break;
                }
                tabelas.push(a.tabela().to_string());
            }
        }
        match erro {
            None => {
                let desc = if passo.len() == 1 { passo[0].descricao().to_string() } else { format!("{} alterações", passo.len()) };
                self.msg = format!("{}: {}", if desfazer { "Desfeito" } else { "Refeito" }, desc);
                if desfazer {
                    self.redo.push(passo);
                } else {
                    self.undo.push(passo);
                }
            }
            Some(e) => {
                self.msg = format!(
                    "Não foi possível {}: {} (o histórico foi limpo)",
                    if desfazer { "desfazer" } else { "refazer" },
                    e
                );
                self.limpar_historico();
            }
        }
        let idx: Vec<usize> = self
            .docs
            .iter()
            .enumerate()
            .filter(|(_, d)| d.kind == ObjKind::Table && tabelas.iter().any(|t| *t == d.name))
            .map(|(i, _)| i)
            .collect();
        for i in idx {
            self.carregar_doc(i);
        }
        self.edit = None;
    }
}
