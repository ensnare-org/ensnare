// Copyright (c) 2024 Mike Tsao

//! Thin wrappers around stock egui widgets that make them easier to use with
//! Ensnare types.

use crate::prelude::*;
use eframe::egui::DragValue;

/// An egui widget that makes it easier to work with a [DragValue] and a [Normal].
#[derive(Debug)]
pub struct DragNormalWidget<'a> {
    normal: &'a mut Normal,
    prefix: Option<String>,
}
impl<'a> DragNormalWidget<'a> {
    fn new(normal: &'a mut Normal) -> Self {
        Self {
            normal,
            prefix: None,
        }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(normal: &'a mut Normal, prefix: &'a str) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| DragNormalWidget::new(normal).prefix(prefix).ui(ui)
    }

    #[allow(missing_docs)]
    pub fn prefix(mut self, prefix: impl ToString) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut value = self.normal.0 * 100.0;
        let mut dv = DragValue::new(&mut value).range(0.0..=100.0).suffix("%");
        if let Some(prefix) = &self.prefix {
            dv = dv.prefix(prefix);
        }
        let response = ui.add(dv);
        if response.changed() {
            *self.normal = Normal::from(value / 100.0);
        }
        response
    }
}
