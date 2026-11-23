//! Totais: resumo da coluna selecionada (contagem, soma, média, mínimo, máximo).

use crate::estado::*;
use eframe::egui::{self, Align2, RichText, Vec2};
use std::collections::HashSet;

fn numero(x: f64) -> String {
    let t = if x.fract() == 0.0 && x.abs() < 1e15 {
        format!("{}", x as i64)
    } else {
        let s = format!("{:.4}", x);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    };
    t.replace('.', ",")
}

impl AccessApp {
    pub(crate) fn desenhar_totais(&mut self, ctx: &egui::Context) {
        if !self.totais_aberto {
            return;
        }
        let Some(doc) = self.docs.get(self.active) else {
            self.totais_aberto = false;
            return;
        };
        if !matches!(doc.kind, ObjKind::Table | ObjKind::Query | ObjKind::Sql)
            || doc.data.cols.is_empty()
        {
            self.totais_aberto = false;
            self.msg = "Abra uma tabela ou consulta com dados para ver os totais".into();
            return;
        }
        let col = self.sel.1.min(doc.data.cols.len() - 1);
        let nome = doc.data.cols[col].0.clone();
        let valores: Vec<&str> = doc
            .data
            .rows
            .iter()
            .filter_map(|r| r.get(col).map(|s| s.as_str()))
            .collect();
        let total = valores.len();
        let preenchidos: Vec<&str> = valores
            .iter()
            .copied()
            .filter(|v| !v.trim().is_empty())
            .collect();
        let vazios = total - preenchidos.len();
        let distintos = preenchidos.iter().collect::<HashSet<_>>().len();
        let nums: Vec<f64> = preenchidos
            .iter()
            .filter_map(|v| v.trim().parse::<f64>().ok())
            .collect();
        let numerica = !preenchidos.is_empty() && nums.len() == preenchidos.len();
        let mut linhas: Vec<(&str, String)> = vec![
            ("Registos mostrados", total.to_string()),
            ("Com valor", preenchidos.len().to_string()),
            ("Vazios", vazios.to_string()),
            ("Valores distintos", distintos.to_string()),
        ];
        if numerica {
            let soma: f64 = nums.iter().sum();
            let min = nums.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            linhas.push(("Soma", numero(soma)));
            linhas.push(("Média", numero(soma / nums.len() as f64)));
            linhas.push(("Mínimo", numero(min)));
            linhas.push(("Máximo", numero(max)));
        } else if let (Some(min), Some(max)) = (preenchidos.iter().min(), preenchidos.iter().max())
        {
            linhas.push(("Primeiro (A-Z)", min.to_string()));
            linhas.push(("Último (A-Z)", max.to_string()));
        }
        let mut aberto = true;
        egui::Window::new("Totais")
            .open(&mut aberto)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(RichText::new(format!("Coluna «{}»", nome)).strong());
                ui.weak(
                    "Conta só os registos que estão a ser mostrados (com os filtros aplicados).",
                );
                ui.add_space(6.0);
                egui::Grid::new("totais_grid")
                    .spacing([24.0, 5.0])
                    .show(ui, |ui| {
                        for (k, v) in &linhas {
                            ui.label(*k);
                            ui.label(RichText::new(v.clone()).strong());
                            ui.end_row();
                        }
                    });
            });
        if !aberto {
            self.totais_aberto = false;
        }
    }
}
