//! Lista de bases de dados recentes, guardada em
//! `$XDG_CONFIG_HOME/accessql/recentes.txt` (ou `~/.config/accessql/recentes.txt`).

use std::path::PathBuf;

const MAX: usize = 10;

fn ficheiro() -> Option<PathBuf> {
    let base = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => PathBuf::from(std::env::var_os("HOME")?).join(".config"),
    };
    Some(base.join("accessql").join("recentes.txt"))
}

pub(crate) fn carregar() -> Vec<PathBuf> {
    let Some(f) = ficheiro() else { return Vec::new() };
    std::fs::read_to_string(f)
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).map(PathBuf::from).take(MAX).collect())
        .unwrap_or_default()
}

pub(crate) fn guardar(lista: &[PathBuf]) {
    let Some(f) = ficheiro() else { return };
    if let Some(dir) = f.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let linhas: Vec<String> = lista.iter().map(|p| p.display().to_string()).collect();
    let _ = std::fs::write(f, linhas.join("\n"));
}

/// Põe `p` no topo da lista (sem repetições, no máximo `MAX` entradas).
pub(crate) fn adicionar(lista: &mut Vec<PathBuf>, p: PathBuf) {
    lista.retain(|x| x != &p);
    lista.insert(0, p);
    lista.truncate(MAX);
}
