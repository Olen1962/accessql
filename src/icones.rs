use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, Vec2};
use crate::tema::*;

// ───────────────────────────── Ícones desenhados (sem imagens) ─────────────────────────────

#[derive(Clone, Copy)]

pub(crate) enum Icon {

    Table,

    Query,

    Form,

    Report,

    Paste,

    Filter,

    Save,

    Undo,

    Redo,

    Badge(&'static str, Color32),

}



pub(crate) fn tri(p: &egui::Painter, pts: [Pos2; 3], col: Color32) {

    p.add(Shape::convex_polygon(pts.to_vec(), col, Stroke::NONE));

}



pub(crate) fn down_arrow(p: &egui::Painter, c: Pos2, col: Color32) {

    tri(

        p,

        [

            Pos2::new(c.x - 3.0, c.y - 1.5),

            Pos2::new(c.x + 3.0, c.y - 1.5),

            Pos2::new(c.x, c.y + 1.8),

        ],

        col,

    );

}



pub(crate) fn draw_icon(p: &egui::Painter, r: Rect, icon: Icon) {

    let s = r.width();

    let line = Stroke::new(1.0_f32, rgb(0x6B7F9E));

    match icon {

        Icon::Table | Icon::Query => {

            let b = r.shrink(s * 0.08);

            p.rect_filled(b, 1.0, Color32::WHITE);

            let hc = if matches!(icon, Icon::Query) { rgb(0x5BA05B) } else { rgb(0x4F81BD) };

            p.rect_filled(Rect::from_min_size(b.min, Vec2::new(b.width(), b.height() * 0.28)), 1.0, hc);

            for i in 1..3 {

                let y = b.top() + b.height() * (0.28 + 0.24 * i as f32);

                p.line_segment([Pos2::new(b.left(), y), Pos2::new(b.right(), y)], line);

            }

            p.line_segment([Pos2::new(b.center().x, b.top()), Pos2::new(b.center().x, b.bottom())], line);

            p.rect_stroke(b, 1.0, line);

            if matches!(icon, Icon::Query) {

                let c = Pos2::new(r.right() - s * 0.25, r.bottom() - s * 0.25);

                p.circle_filled(c, s * 0.16, rgb(0xFFF3B0));

                p.circle_stroke(c, s * 0.16, Stroke::new(1.5_f32, rgb(0x7A5A00)));

            }

        }

        Icon::Form => {

            let b = r.shrink(s * 0.08);

            p.rect_filled(b, 1.0, Color32::WHITE);

            p.rect_filled(Rect::from_min_size(b.min, Vec2::new(b.width(), b.height() * 0.22)), 1.0, rgb(0xE0A040));

            for i in 0..3 {

                let y = b.top() + b.height() * (0.34 + 0.22 * i as f32);

                p.line_segment([Pos2::new(b.left() + s * 0.08, y), Pos2::new(b.left() + s * 0.28, y)], line);

                p.rect_stroke(

                    Rect::from_min_max(Pos2::new(b.left() + s * 0.34, y - s * 0.06), Pos2::new(b.right() - s * 0.08, y + s * 0.06)),

                    0.0,

                    line,

                );

            }

            p.rect_stroke(b, 1.0, line);

        }

        Icon::Report => {

            let b = r.shrink2(Vec2::new(s * 0.18, s * 0.06));

            p.rect_filled(b, 1.0, Color32::WHITE);

            for i in 0..4 {

                let y = b.top() + b.height() * (0.14 + 0.14 * i as f32);

                p.line_segment([Pos2::new(b.left() + 3.0, y), Pos2::new(b.right() - 3.0, y)], line);

            }

            for i in 0..3 {

                let x = b.left() + b.width() * (0.18 + 0.25 * i as f32);

                let h = b.height() * (0.12 + 0.07 * i as f32);

                p.rect_filled(Rect::from_min_max(Pos2::new(x, b.bottom() - 3.0 - h), Pos2::new(x + b.width() * 0.16, b.bottom() - 3.0)), 0.0, C_RED);

            }

            p.rect_stroke(b, 1.0, line);

        }

        Icon::Paste => {

            let b = r.shrink2(Vec2::new(s * 0.12, s * 0.06));

            p.rect_filled(b, 2.0, rgb(0xB07A3A));

            p.rect_filled(Rect::from_center_size(Pos2::new(b.center().x, b.top() + 1.0), Vec2::new(b.width() * 0.45, s * 0.14)), 1.0, rgb(0x6E6E6E));

            p.rect_filled(Rect::from_min_max(Pos2::new(b.left() + s * 0.1, b.top() + s * 0.22), Pos2::new(b.right() - s * 0.1, b.bottom() - s * 0.06)), 0.0, Color32::WHITE);

        }

        Icon::Filter => {

            let c = r.center();

            let col = rgb(0x3F7FC4);

            tri(p, [Pos2::new(r.left() + s * 0.1, r.top() + s * 0.12), Pos2::new(r.right() - s * 0.1, r.top() + s * 0.12), Pos2::new(c.x, c.y + s * 0.12)], col);

            p.rect_filled(Rect::from_min_max(Pos2::new(c.x - s * 0.08, c.y), Pos2::new(c.x + s * 0.08, r.bottom() - s * 0.1)), 0.0, col);

        }

        Icon::Save => {

            let b = r.shrink(s * 0.1);

            p.rect_filled(b, 2.0, rgb(0x2F5FA7));

            p.rect_filled(Rect::from_min_max(Pos2::new(b.left() + s * 0.15, b.top()), Pos2::new(b.right() - s * 0.15, b.top() + b.height() * 0.4)), 0.0, Color32::WHITE);

            p.rect_filled(Rect::from_min_max(Pos2::new(b.left() + s * 0.22, b.bottom() - b.height() * 0.38), Pos2::new(b.right() - s * 0.22, b.bottom())), 0.0, rgb(0x1B3C6E));

        }

        Icon::Undo | Icon::Redo => {

            let flip = if matches!(icon, Icon::Redo) { -1.0 } else { 1.0 };

            let c = r.center();

            let rad = s * 0.32;

            let pts: Vec<Pos2> = (0..=8)

                .map(|i| {

                    let a = std::f32::consts::PI * (1.0 - i as f32 / 8.0);

                    Pos2::new(c.x + flip * rad * a.cos(), c.y + s * 0.1 - rad * a.sin())

                })

                .collect();

            p.add(Shape::line(pts, Stroke::new(1.8_f32, C_BLUE)));

            let h = Pos2::new(c.x - flip * rad, c.y + s * 0.1);

            tri(p, [Pos2::new(h.x - 3.0, h.y), Pos2::new(h.x + 3.0, h.y), Pos2::new(h.x, h.y + 4.5)], C_BLUE);

        }

        Icon::Badge(txt, col) => {

            p.rect_filled(r.shrink(s * 0.06), s * 0.18, col);

            p.text(r.center(), Align2::CENTER_CENTER, txt, FontId::proportional(s * 0.52), Color32::WHITE);

        }

    }

}
