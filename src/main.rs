#![cfg(not(target_arch = "wasm32"))]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;

fn main() -> eframe::Result {
  let native_options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
      .with_title("PLOTTER")
      .with_inner_size([916.0, 537.0])
      .with_fullscreen(false)
      .with_maximized(false)
      .with_resizable(false),
    ..Default::default()
  };

  eframe::run_native(
    "plotter-rs",
    native_options,
    Box::new(|_cc| Ok(Box::new(app::PlotterApp::default()))),
  )
}
