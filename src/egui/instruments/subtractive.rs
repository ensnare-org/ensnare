// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::{
    cores::instruments::{SubtractiveSynthCore, SUBTRACTIVE_PATCH_DIR},
    egui::{
        generators::LfoWidget, util::EnumComboBoxWidget, BiQuadFilterLowPass24dbWidget,
        BiQuadFilterWidgetAction, DcaWidget, DcaWidgetAction, EnvelopeWidget, LfoControllerWidget,
        OscillatorWidget,
    },
    prelude::*,
};
use convert_case::{Case, Casing};
use eframe::egui::{CollapsingHeader, ComboBox, Slider, Widget};
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum SubtractiveSynthWidgetAction {
    Link(ControlLinkSource, ControlIndex),
}

#[derive(Debug)]
pub struct SubtractiveSynthWidget<'a> {
    inner: &'a mut SubtractiveSynthCore,
    action: &'a mut Option<SubtractiveSynthWidgetAction>,
}
impl<'a> SubtractiveSynthWidget<'a> {
    fn new(
        inner: &'a mut SubtractiveSynthCore,
        action: &'a mut Option<SubtractiveSynthWidgetAction>,
    ) -> Self {
        Self { inner, action }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        inner: &'a mut SubtractiveSynthCore,
        action: &'a mut Option<SubtractiveSynthWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| SubtractiveSynthWidget::new(inner, action).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for SubtractiveSynthWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut response = ComboBox::from_label("Preset")
            .selected_text(self.inner.preset_name().unwrap_or(&"None".to_string()))
            .show_ui(ui, |ui| {
                let mut bool_response = false;

                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);
                let mut current_value = self
                    .inner
                    .preset_name()
                    .cloned()
                    .unwrap_or("None".to_string());
                for patch in SUBTRACTIVE_PATCH_DIR.files() {
                    let visible = patch
                        .path()
                        .to_str()
                        .unwrap_or_default()
                        .replace(".json", "")
                        .to_case(Case::Title);
                    if ui
                        .selectable_value(&mut current_value, visible.clone(), visible.clone())
                        .changed()
                    {
                        bool_response = true;

                        // TODO - this is just a hack. It's doing real work on
                        // the UI thread, and it doesn't handle failure well.
                        *self.inner = SubtractiveSynthCore::load_patch_from_json(
                            patch.contents_utf8().unwrap(),
                        )
                        .unwrap();
                        self.inner.preset_name = Some(visible);
                    }
                }
                bool_response
            });
        if let Some(bool_response) = response.inner {
            if bool_response {
                response.response.mark_changed();
            }
        }
        let mut response = response.response;

        response |= CollapsingHeader::new("Oscillator 1")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show_unindented(ui, |ui| {
                if ui
                    .add(OscillatorWidget::widget(&mut self.inner.oscillator_1))
                    .changed()
                {
                    self.inner.notify_change_oscillator_1();
                }
            })
            .header_response;
        response |= CollapsingHeader::new("Oscillator 2")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show_unindented(ui, |ui| {
                if ui
                    .add(OscillatorWidget::widget(&mut self.inner.oscillator_2))
                    .changed()
                {
                    self.inner.notify_change_oscillator_2();
                }
            })
            .header_response;
        let mut oscillator_mix = self.inner.oscillator_mix.0;
        if ui
            .add(Slider::new(&mut oscillator_mix, 0.0..=1.0).text("Osc Blend"))
            .changed()
        {
            self.inner.set_oscillator_mix(oscillator_mix.into());
        }

        if let Some(lfo_response) = CollapsingHeader::new("LFO")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show_unindented(ui, |ui| {
                ui.add(LfoWidget::widget(&mut self.inner.lfo))
                    | ui.add(EnumComboBoxWidget::new(
                        &mut self.inner.lfo_routing,
                        "Routing",
                    ))
            })
            .body_response
        {
            response |= lfo_response;
        }

        response |= CollapsingHeader::new("DCA")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show_unindented(ui, |ui| {
                let mut action = None;
                if ui
                    .add(DcaWidget::widget(&mut self.inner.dca, &mut action))
                    .changed()
                {
                    self.inner.notify_change_dca();
                }
                if let Some(action) = action {
                    match action {
                        DcaWidgetAction::Link(source, index) => {
                            *self.action = Some(SubtractiveSynthWidgetAction::Link(
                                source,
                                index + SubtractiveSynthCore::DCA_INDEX,
                            ))
                        }
                    }
                }
            })
            .header_response;
        response |= CollapsingHeader::new("Amplitude")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show_unindented(ui, |ui| {
                if ui
                    .add(EnvelopeWidget::widget(&mut self.inner.amp_envelope))
                    .changed()
                {
                    self.inner.notify_change_amp_envelope();
                }
            })
            .header_response;
        response |= CollapsingHeader::new("Low-Pass Filter")
            .default_open(true)
            .id_source(ui.next_auto_id())
            .show_unindented(ui, |ui| {
                let mut action = None;
                if ui
                    .add(BiQuadFilterLowPass24dbWidget::widget(
                        &mut self.inner.filter,
                        &mut action,
                    ))
                    .changed()
                {
                    self.inner.notify_change_filter();
                }
                if ui
                    .add(EnvelopeWidget::widget(&mut self.inner.filter_envelope))
                    .changed()
                {
                    self.inner.notify_change_filter_envelope();
                }
                if let Some(action) = action {
                    match action {
                        BiQuadFilterWidgetAction::Link(source, param) => {
                            *self.action = Some(SubtractiveSynthWidgetAction::Link(source, param));
                        }
                    }
                }
            })
            .header_response;
        response
    }
}
