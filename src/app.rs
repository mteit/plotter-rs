use math_parser::MathExpression;

const PIXELS_PER_MARK: f32 = 25.0;
const FUNCTION_RESOLUTION: i32 = 500;

struct Function {
  expression: Result<MathExpression, String>,
  formula: String,
  color: egui::Color32,
  resolution: i32,
}

impl Default for Function {
  fn default() -> Self {
    Self {
      expression: MathExpression::new(""),
      formula: String::new(),
      color: egui::Color32::GREEN,
      resolution: FUNCTION_RESOLUTION,
    }
  }
}

pub struct PlotterApp {
  function_list: Vec<Function>,
  scale_factor: f32,
  position_offset: egui::Vec2,
}

impl Default for PlotterApp {
  fn default() -> Self {
    Self {
      function_list: Vec::new(),
      scale_factor: 1.0,
      position_offset: egui::Vec2::ZERO,
    }
  }
}

impl eframe::App for PlotterApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.horizontal(|ui| {
        // Left column - Canvas
        ui.vertical(|ui| {
          ui.set_max_width(500.0);
          let canvas_size = egui::Vec2::new(500.0, 500.0);
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
          draw_axes(
            &painter,
            &canvas_rect,
            &self.scale_factor,
            &self.position_offset,
          );
          for function in &self.function_list {
            match &function.expression {
              Ok(expression) => {
                draw_function(
                  &painter,
                  &canvas_rect,
                  &self.scale_factor,
                  &self.position_offset,
                  &expression,
                  &function.color,
                  function.resolution,
                );
              },
              Err(_) => {},
            }
          }
          ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Reset view").clicked() {
              self.scale_factor = 1.0;
              self.position_offset = egui::Vec2::ZERO;
            }
          });
        }); // columns[0].vertical Left column - Canvas

        // Right column - Formula input
        self
          .function_list
          .retain(|function| !function.formula.is_empty());
        self.function_list.push(Function::default());
        ui.vertical(|ui| {
          ui.heading("Formulas");
          egui::ScrollArea::vertical().show(ui, |ui| {
            for function in &mut self.function_list {
              ui.horizontal(|ui| {
                if !function.formula.is_empty() {
                  ui.color_edit_button_srgba(&mut function.color);
                  ui.label("Points:");
                  let function_resolution = egui::DragValue::new(&mut function.resolution)
                    .range(10..=9999)
                    .speed(1);
                  ui.add(function_resolution);
                }
                let formula_imput = egui::TextEdit::singleline(&mut function.formula)
                  .text_color(match function.expression {
                    Ok(_) => egui::Color32::BLACK,
                    Err(_) => egui::Color32::RED,
                  })
                  .hint_text("Enter a formula")
                  .char_limit(100)
                  .desired_width(f32::INFINITY);
                if ui.add(formula_imput).changed() {
                  function.expression = MathExpression::new(function.formula.as_str());
                }
              }); // ui.horizontal
            } // for
          }); // egui::ScrollArea::vertical
        }); // columns[1].vertical Right column - Formula input
      }); // ui.columns
    }); // egui::CentralPanel
  }
}

fn draw_axes(
  painter: &egui::Painter,
  canvas: &egui::Rect,
  scale_factor: &f32,
  offset: &egui::Vec2,
) {
  let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100));
  painter.rect_stroke(*canvas, 0.0, stroke, egui::StrokeKind::Inside);
  let x_axis: [egui::Pos2; 2] = [
    canvas.clamp(egui::Pos2::new(
      canvas.left(),
      (canvas.center() + *offset).y,
    )),
    canvas.clamp(egui::Pos2::new(
      canvas.right(),
      (canvas.center() + *offset).y,
    )),
  ];
  let y_axis: [egui::Pos2; 2] = [
    canvas.clamp(egui::Pos2::new((canvas.center() + *offset).x, canvas.top())),
    canvas.clamp(egui::Pos2::new(
      (canvas.center() + *offset).x,
      canvas.bottom(),
    )),
  ];
  painter.line_segment(x_axis, stroke);
  painter.line_segment(y_axis, stroke);
  let mark_length = 5.0;
  for i in 0..=20 {
    let x = x_axis[0].x + offset.x % PIXELS_PER_MARK + PIXELS_PER_MARK * i as f32;
    let y = y_axis[0].y + offset.y % PIXELS_PER_MARK + PIXELS_PER_MARK * i as f32;
    painter.line_segment(
      [
        egui::Pos2::new(x, x_axis[0].y - mark_length),
        egui::Pos2::new(x, x_axis[0].y + mark_length),
      ],
      stroke,
    );
    painter.line_segment(
      [
        egui::Pos2::new(y_axis[0].x - mark_length, y),
        egui::Pos2::new(y_axis[0].x + mark_length, y),
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
    let font_id = egui::FontId::new(10.0, egui::FontFamily::Proportional);
    let text_indent = 10.0;
    if x_value != "0" {
      let text_rect = painter
        .layout_no_wrap(x_value.clone(), font_id.clone(), egui::Color32::GRAY)
        .rect;
      let text_bottom_edge = x_axis[0].y + text_indent + text_rect.height();
      let (pos, align) = if text_bottom_edge < canvas.max.y {
        (
          egui::Pos2::new(x, x_axis[0].y + text_indent),
          egui::Align2::CENTER_TOP,
        )
      } else {
        (
          egui::Pos2::new(x, x_axis[0].y - text_indent),
          egui::Align2::CENTER_BOTTOM,
        )
      };
      painter.text(pos, align, x_value, font_id.clone(), egui::Color32::GRAY);
    }
    if y_value != "-0" {
      let text_rect = painter
        .layout_no_wrap(y_value.clone(), font_id.clone(), egui::Color32::GRAY)
        .rect;
      let text_left_edge = y_axis[0].x - text_indent - text_rect.width();
      let (pos, align) = if text_left_edge > canvas.min.x {
        (
          egui::Pos2::new(y_axis[0].x - text_indent, y),
          egui::Align2::RIGHT_CENTER,
        )
      } else {
        (
          egui::Pos2::new(y_axis[0].x + text_indent, y),
          egui::Align2::LEFT_CENTER,
        )
      };
      painter.text(pos, align, y_value, font_id.clone(), egui::Color32::GRAY);
    }
  }
}

fn draw_function(
  painter: &egui::Painter,
  canvas: &egui::Rect,
  scale_factor: &f32,
  offset: &egui::Vec2,
  math_function: &MathExpression,
  color: &egui::Color32,
  resolution: i32,
) {
  let stroke = egui::Stroke::new(2.0, *color);
  let to_screen = |point: egui::Pos2| -> egui::Pos2 {
    egui::Pos2::new(
      canvas.center().x + point.x * PIXELS_PER_MARK / scale_factor,
      canvas.center().y - point.y * PIXELS_PER_MARK / scale_factor,
    ) + *offset
  };
  let x_min = (-canvas.width() / 2.0 - offset.x) / PIXELS_PER_MARK * scale_factor;
  let x_max = (canvas.width() / 2.0 - offset.x) / PIXELS_PER_MARK * scale_factor;
  let y_min = -(canvas.height() / 2.0 - offset.y) / PIXELS_PER_MARK * scale_factor;
  let y_max = -(-canvas.height() / 2.0 - offset.y) / PIXELS_PER_MARK * scale_factor;
  let step = (x_max - x_min) / (resolution - 1) as f32;
  let math_points = {
    (0..resolution)
      .map(|i| x_min + step * i as f32)
      .map(|x| -> Result<egui::Pos2, String> {
        let y = math_function.calculate(x as f64)? as f32;
        if (y_min..=y_max).contains(&y) {
          Ok(egui::Pos2::new(x, y as f32))
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
