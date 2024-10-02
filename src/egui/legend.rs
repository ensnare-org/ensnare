// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use eframe::{
    egui::{vec2, Widget},
    emath::{Align2, RectTransform},
    epaint::{pos2, FontId},
};

/// An egui widget that draws a legend on the horizontal axis of the timeline
/// view.
#[derive(Debug)]
pub struct LegendWidget<'a> {
    /// The GUI view's time range.
    view_range: &'a mut ViewRange,
}
impl<'a> LegendWidget<'a> {
    fn new(view_range: &'a mut ViewRange) -> Self {
        Self { view_range }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(view_range: &mut ViewRange) -> impl eframe::egui::Widget + '_ {
        move |ui: &mut eframe::egui::Ui| LegendWidget::new(view_range).ui(ui)
    }

    pub(super) fn steps(view_range: &ViewRange) -> std::iter::StepBy<core::ops::Range<usize>> {
        let beat_count = view_range.0.end.total_beats() - view_range.0.start.total_beats();
        let step = (beat_count as f32).log10().round() as usize;
        (view_range.0.start.total_beats()..view_range.0.end.total_beats()).step_by(step * 2)
    }
}
impl<'a> eframe::egui::Widget for LegendWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let desired_size = vec2(ui.available_width(), ui.spacing().interact_size.y);
        let (rect, response) = ui.allocate_exact_size(desired_size, eframe::egui::Sense::click());
        let to_screen = RectTransform::from_to(
            eframe::epaint::Rect::from_x_y_ranges(
                self.view_range.0.start.total_beats() as f32
                    ..=self.view_range.0.end.total_beats() as f32,
                rect.top()..=rect.bottom(),
            ),
            rect,
        );

        let font_id = FontId::proportional(12.0);
        for beat in Self::steps(self.view_range) {
            let beat_plus_one = beat + 1;
            let pos = to_screen * pos2(beat as f32, rect.top());
            ui.painter().text(
                pos,
                Align2::CENTER_TOP,
                format!("{beat_plus_one}"),
                font_id.clone(),
                ui.style().noninteractive().text_color(),
            );
        }
        response
    }
}
