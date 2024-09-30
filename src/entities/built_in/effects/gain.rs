// Copyright (c) 2024 Mike Tsao

use crate::{cores::GainCore, prelude::*};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Controls, GeneratesStereoSample, HandlesMidi, SkipInner)]

/// Entity wrapper for [GainCore].
pub struct Gain {
    uid: Uid,
    inner: GainCore,
}
impl Gain {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: GainCore) -> Self {
        Self { uid, inner }
    }
}

#[cfg(feature = "egui")]
mod egui {
    use super::Gain;
    use crate::prelude::*;
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
