// Copyright (c) 2024 Mike Tsao

use crate::egui::{colors::ColorSchemeConverter, track::TrackSource};
use crate::{prelude::*, types::ColorScheme, util::SelectionSet};
use core::ops::Range;
use eframe::{
    egui::{style::WidgetVisuals, Modifiers, Sense, Widget},
    emath::RectTransform,
    epaint::{pos2, vec2, Color32, Rect, RectShape, Shape, Stroke},
};

#[derive(Debug)]
pub enum ArrangementWidgetAction {
    Unarrange(ArrangementUid),
    Duplicate(ArrangementUid),
    AddPattern(MusicalTime),
    ClearEditPattern,
    SetEditPattern(PatternUid),
}

/// An egui widget that draws a track arrangement overlaid in the track view.
#[derive(Debug)]
pub struct ArrangementWidget<'a> {
    track_uid: TrackUid,
    composer: &'a Composer,
    view_range: &'a ViewRange,
    color_scheme: ColorScheme,
    new_arrangement_to_select: Option<ArrangementUid>,
    action: &'a mut Option<ArrangementWidgetAction>,
}
impl<'a> eframe::egui::Widget for ArrangementWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.allocate_ui(vec2(ui.available_width(), 64.0), |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
            let x_range_f32 = self.view_range.0.start.total_units() as f32
                ..=self.view_range.0.end.total_units() as f32;
            let y_range = i8::MAX as f32..=u8::MIN as f32;
            let local_space_rect = Rect::from_x_y_ranges(x_range_f32, y_range);
            let to_screen = RectTransform::from_to(local_space_rect, response.rect);
            let from_screen = to_screen.inverse();

            let (track_foreground_color, track_background_color) =
                ColorSchemeConverter::to_color32(self.color_scheme);

            let interact_position = if let Some(click_pos) = ui.ctx().pointer_interact_pos() {
                let local_pos = from_screen * click_pos;
                Some(MusicalTime::new_with_units(local_pos.x as usize))
            } else {
                None
            };

            let visuals = if ui.is_enabled() {
                ui.ctx().style().visuals.widgets.active
            } else {
                ui.ctx().style().visuals.widgets.inactive
            };

            let is_control_down = ui.ctx().input(|i| i.modifiers.command_only());

            let mut is_clicked_area_arranged = false;

            let mut selection_set: SelectionSet<ArrangementUid> = ui
                .memory(|mem| mem.data.get_temp(response.id))
                .unwrap_or_default();
            // Since the last time this widget was instantiated, a new
            // arrangement was added. We should select it.
            if let Some(arrangement_uid) = self.new_arrangement_to_select {
                selection_set.click(&arrangement_uid, false);
            }

            // Generate all the pattern note shapes
            let arrangement_uids = self
                .composer
                .tracks_to_ordered_arrangement_uids
                .get(&self.track_uid)
                .cloned()
                .unwrap_or_default();
            let (pattern_backgrounds, pattern_shapes): (Vec<Shape>, Vec<Shape>) =
                arrangement_uids.iter().fold(
                    (Vec::default(), Vec::default()),
                    |(mut background_v, mut shape_v), arrangement_uid| {
                        if let Some(arrangement) = self.composer.arrangements.get(&arrangement_uid)
                        {
                            if let Some(pattern) =
                                self.composer.patterns.get(&arrangement.pattern_uid)
                            {
                                let arrangement_extent = arrangement.position
                                    ..arrangement.position + arrangement.duration;
                                if let Some(position) = interact_position {
                                    if arrangement_extent.contains(&position) {
                                        if response.clicked() {
                                            selection_set.click(arrangement_uid, is_control_down);
                                            *self.action =
                                                Some(ArrangementWidgetAction::ClearEditPattern);
                                            is_clicked_area_arranged = true;
                                        }
                                        if response.double_clicked() {
                                            *self.action =
                                                Some(ArrangementWidgetAction::SetEditPattern(
                                                    arrangement.pattern_uid,
                                                ));
                                            is_clicked_area_arranged = true;
                                        }
                                        if response.dragged() {
                                            response.dnd_set_drag_payload(
                                                TrackSource::ArrangementUid(*arrangement_uid),
                                            );
                                        }
                                    }
                                }
                                let is_selected = selection_set.contains(arrangement_uid);
                                background_v.push(Self::background_for_arrangement(
                                    &to_screen,
                                    &visuals,
                                    if is_selected {
                                        Color32::YELLOW
                                    } else {
                                        track_background_color
                                    },
                                    arrangement_extent,
                                ));
                                pattern.notes().iter().for_each(|note| {
                                    let note = Note::new_with_start_and_end(
                                        note.key,
                                        note.extent.0.start + arrangement.position,
                                        note.extent.0.end + arrangement.position,
                                    );
                                    shape_v.push(Self::shape_for_note(
                                        &to_screen,
                                        &visuals,
                                        track_foreground_color,
                                        &note,
                                    ));
                                });

                                // If this arrangement is selected, and the user
                                // presses a key, then we should handle it.
                                if is_selected {
                                    ui.ctx().input_mut(|i| {
                                        if i.consume_key(
                                            Modifiers::default(),
                                            eframe::egui::Key::Delete,
                                        ) {
                                            *self.action =
                                                Some(ArrangementWidgetAction::Unarrange(
                                                    *arrangement_uid,
                                                ));
                                        } else if i
                                            .consume_key(Modifiers::COMMAND, eframe::egui::Key::D)
                                        {
                                            *self.action =
                                                Some(ArrangementWidgetAction::Duplicate(
                                                    *arrangement_uid,
                                                ));
                                        }
                                    });
                                }
                            }
                        }
                        (background_v, shape_v)
                    },
                );

            // Click on empty space = add new arranged pattern
            if response.clicked() && !is_clicked_area_arranged {
                if let Some(position) = interact_position {
                    *self.action = Some(ArrangementWidgetAction::AddPattern(position));
                }
            }

            ui.memory_mut(|mem| mem.data.insert_temp(response.id, selection_set));

            // Paint all the shapes
            painter.extend(pattern_backgrounds);
            painter.extend(pattern_shapes);

            response
        })
        .inner
    }
}
impl<'a> ArrangementWidget<'a> {
    fn new(
        track_uid: TrackUid,
        composer: &'a mut Composer,
        view_range: &'a ViewRange,
        color_scheme: ColorScheme,
        new_arrangement_to_select: Option<ArrangementUid>,
        action: &'a mut Option<ArrangementWidgetAction>,
    ) -> Self {
        Self {
            track_uid,
            composer,
            view_range,
            color_scheme,
            new_arrangement_to_select,
            action,
        }
    }

    /// Instantiates a widget suitable for adding to a [Ui](eframe::egui::Ui).
    pub fn widget(
        track_uid: TrackUid,
        composer: &'a mut Composer,
        view_range: &'a ViewRange,
        color_scheme: ColorScheme,
        new_arrangement_to_select: Option<ArrangementUid>,
        action: &'a mut Option<ArrangementWidgetAction>,
    ) -> impl eframe::egui::Widget + 'a {
        move |ui: &mut eframe::egui::Ui| {
            ArrangementWidget::new(
                track_uid,
                composer,
                view_range,
                color_scheme,
                new_arrangement_to_select,
                action,
            )
            .ui(ui)
        }
    }

    fn shape_for_note(
        to_screen: &RectTransform,
        visuals: &WidgetVisuals,
        foreground_color: Color32,
        note: &Note,
    ) -> Shape {
        let a = to_screen * pos2(note.extent.0.start.total_units() as f32, note.key as f32);
        let b = to_screen * pos2(note.extent.0.end.total_units() as f32, note.key as f32);
        Shape::Rect(RectShape::new(
            Rect::from_two_pos(a, b),
            visuals.rounding,
            foreground_color,
            Stroke {
                color: foreground_color,
                width: visuals.fg_stroke.width,
            },
        ))
    }

    fn background_for_arrangement(
        to_screen: &RectTransform,
        visuals: &WidgetVisuals,
        background_color: Color32,
        time_range: Range<MusicalTime>,
    ) -> Shape {
        let upper_left = to_screen * pos2(time_range.start.total_units() as f32, 0.0);
        let bottom_right = to_screen * pos2(time_range.end.total_units() as f32, 127.0);
        Shape::Rect(RectShape::new(
            Rect::from_two_pos(upper_left, bottom_right),
            visuals.rounding,
            background_color,
            visuals.fg_stroke,
        ))
    }
}
