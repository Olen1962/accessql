use eframe::egui::Color32;

// ───────────────────────────── Paleta Office 2010 (azul) + Access ─────────────────────────────

pub(crate) const fn rgb(c: u32) -> Color32 {

    Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)

}

pub(crate) const ACCENT: Color32 = rgb(0xA4373A); // vermelho-Access (separador Ficheiro)

pub(crate) const ACCENT_HOVER: Color32 = rgb(0xBB4A4D);

pub(crate) const TITLE_BG: Color32 = rgb(0xC9DAF0);

pub(crate) const TAB_BG: Color32 = rgb(0xD5E3F5);

pub(crate) const RIBBON_BG: Color32 = rgb(0xEAF1FA);

pub(crate) const RIBBON_LINE: Color32 = rgb(0x9CB7DC);

pub(crate) const GROUP_SEP: Color32 = rgb(0xC3D3E9);

pub(crate) const GROUP_TXT: Color32 = rgb(0x3E5C8A);

pub(crate) const TEXT: Color32 = rgb(0x1E2F4A);

pub(crate) const BLUE_TITLE: Color32 = rgb(0x1F3F73);

pub(crate) const HOVER_FILL: Color32 = rgb(0xFFF1C2);

pub(crate) const HOVER_LINE: Color32 = rgb(0xE8BE5F);

pub(crate) const PRESS_FILL: Color32 = rgb(0xFFD58F);

pub(crate) const NAV_BG: Color32 = rgb(0xF2F6FC);

pub(crate) const NAV_HEAD: Color32 = rgb(0xB9CDE8);

pub(crate) const NAV_GROUP: Color32 = rgb(0x7F9FCE);

pub(crate) const WORK_BG: Color32 = rgb(0xBFCFE5);

pub(crate) const STATUS_BG: Color32 = rgb(0xD5E3F5);

pub(crate) const GRID: Color32 = rgb(0xD3D8E0);



pub(crate) const C_BLUE: Color32 = rgb(0x3B6EB5);

pub(crate) const C_GREEN: Color32 = rgb(0x4E9A4E);

pub(crate) const C_RED: Color32 = rgb(0xC0504D);

pub(crate) const C_ORANGE: Color32 = rgb(0xE08A2E);

pub(crate) const C_GRAY: Color32 = rgb(0x7F8FA9);

pub(crate) const C_PURPLE: Color32 = rgb(0x7B5EA7);
