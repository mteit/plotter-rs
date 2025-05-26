#![cfg(target_arch = "wasm32")]
mod app;

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[derive(Clone)]
#[wasm_bindgen]
pub struct WebHandle {
  runner: eframe::WebRunner,
}

#[wasm_bindgen]
impl WebHandle {
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self {
      runner: eframe::WebRunner::new(),
    }
  }

  #[wasm_bindgen]
  pub async fn start(&self, canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
      .get_element_by_id(canvas_id)
      .unwrap()
      .dyn_into::<HtmlCanvasElement>()
      .unwrap();
    self
      .runner
      .start(
        canvas,
        eframe::WebOptions::default(),
        Box::new(|_cc| Ok(Box::new(app::PlotterApp::default()))),
      )
      .await
  }

  #[wasm_bindgen]
  pub fn destroy(&self) {
    self.runner.destroy();
  }
}

#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<WebHandle, wasm_bindgen::JsValue> {
  let web_handle = WebHandle::new();
  web_handle.start(canvas_id).await?;
  Ok(web_handle)
}
