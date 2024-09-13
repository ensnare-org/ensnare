// Copyright (c) 2024 Mike Tsao

use super::DragNormalWidget;
use crate::prelude::*;
use anyhow::anyhow;
use eframe::{
    egui::{self, ComboBox, Sense, Slider, Widget},
    emath::RectTransform,
    epaint::{pos2, Color32, Rect, RectShape, Rounding, Stroke},
};
use spectrum_analyzer::{scaling::divide_by_N_sqrt, FrequencyLimit};
use strum::IntoEnumIterator;

/// An egui widget for picking a waveform.
#[derive(Debug)]
pub struct WaveformWidget<'a> {
    inner: &'a mut Waveform,
}
impl<'a> WaveformWidget<'a> {
    fn new(inner: &'a mut Waveform) -> Self {
        Self { inner }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(waveform: &'a mut Waveform) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| WaveformWidget::new(waveform).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for WaveformWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let r = ComboBox::new(ui.next_auto_id(), "Waveform")
            .selected_text(self.inner.to_string())
            .show_ui(ui, |ui| {
                Waveform::iter()
                    .map(|w| ui.selectable_value(self.inner, w, w.to_string()))
                    .reduce(|acc, r| acc | r)
                    .unwrap()
            });
        let mut response = r.inner.unwrap_or(r.response);

        if let Waveform::PulseWidth(pulse_width) = self.inner {
            let mut normal = pulse_width.clone();
            let pulse_width_response =
                ui.add(DragNormalWidget::widget(&mut normal, "Duty cycle: "));
            if pulse_width_response.changed() {
                *self.inner = Waveform::PulseWidth(normal);
            }
            response |= pulse_width_response;
        }

        response
    }
}

/// An egui widget for picking a frequency within a range.
#[derive(Debug)]
pub struct FrequencyWidget<'a> {
    range: FrequencyRange,
    frequency: &'a mut FrequencyHz,
}
impl<'a> FrequencyWidget<'a> {
    fn new(range: FrequencyRange, frequency: &'a mut FrequencyHz) -> Self {
        Self { range, frequency }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        range: FrequencyRange,
        frequency: &'a mut FrequencyHz,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| FrequencyWidget::new(range, frequency).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for FrequencyWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut frequency = self.frequency.0;
        let range = self.range.as_range_frequency_hz();
        let slider = Slider::new(&mut frequency, range.start().0..=range.end().0);
        let response = ui.add(
            slider
                .fixed_decimals(self.range.fixed_digit_count())
                .suffix(FrequencyHz::UNITS_SUFFIX)
                .text("Frequency"),
        );
        if response.changed() {
            *self.frequency = FrequencyHz(frequency);
        }
        response
    }
}

/// Does a quick-and-dirty FFT of the input samples, producing a buffer that
/// is suitable for an unlabeled visualization. If you want labels, then do
/// this transformation yourself so you can display the Hz bucket labels.
///
/// TODO: there's a ton of heap usage in this method. See whether the crate
/// can be enhanced to work better with preallocated buffers.
pub fn analyze_spectrum(slice_1: &[Sample], slice_2: &[Sample]) -> anyhow::Result<Vec<f32>> {
    let rotated: Vec<f32> = slice_1
        .iter()
        .chain(slice_2.iter())
        .map(|s| s.0 as f32)
        .collect();
    let hann_window = spectrum_analyzer::windows::hann_window(&rotated);
    match spectrum_analyzer::samples_fft_to_spectrum(
        &hann_window,
        44100,
        FrequencyLimit::All,
        Some(&divide_by_N_sqrt),
    ) {
        Ok(spectrum) => Ok(spectrum.data().iter().map(|pair| pair.1.val()).collect()),
        Err(e) => Err(anyhow!("samples_fft_to_spectrum failed: {e:?}")),
    }
}

/// Displays a series of [Sample]s in the time domain. That's a fancy way of
/// saying it shows the amplitudes.
///
/// The series is passed in as two slices because we expect that callers will
/// have used a ring buffer to hold the incoming samples, and we don't want to
/// require a contiguous buffer, which would require expensive ring-buffer
/// rotations most of the time.
#[derive(Debug)]
pub struct TimeDomainWidget<'a> {
    slice_1: &'a [Sample],
    slice_2: &'a [Sample],
}
impl<'a> TimeDomainWidget<'a> {
    fn new(slice_1: &'a [Sample], slice_2: &'a [Sample]) -> Self {
        Self { slice_1, slice_2 }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(slice_1: &'a [Sample], slice_2: &'a [Sample]) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| TimeDomainWidget::new(slice_1, slice_2).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for TimeDomainWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> eframe::egui::Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        let rect = response.rect.shrink(1.0);

        let buffer_len = self.slice_1.len() + self.slice_2.len();
        let to_screen = RectTransform::from_to(
            Rect::from_x_y_ranges(0.0..=buffer_len as f32, Sample::MAX.0 as f32..=0.0f32),
            rect,
        );
        let mut shapes = Vec::default();

        shapes.push(eframe::epaint::Shape::Rect(RectShape::new(
            rect,
            Rounding::same(3.0),
            ui.visuals().window_fill,
            ui.visuals().window_stroke,
        )));

        for (i, sample) in self.slice_1.iter().chain(self.slice_2).enumerate() {
            let sample_abs = sample.0.abs();
            let sample_as_u8 = (sample_abs * 255.0) as u8;
            shapes.push(eframe::epaint::Shape::LineSegment {
                points: [
                    to_screen * pos2(i as f32, 0.0f32),
                    to_screen * pos2(i as f32, sample_abs as f32),
                ],
                stroke: Stroke::new(
                    1.0,
                    Color32::from_rgb(sample_as_u8, u8::MAX - sample_as_u8, 0),
                )
                .into(),
            })
        }

        painter.extend(shapes);
        response
    }
}

/// Displays a series of [Sample]s in the frequency domain. Or, to put it
/// another way, shows a spectrum analysis of a clip.
#[derive(Debug)]
pub struct FrequencyDomainWidget<'a> {
    values: &'a [f32],
}
impl<'a> FrequencyDomainWidget<'a> {
    fn new(values: &'a [f32]) -> Self {
        Self { values }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(values: &[f32]) -> impl eframe::egui::Widget + '_ {
        move |ui: &mut eframe::egui::Ui| FrequencyDomainWidget::new(values).ui(ui)
    }
}
impl<'a> eframe::egui::Widget for FrequencyDomainWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> eframe::egui::Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        let rect = response.rect.shrink(1.0);

        let buf_min = 0.0;
        let buf_max = 1.0;

        #[allow(unused_variables)]
        let to_screen = RectTransform::from_to(
            Rect::from_x_y_ranges(0.0..=self.values.len() as f32, buf_max..=buf_min),
            rect,
        );
        let mut shapes = Vec::default();

        shapes.push(eframe::epaint::Shape::Rect(RectShape::new(
            rect,
            Rounding::same(3.0),
            ui.visuals().window_fill,
            ui.visuals().window_stroke,
        )));

        for (i, value) in self.values.iter().enumerate() {
            shapes.push(eframe::epaint::Shape::LineSegment {
                points: [
                    to_screen * pos2(i as f32, buf_min),
                    to_screen * pos2(i as f32, *value),
                ],
                stroke: Stroke {
                    width: 1.0,
                    color: Color32::GREEN,
                }
                .into(),
            });
        }

        painter.extend(shapes);
        response
    }
}
