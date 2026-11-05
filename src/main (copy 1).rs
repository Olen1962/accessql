mod tema;
mod icones;
mod widgets;
mod dados_exemplo;
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
            .with_title("Microsoft Access")
            .with_app_id("msaccess-mock"),
        ..Default::default()
    };
    eframe::run_native(
        "Microsoft Access",
        options,
        Box::new(|cc| Ok(Box::new(estado::AccessApp::new(cc)))),
    )
}
