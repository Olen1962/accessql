mod tema;
mod icones;
mod widgets;
mod dados_exemplo;
mod db;
mod ficheiro;
mod recentes;
mod estado;
mod ribbon;
mod navegacao;
mod area_trabalho;
mod status;
mod backstage;
mod menus;
mod app;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1366.0, 820.0])
            .with_min_inner_size([1150.0, 600.0])
            .with_title("AccesSQL")
            .with_app_id("accessql"),
        ..Default::default()
    };
    eframe::run_native(
        "AccesSQL",
        options,
        Box::new(|cc| Ok(Box::new(estado::AccessApp::new(cc)))),
    )
}
