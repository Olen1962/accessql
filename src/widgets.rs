use eframe::egui::{self, Align, Align2, Color32, FontId, Frame, Layout, Margin, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use crate::{icones::*, tema::*};

// ───────────────────────────── Widgets do ribbon ─────────────────────────────

pub(crate) fn tw(ui: &Ui, text: &str, size: f32) -> f32 {

    ui.painter()

        .layout_no_wrap(text.to_string(), FontId::proportional(size), Color32::BLACK)

        .size()

        .x

}



pub(crate) fn hover_bg(p: &egui::Painter, resp: &Response, rect: Rect) {

    if resp.is_pointer_button_down_on() {

        p.rect_filled(rect, 3.0, PRESS_FILL);

        p.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, HOVER_LINE));

    } else if resp.hovered() {

        p.rect_filled(rect, 3.0, HOVER_FILL);

        p.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, HOVER_LINE));

    }

}



pub(crate) fn big_btn(ui: &mut Ui, icon: Icon, label: &str, arrow: bool) -> Response {

    let font = FontId::proportional(12.0);

    let lines: Vec<&str> = label.split('\n').collect();

    let tw_max = lines.iter().map(|l| tw(ui, l, 12.0)).fold(0.0_f32, f32::max);

    let w = (tw_max + 14.0 + if arrow { 10.0 } else { 0.0 }).max(52.0);

    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 66.0), Sense::click());

    let p = ui.painter();

    hover_bg(p, &resp, rect);

    draw_icon(p, Rect::from_center_size(Pos2::new(rect.center().x, rect.top() + 21.0), Vec2::splat(32.0)), icon);

    for (i, l) in lines.iter().enumerate() {

        p.text(Pos2::new(rect.center().x, rect.top() + 41.0 + 12.0 * i as f32), Align2::CENTER_TOP, *l, font.clone(), TEXT);

    }

    if arrow {

        let last = lines.last().copied().unwrap_or("");

        let lw = tw(ui, last, 12.0);

        let y = rect.top() + 47.0 + 12.0 * (lines.len() - 1) as f32;

        down_arrow(ui.painter(), Pos2::new(rect.center().x + lw / 2.0 + 6.0, y), TEXT);

    }

    resp

}



pub(crate) fn small_btn(ui: &mut Ui, icon: Icon, label: &str, arrow: bool) -> Response {

    let tw_ = if label.is_empty() { 0.0 } else { tw(ui, label, 12.0) + 6.0 };

    let w = 24.0 + tw_ + if arrow { 12.0 } else { 2.0 };

    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 22.0), Sense::click());

    let p = ui.painter();

    hover_bg(p, &resp, rect);

    draw_icon(p, Rect::from_center_size(Pos2::new(rect.left() + 12.0, rect.center().y), Vec2::splat(16.0)), icon);

    if !label.is_empty() {

        p.text(Pos2::new(rect.left() + 24.0, rect.center().y), Align2::LEFT_CENTER, label, FontId::proportional(12.0), TEXT);

    }

    if arrow {

        down_arrow(p, Pos2::new(rect.right() - 7.0, rect.center().y), TEXT);

    }

    resp

}



pub(crate) fn group(ui: &mut Ui, title: &str, add: impl FnOnce(&mut Ui)) {

    let inner = ui.vertical(|ui| {

        ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);

        ui.horizontal_top(|ui| {

            ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);

            add(ui);

        });

        let w = ui.min_rect().width().max(tw(ui, title, 11.5) + 8.0);

        let (r, _) = ui.allocate_exact_size(Vec2::new(w, 16.0), Sense::hover());

        ui.painter().text(r.center(), Align2::CENTER_CENTER, title, FontId::proportional(11.5), GROUP_TXT);

    });

    let r = inner.response.rect;

    let x = r.right() + 3.0;

    ui.painter().line_segment([Pos2::new(x, r.top() + 2.0), Pos2::new(x, r.bottom())], Stroke::new(1.0_f32, GROUP_SEP));

    ui.add_space(6.0);

}



pub(crate) fn faux_combo(ui: &mut Ui, text: &str, w: f32) {

    Frame::none()

        .fill(Color32::WHITE)

        .stroke(Stroke::new(1.0_f32, rgb(0xABC0DE)))

        .inner_margin(Margin::symmetric(4.0, 1.0))

        .show(ui, |ui| {

            ui.set_width(w);

            ui.horizontal(|ui| {

                ui.label(RichText::new(text).size(12.0).color(TEXT));

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {

                    let (r, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());

                    down_arrow(ui.painter(), r.center(), TEXT);

                });

            });

        });

}



#[derive(Clone, Copy)]

pub(crate) enum TabStyle {

    File,

    Normal,

    Context,

}



pub(crate) fn ribbon_tab(ui: &mut Ui, label: &str, selected: bool, style: TabStyle) -> Response {

    let extra = if matches!(style, TabStyle::File) { 34.0 } else { 22.0 };

    let w = tw(ui, label, 13.0) + extra;

    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 24.0), Sense::click());

    let p = ui.painter();

    let top = Rounding { nw: 3.0, ne: 3.0, sw: 0.0, se: 0.0 };

    let font = FontId::proportional(13.0);

    if matches!(style, TabStyle::File) {

        let fill = if selected || resp.hovered() { ACCENT_HOVER } else { ACCENT };

        p.rect_filled(rect, top, fill);

        p.text(rect.center(), Align2::CENTER_CENTER, label, font, Color32::WHITE);

    } else {

        if selected {

            p.rect_filled(rect, top, RIBBON_BG);

            p.rect_stroke(rect, top, Stroke::new(1.0_f32, RIBBON_LINE));

            p.line_segment(

                [Pos2::new(rect.left() + 1.0, rect.bottom() - 0.5), Pos2::new(rect.right() - 1.0, rect.bottom() - 0.5)],

                Stroke::new(1.5_f32, RIBBON_BG),

            );

        } else if resp.hovered() {

            p.rect_filled(rect, top, rgb(0xE6EEF9));

        }

        if matches!(style, TabStyle::Context) {

            p.rect_filled(Rect::from_min_size(rect.min, Vec2::new(w, 3.0)), top, rgb(0xF2B84B));

        }

        p.text(rect.center() + Vec2::new(0.0, 1.0), Align2::CENTER_CENTER, label, font, TEXT);

    }

    resp

}



pub(crate) fn nav_arrow(ui: &mut Ui, kind: u8) -> Response {

    let (rect, resp) = ui.allocate_exact_size(Vec2::new(22.0, 18.0), Sense::click());

    let p = ui.painter();

    if resp.hovered() {

        p.rect_filled(rect, 2.0, rgb(0xDCE6F4));

    }

    let c = rect.center();

    let col = rgb(0x4A5F80);

    let h = 4.5;

    let left = |cx: f32| tri(p, [Pos2::new(cx + 3.0, c.y - h), Pos2::new(cx + 3.0, c.y + h), Pos2::new(cx - 3.0, c.y)], col);

    let right = |cx: f32| tri(p, [Pos2::new(cx - 3.0, c.y - h), Pos2::new(cx - 3.0, c.y + h), Pos2::new(cx + 3.0, c.y)], col);

    let bar = |cx: f32| p.line_segment([Pos2::new(cx, c.y - h), Pos2::new(cx, c.y + h)], Stroke::new(1.5_f32, col));

    match kind {

        0 => {

            bar(c.x - 4.5);

            left(c.x + 1.0);

        }

        1 => left(c.x),

        2 => right(c.x),

        3 => {

            right(c.x - 1.0);

            bar(c.x + 4.5);

        }

        _ => {

            right(c.x - 4.0);

            p.text(Pos2::new(c.x + 5.0, c.y + 1.0), Align2::CENTER_CENTER, "*", FontId::proportional(13.0), col);

        }

    }

    resp

}



pub(crate) fn card(ui: &mut Ui, icon: Icon, title: &str, desc: &str) -> Response {

    let (rect, resp) = ui.allocate_exact_size(Vec2::new(440.0, 62.0), Sense::click());

    let p = ui.painter();

    hover_bg(p, &resp, rect);

    draw_icon(p, Rect::from_center_size(Pos2::new(rect.left() + 32.0, rect.center().y), Vec2::splat(38.0)), icon);

    p.text(Pos2::new(rect.left() + 62.0, rect.top() + 12.0), Align2::LEFT_TOP, title, FontId::proportional(14.0), BLUE_TITLE);

    p.text(Pos2::new(rect.left() + 62.0, rect.top() + 34.0), Align2::LEFT_TOP, desc, FontId::proportional(11.5), rgb(0x5A6B85));

    resp

}
