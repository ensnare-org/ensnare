// Copyright (c) 2024 Mike Tsao

//! Widgets that work with the [egui](https://www.egui.rs/) GUI library.

/// A collection of imports that are useful for users of this crate who are also using egui.
pub mod prelude {
    pub use super::fill_remaining_ui_space;
}

// Public/reusable
pub use {
    audio::{
        analyze_spectrum, FrequencyDomainWidget, FrequencyWidget, TimeDomainWidget, WaveformWidget,
    },
    automation::{SignalPathWidget, SignalPathWidgetAction, TargetInstrument},
    chrome::{ControlBar, ControlBarAction, ControlBarWidget, TransportWidget},
    composition::{ComposerWidget, NoteLabeler, TimeLabeler},
    controllers::{ArpeggiatorWidget, LfoControllerWidget, NoteSequencerWidget},
    effects::{
        BiQuadFilterAllPassWidget, BiQuadFilterBandPassWidget, BiQuadFilterBandStopWidget,
        BiQuadFilterHighPassWidget, BiQuadFilterLowPass24dbWidget, BiQuadFilterWidgetAction,
    },
    entities::EntityPaletteWidget,
    generators::{EnvelopeWidget, LfoWidget, OscillatorWidget},
    glue::DragNormalWidget,
    instruments::{
        drumkit::{DrumkitWidget, DrumkitWidgetAction},
        fm::{FmSynthWidget, FmSynthWidgetAction},
        sampler::{SamplerWidget, SamplerWidgetAction},
        subtractive::{SubtractiveSynthWidget, SubtractiveSynthWidgetAction},
    },
    misc::ObliqueStrategiesWidget,
    modulators::{DcaWidget, DcaWidgetAction},
    project::{ProjectAction, ProjectWidget},
    settings::{AudioSettingsWidget, MidiSettingsWidget},
    util::fill_remaining_ui_space,
};

/// Exported only for widget explorer example.
// TODO maybe replace with a sneaky factory
pub mod widget_explorer {
    pub use super::{
        grid::GridWidget,
        legend::LegendWidget,
        placeholders::Wiggler,
        track::{make_title_bar_galley, TitleBarWidget},
    };
}

// Used only by other widgets
pub(in crate::egui) use {
    // audio::{
    //     analyze_spectrum, FrequencyDomainWidget, FrequencyWidget, TimeDomainWidget, WaveformWidget,
    // },
    grid::GridWidget,
    indicators::activity_indicator,
    legend::LegendWidget,
};

mod audio;
mod automation;
mod chrome;
mod colors;
mod composition;
mod controllers;
mod cursor;
mod effects;
mod entities;
mod generators;
mod glue;
mod grid;
mod indicators;
mod instruments;
mod legend;
mod misc;
mod modulators;
mod placeholders;
mod project;
mod settings;
mod signal_chain;
mod track;
mod util;
