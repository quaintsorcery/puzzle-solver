use app::App;

mod app;
mod node;
mod transform;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Puzzle Solver",
        native_options,
        Box::new(|cx| Ok(Box::new(App::new(cx)))),
    )
}
