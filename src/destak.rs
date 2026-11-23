use crate::estado::*;
use eframe::egui::{self, Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub(crate) enum DockZone { #[default] None, Left, Right, Top, Bottom, Center }

#[derive(Clone, Copy, Debug)]
pub(crate) struct DockPreview { pub zone: DockZone, pub rect: Rect }

impl AccessApp {
    pub(crate) fn destacar_doc(&mut self, index: usize) {
        if let Some(doc) = self.docs.get_mut(index) { doc.destacar(); }
    }
    pub(crate) fn ancorar_doc(&mut self, index: usize) {
        if let Some(doc) = self.docs.get_mut(index) { doc.ancorar(); }
    }
    pub(crate) fn alternar_destaque(&mut self) {
        if self.docs.is_empty() { return; }
        let i = self.active.min(self.docs.len() - 1);
        if self.docs[i].flutuante() { self.docs[i].ancorar(); } else { self.docs[i].destacar(); }
    }
    pub(crate) fn calcular_zona_drop(&self, pos: Pos2, alvo: Rect) -> DockZone {
        if !alvo.contains(pos) { return DockZone::None; }
        let dx = pos.x - alvo.left();
        let dy = pos.y - alvo.top();
        let w = alvo.width();
        let h = alvo.height();
        let mx = w * 0.25;
        let my = h * 0.25;
        if dx < mx { return DockZone::Left; }
        if dx > w - mx { return DockZone::Right; }
        if dy < my { return DockZone::Top; }
        if dy > h - my { return DockZone::Bottom; }
        DockZone::Center
    }
    pub(crate) fn rect_zona(&self, zona: DockZone, alvo: Rect) -> Rect {
        let w = alvo.width();
        let h = alvo.height();
        match zona {
            DockZone::Left => Rect::from_min_max(alvo.left_top(), Pos2::new(alvo.left() + w * 0.5, alvo.bottom())),
            DockZone::Right => Rect::from_min_max(Pos2::new(alvo.left() + w * 0.5, alvo.top()), alvo.right_bottom()),
            DockZone::Top => Rect::from_min_max(alvo.left_top(), Pos2::new(alvo.right(), alvo.top() + h * 0.5)),
            DockZone::Bottom => Rect::from_min_max(Pos2::new(alvo.left(), alvo.top() + h * 0.5), alvo.right_bottom()),
            DockZone::Center => alvo,
            DockZone::None => alvo,
        }
    }
    pub(crate) fn desenhar_indicador_dock(&self, ui: &mut Ui, preview: &DockPreview) {
        if preview.zone == DockZone::None { return; }
        let p = ui.painter();
        p.rect_filled(preview.rect, 0.0, Color32::from_rgba_unmultiplied(0x2B, 0x5A, 0x9E, 0x40));
        p.rect_stroke(preview.rect, 0.0, Stroke::new(2.0, Color32::from_rgb(0x2B, 0x5A, 0x9E)));
    }
    pub(crate) fn unir_doc(&mut self, flutuante: usize, alvo: usize, zona: DockZone) {
        if flutuante == alvo { return; }
        if let Some(d) = self.docs.get_mut(flutuante) { d.ancorar(); }
        // Reordenação simples: move o doc flutuante para junto do alvo
        if flutuante < self.docs.len() && alvo < self.docs.len() {
            let d = self.docs.remove(flutuante);
            let novo_alvo = if flutuante < alvo { alvo - 1 } else { alvo };
            let pos = match zona {
                DockZone::Left | DockZone::Top => novo_alvo,
                DockZone::Right | DockZone::Bottom | DockZone::Center => novo_alvo + 1,
                DockZone::None => novo_alvo + 1,
            };
            let pos = pos.min(self.docs.len());
            self.docs.insert(pos, d);
        }
        self.active = alvo.min(self.docs.len().saturating_sub(1));
    }
}