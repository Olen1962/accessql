//! Validação dos valores introduzidos na folha de dados, antes de gravar.
//!
//! As restrições que o próprio SQLite aplica (`UNIQUE`, `CHECK`, chaves estrangeiras)
//! continuam a ser tratadas por ele; aqui valida-se o que o Access validaria antes
//! de enviar o valor para a base de dados: tipo, obrigatoriedade e formato.

/// Valida `valor` para um campo com o tipo declarado `tipo`.
/// `nome` só entra nas mensagens de erro.
pub(crate) fn validar(
    tipo: &str,
    obrigatorio: bool,
    valor: &str,
    nome: &str,
) -> Result<(), String> {
    let erro = |m: String| format!("Campo «{}»: {}", nome, m);
    let v = valor.trim();
    if v.is_empty() {
        return if obrigatorio {
            Err(erro("é obrigatório e não pode ficar vazio".into()))
        } else {
            Ok(())
        };
    }
    match tipo.trim().to_ascii_uppercase().as_str() {
        "INTEGER" => v
            .parse::<i64>()
            .map(|_| ())
            .map_err(|_| erro(format!("«{}» não é um número inteiro válido", valor))),
        "REAL" | "FLOAT" | "DOUBLE" | "NUMERIC" | "DECIMAL" => v
            .replace(',', ".")
            .parse::<f64>()
            .map(|_| ())
            .map_err(|_| erro(format!("«{}» não é um número válido", valor))),
        "BOOLEAN" | "BOOL" | "YESNO" => {
            let b = v.to_lowercase();
            let aceites = [
                "0",
                "1",
                "s",
                "n",
                "sim",
                "não",
                "nao",
                "y",
                "yes",
                "no",
                "t",
                "true",
                "f",
                "false",
                "verdadeiro",
                "falso",
            ];
            if aceites.contains(&b.as_str()) {
                Ok(())
            } else {
                Err(erro(format!("«{}» não é um valor Sim/Não válido", valor)))
            }
        }
        "DATE" => data(v).map(|_| ()).ok_or_else(|| {
            erro(format!(
                "«{}» não é uma data válida (AAAA-MM-DD ou DD/MM/AAAA)",
                valor
            ))
        }),
        "DATETIME" | "TIMESTAMP" => data_hora(v)
            .map(|_| ())
            .ok_or_else(|| erro(format!("«{}» não é uma data/hora válida", valor))),
        // TEXT, BLOB e tipos declarados personalizados: texto livre.
        _ => Ok(()),
    }
}

/// Aceita `AAAA-MM-DD` e `DD/MM/AAAA`. Devolve (ano, mês, dia).
fn data(s: &str) -> Option<(i32, u32, u32)> {
    let sep = if s.contains('-') {
        '-'
    } else if s.contains('/') {
        '/'
    } else {
        return None;
    };
    let p: Vec<&str> = s.split(sep).collect();
    if p.len() != 3 {
        return None;
    }
    let (a, m, d) = if sep == '-' {
        (
            p[0].parse::<i32>().ok()?,
            p[1].parse::<u32>().ok()?,
            p[2].parse::<u32>().ok()?,
        )
    } else {
        (
            p[2].parse::<i32>().ok()?,
            p[1].parse::<u32>().ok()?,
            p[0].parse::<u32>().ok()?,
        )
    };
    if a < 1 || !(1..=12).contains(&m) || d < 1 || d > dias_do_mes(a, m) {
        return None;
    }
    Some((a, m, d))
}

fn data_hora(s: &str) -> Option<()> {
    let (d, h) = if let Some(i) = s.find('T') {
        (&s[..i], &s[i + 1..])
    } else if let Some(i) = s.find(' ') {
        (&s[..i], &s[i + 1..])
    } else {
        return data(s).map(|_| ());
    };
    data(d)?;
    let p: Vec<&str> = h.split(':').collect();
    if p.is_empty() || p.len() > 3 {
        return None;
    }
    let hh = p[0].parse::<u32>().ok()?;
    let mm = match p.get(1) {
        Some(x) => x.parse::<u32>().ok()?,
        None => 0,
    };
    let ss = match p.get(2) {
        Some(x) => x.parse::<f64>().ok()?,
        None => 0.0,
    };
    if hh > 23 || mm > 59 || ss >= 60.0 {
        return None;
    }
    Some(())
}

fn dias_do_mes(a: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if a % 4 == 0 && (a % 100 != 0 || a % 400 == 0) => 29,
        2 => 28,
        _ => 0,
    }
}

#[cfg(test)]
mod testes {
    use super::validar;

    #[test]
    fn texto_e_obrigatoriedade() {
        assert!(validar("TEXT", false, "", "Nome").is_ok());
        assert!(validar("TEXT", true, "", "Nome").is_err());
        assert!(validar("TEXT", false, "   ", "Nome").is_ok());
        assert!(validar("TEXT", true, "   ", "Nome").is_err());
        assert!(validar("TEXT", false, "Hugo", "Nome").is_ok());
    }

    #[test]
    fn numeros() {
        assert!(validar("INTEGER", false, "42", "ID").is_ok());
        assert!(validar("INTEGER", false, "-7", "ID").is_ok());
        assert!(validar("INTEGER", false, "1,5", "ID").is_err());
        assert!(validar("REAL", false, "1.5", "Preço").is_ok());
        assert!(validar("REAL", false, "1,5", "Preço").is_ok());
        assert!(validar("NUMERIC", false, "abc", "Preço").is_err());
    }

    #[test]
    fn sim_nao() {
        for v in ["0", "1", "sim", "NÃO", "true", "falso", "y"] {
            assert!(
                validar("BOOLEAN", false, v, "Ativo").is_ok(),
                "valor: {}",
                v
            );
        }
        assert!(validar("BOOLEAN", false, "talvez", "Ativo").is_err());
    }

    #[test]
    fn datas() {
        assert!(validar("DATE", false, "2026-11-19", "Nascimento").is_ok());
        assert!(validar("DATE", false, "19/11/2026", "Nascimento").is_ok());
        assert!(validar("DATE", false, "2024-02-29", "Nascimento").is_ok());
        assert!(validar("DATE", false, "2026-02-29", "Nascimento").is_err());
        assert!(validar("DATE", false, "31/04/2026", "Nascimento").is_err());
        assert!(validar("DATE", false, "19-11", "Nascimento").is_err());
        assert!(validar("DATETIME", false, "2026-11-19 14:30", "Quando").is_ok());
        assert!(validar("DATETIME", false, "2026-11-19T25:00", "Quando").is_err());
    }

    #[test]
    fn tipos_personalizados_aceitam_texto() {
        assert!(validar("VARCHAR(50)", false, "abc", "Código").is_ok());
        assert!(validar("BLOB", false, "abc", "Ficheiro").is_ok());
    }
}
