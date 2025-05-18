use eframe::{App, Frame, egui};
use egui::{Color32, FontFamily, Pos2, Rect, Stroke, StrokeKind, Vec2};
use math_parser::{MathFunction, generate_lambda};

const PIXELS_PER_MARK: f32 = 25.0;
const FUNCTION_RESOLUTION: i32 = 50001;

struct PlotterApp {
  expression: MathFunction,
  scale_factor: f32,
  position_offset: Vec2,
  formula: String,
}

impl Default for PlotterApp {
  fn default() -> Self {
    Self {
      expression: generate_lambda("x^2").unwrap(),
      scale_factor: 1.0,
      position_offset: egui::Vec2::ZERO,
      formula: String::new(),
    }
  }
}

impl App for PlotterApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      let canvas_size = Vec2::new(500.0, 500.0);
      let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::drag());
      let canvas_rect = response.rect;
      if response.hovered() {
        ctx.input(|input| {
          for event in &input.events {
            if let egui::Event::Zoom(factor) = event {
              self.scale_factor = (self.scale_factor * factor).max(0.1).min(10.0);
            }
          }
          let scroll_delta = input.raw_scroll_delta.y;
          if scroll_delta != 0.0 {
            let zoom_factor = {
              let x: f32 = if self.scale_factor > 1.0 { 0.1 } else { 0.01 };
              x.copysign(scroll_delta)
            };
            self.scale_factor = {
              let multiplier = if self.scale_factor > 1.0 { 10.0 } else { 100.0 };
              let value = self.scale_factor + zoom_factor;
              (value * multiplier).round() / multiplier
            }
            .min(10.0)
            .max(0.1);
          };
        });
      }
      if response.dragged() {
        self.position_offset += response.drag_delta();
      }
      if ui.button("Reset view").clicked() {
        self.scale_factor = 1.0;
        self.position_offset = Vec2::ZERO;
      }
      if ui.text_edit_singleline(&mut self.formula).changed() {
        self.expression = generate_lambda(&self.formula).unwrap_or(Box::new(|x| Ok(x * x)));
      }
      draw_axes(
        &painter,
        &canvas_rect,
        &self.scale_factor,
        &self.position_offset,
      );
      draw_function(
        &painter,
        &canvas_rect,
        &self.scale_factor,
        &self.position_offset,
        &self.expression,
      );
    });
  }
}

fn draw_axes(painter: &egui::Painter, canvas: &Rect, scale_factor: &f32, offset: &Vec2) {
  let stroke = Stroke::new(1.0, Color32::from_rgb(100, 100, 100));
  painter.rect_stroke(*canvas, 0.0, stroke, StrokeKind::Inside);
  let x_axis: [Pos2; 2] = [
    canvas.clamp(Pos2::new(canvas.left(), (canvas.center() + *offset).y)),
    canvas.clamp(Pos2::new(canvas.right(), (canvas.center() + *offset).y)),
  ];
  let y_axis: [Pos2; 2] = [
    canvas.clamp(Pos2::new((canvas.center() + *offset).x, canvas.top())),
    canvas.clamp(Pos2::new((canvas.center() + *offset).x, canvas.bottom())),
  ];
  painter.line_segment(x_axis, stroke);
  painter.line_segment(y_axis, stroke);
  let mark_length = 5.0;
  for i in 0..=20 {
    let x = x_axis[0].x + offset.x % PIXELS_PER_MARK + PIXELS_PER_MARK * i as f32;
    let y = y_axis[0].y + offset.y % PIXELS_PER_MARK + PIXELS_PER_MARK * i as f32;
    painter.line_segment(
      [
        Pos2::new(x, x_axis[0].y - mark_length),
        Pos2::new(x, x_axis[0].y + mark_length),
      ],
      stroke,
    );
    painter.line_segment(
      [
        Pos2::new(y_axis[0].x - mark_length, y),
        Pos2::new(y_axis[0].x + mark_length, y),
      ],
      stroke,
    );

    let format_mark_value = |value: f32| -> String {
      match value.abs() {
        x if x == value.abs().trunc() => format!("{:.0}", value),
        ..0.01 => format!("{:.4}", value),
        ..100.0 => format!("{:.1}", value),
        _ => format!("{:.0}", value),
      }
    };
    let axis_offset = -10.0; // because marks start from (0;0)
    let x_value = format_mark_value(
      (axis_offset - (offset.x / PIXELS_PER_MARK).trunc() + i as f32) * scale_factor,
    );
    let y_value = format_mark_value(
      -(axis_offset - (offset.y / PIXELS_PER_MARK).trunc() + i as f32) * scale_factor,
    );
    let font_id = egui::FontId::new(10.0, FontFamily::Proportional);
    let text_indent = 10.0;
    if x_value != "0" {
      let text_rect = painter
        .layout_no_wrap(x_value.clone(), font_id.clone(), Color32::GRAY)
        .rect;
      let text_bottom_edge = x_axis[0].y + text_indent + text_rect.height();
      let (pos, align) = if text_bottom_edge < canvas.max.y {
        (
          Pos2::new(x, x_axis[0].y + text_indent),
          egui::Align2::CENTER_TOP,
        )
      } else {
        (
          Pos2::new(x, x_axis[0].y - text_indent),
          egui::Align2::CENTER_BOTTOM,
        )
      };
      painter.text(pos, align, x_value, font_id.clone(), Color32::GRAY);
    }
    if y_value != "-0" {
      let text_rect = painter
        .layout_no_wrap(y_value.clone(), font_id.clone(), Color32::GRAY)
        .rect;
      let text_left_edge = y_axis[0].x - text_indent - text_rect.width();
      let (pos, align) = if text_left_edge > canvas.min.x {
        (
          Pos2::new(y_axis[0].x - text_indent, y),
          egui::Align2::RIGHT_CENTER,
        )
      } else {
        (
          Pos2::new(y_axis[0].x + text_indent, y),
          egui::Align2::LEFT_CENTER,
        )
      };
      painter.text(pos, align, y_value, font_id.clone(), Color32::GRAY);
    }
  }
}

fn draw_function(
  painter: &egui::Painter,
  canvas: &Rect,
  scale_factor: &f32,
  offset: &Vec2,
  math_function: &MathFunction,
) {
  let stroke = Stroke::new(2.0, Color32::from_rgb(100, 200, 100));
  let to_screen = |point: Pos2| -> Pos2 {
    Pos2::new(
      canvas.center().x + point.x * PIXELS_PER_MARK / scale_factor,
      canvas.center().y - point.y * PIXELS_PER_MARK / scale_factor,
    ) + *offset
  };
  let x_min = (-canvas.width() / 2.0 - offset.x) / PIXELS_PER_MARK * scale_factor;
  let x_max = (canvas.width() / 2.0 - offset.x) / PIXELS_PER_MARK * scale_factor;
  let y_min = -(canvas.height() / 2.0 - offset.y) / PIXELS_PER_MARK * scale_factor;
  let y_max = -(-canvas.height() / 2.0 - offset.y) / PIXELS_PER_MARK * scale_factor;
  let step = (x_max - x_min) / (FUNCTION_RESOLUTION - 1) as f32;
  let math_points = {
    (0..FUNCTION_RESOLUTION)
      .map(|i| x_min + step * i as f32)
      .map(|x| -> Result<Pos2, String> {
        let y = math_function(x as f64)? as f32;
        if (y_min..=y_max).contains(&y) {
          Ok(Pos2::new(x, y as f32))
        } else {
          Err(String::new())
        }
      })
  };
  let graph_segments = {
    let mut segments = Vec::new();
    let mut current_segment = Vec::new();
    for item in math_points {
      match item {
        Ok(point) => {
          current_segment.push(to_screen(point));
        },
        Err(_) => {
          if !current_segment.is_empty() {
            segments.push(current_segment);
            current_segment = Vec::new();
          }
        },
      }
    }
    if !current_segment.is_empty() {
      segments.push(current_segment);
    }
    segments
  };
  graph_segments.into_iter().for_each(|segment| {
    painter.add(egui::Shape::line(segment, stroke));
  });
}

fn main() -> eframe::Result {
  let native_options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
      .with_title("PLOTTER")
      .with_inner_size([516.0, 558.0])
      .with_fullscreen(false)
      .with_maximized(false)
      .with_resizable(false),
    ..Default::default()
  };
  eframe::run_native(
    "plotter-rs",
    native_options,
    Box::new(|_| Ok(Box::new(PlotterApp::default()))),
  )
}
