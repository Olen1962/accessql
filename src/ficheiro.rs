//! Operações de ficheiro: novo, abrir, guardar cópia, fechar, compactar.

use crate::{db::Database, estado::*, recentes};
use std::path::{Path, PathBuf};

pub(crate) fn pasta_inicial() -> String {
    std::env::var("HOME").unwrap_or_else(|_| ".".into())
}

fn nome_de(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

impl AccessApp {
    /// Passa a trabalhar com a base de dados `db` (fecha a anterior).
    fn instalar_bd(&mut self, db: Database, ficheiro: Option<PathBuf>, nome: String) {
        self.db = Some(db);
        self.db_file = ficheiro.clone();
        self.db_path = nome;
        self.docs.clear();
        self.active = 0;
        self.sel = (0, 1);
        self.edit = None;
        self.confirm_del = None;
        self.pedir_nome = None;
        self.n_consulta = 0;
        self.n_tabela = 0;
        self.limpar_historico();
        self.totais_aberto = false;
        self.indices = None;
        self.relacoes = None;
        self.eliminar_obj = None;
        self.renomear_obj = None;
        self.importar = None;
        self.exportar_csv = None;
        self.foco_nav = false;
        self.refresh_nav();
        if let Some(f) = ficheiro {
            recentes::adicionar(&mut self.recentes, f);
            recentes::guardar(&self.recentes);
        }
        self.backstage = false;
    }

    pub(crate) fn dialogo_abrir(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Bases de dados AccesSQL (accsql, sqlite, db)",
                &["accsql", "sqlite", "sqlite3", "db"],
            )
            .pick_file()
        {
            self.abrir_ficheiro(path);
        }
    }

    pub(crate) fn abrir_ficheiro(&mut self, path: PathBuf) {
        if !path.exists() {
            self.msg = format!("O ficheiro já não existe: {}", path.display());
            self.recentes.retain(|p| p != &path);
            recentes::guardar(&self.recentes);
            return;
        }
        match Database::open(&path) {
            Ok(db) => {
                let nome = nome_de(&path);
                self.instalar_bd(db, Some(path.clone()), nome);
                self.msg = format!("Aberta: {}", path.display());
            }
            Err(e) => self.msg = e,
        }
    }

    /// Cria um ficheiro novo com o nome e a pasta indicados no separador «Novo».
    pub(crate) fn criar_ficheiro(&mut self) {
        let mut nome = self.new_db_name.trim().to_string();
        if nome.is_empty() {
            self.msg = "Indique um nome para o ficheiro".into();
            return;
        }
        if Path::new(&nome).extension().is_none() {
            nome.push_str(".accsql");
        }
        let path = PathBuf::from(self.new_db_dir.trim()).join(&nome);
        match Database::create(&path) {
            Ok(db) => {
                let n = nome_de(&path);
                self.instalar_bd(db, Some(path.clone()), n);
                self.msg = format!("Criada: {}", path.display());
            }
            Err(e) => self.msg = e,
        }
    }

    /// Guarda uma cópia da base de dados aberta noutro ficheiro (a atual continua aberta).
    pub(crate) fn guardar_copia(&mut self) {
        let Some(db) = self.db.as_ref() else {
            self.msg = "Nenhuma base de dados aberta".into();
            return;
        };
        let inicial = format!("{}.accsql", self.db_path);
        let Some(mut path) = rfd::FileDialog::new()
            .set_file_name(inicial)
            .add_filter("Base de dados AccesSQL (accsql)", &["accsql"])
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension("accsql");
        }
        if self.db_file.as_ref() == Some(&path) {
            self.msg = "Escolha um ficheiro diferente do atual".into();
            return;
        }
        // Gravar primeiro num ficheiro temporário: se a cópia falhar, o ficheiro
        // de destino original fica intacto (a caixa de diálogo já confirmou a substituição).
        let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        let nome_tmp = format!(
            ".{}.{}-{}.tmp",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("copia"),
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let tmp = dir.join(nome_tmp);
        match db.copy_to(&tmp) {
            Ok(()) => match std::fs::rename(&tmp, &path) {
                Ok(()) => self.msg = format!("Cópia guardada em {}", path.display()),
                Err(e) => {
                    let _ = std::fs::remove_file(&tmp);
                    self.msg = format!("Não foi possível guardar a cópia: {}", e);
                }
            },
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                self.msg = e;
            }
        }
    }

    pub(crate) fn fechar_bd(&mut self) {
        if self.db.is_none() {
            self.msg = "Nenhuma base de dados aberta".into();
            return;
        }
        self.db = None;
        self.db_file = None;
        self.db_path.clear();
        self.docs.clear();
        self.active = 0;
        self.edit = None;
        self.confirm_del = None;
        self.pedir_nome = None;
        self.n_consulta = 0;
        self.n_tabela = 0;
        self.limpar_historico();
        self.totais_aberto = false;
        self.indices = None;
        self.relacoes = None;
        self.eliminar_obj = None;
        self.renomear_obj = None;
        self.importar = None;
        self.exportar_csv = None;
        self.foco_nav = false;
        self.refresh_nav();
        self.msg = "Base de dados fechada".into();
    }

    pub(crate) fn compactar_bd(&mut self) {
        let Some(db) = self.db.as_ref() else {
            self.msg = "Nenhuma base de dados aberta".into();
            return;
        };
        match db.compact() {
            Ok(r) if r == "ok" => {
                self.msg = "Base de dados verificada e compactada: sem erros".into()
            }
            Ok(r) => self.msg = format!("A verificação encontrou problemas: {}", r),
            Err(e) => self.msg = e,
        }
    }
}
