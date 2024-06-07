// Copyright (c) 2024 Mike Tsao

#[cfg(feature = "egui")]
mod egui {
    use crate::{entities::Gain, prelude::*};
    use eframe::egui::Slider;

    impl Displays for Gain {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let mut ceiling = self.inner.ceiling().to_percentage();
            let response = ui.add(
                Slider::new(&mut ceiling, 0.0..=100.0)
                    .fixed_decimals(2)
                    .suffix(" %")
                    .text("Ceiling"),
            );
            if response.changed() {
                self.inner.set_ceiling(Normal::from_percentage(ceiling));
            };
            response
        }
    }
}
