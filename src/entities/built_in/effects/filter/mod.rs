// Copyright (c) 2024 Mike Tsao

use crate::{
    cores::{
        BiQuadFilterAllPassCore, BiQuadFilterBandPassCore, BiQuadFilterBandStopCore,
        BiQuadFilterHighPassCore, BiQuadFilterLowPass24dbCore,
    },
    prelude::*,
};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// Entity wrapper for [BiQuadFilterBandPass]
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
#[entity(HandlesMidi, GeneratesStereoSample, Controls, SkipInner)]
pub struct BiQuadFilterBandPass {
    uid: Uid,
    inner: BiQuadFilterBandPassCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<crate::egui::BiQuadFilterWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl BiQuadFilterBandPass {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: BiQuadFilterBandPassCore) -> Self {
        Self {
            uid,
            inner,
            #[cfg(feature = "egui")]
            widget_action: Default::default(),
            #[cfg(feature = "egui")]
            action: Default::default(),
        }
    }
}

/// Entity wrapper for [BiQuadFilterBandStop]
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
#[entity(HandlesMidi, GeneratesStereoSample, Controls, SkipInner)]
pub struct BiQuadFilterBandStop {
    uid: Uid,
    inner: BiQuadFilterBandStopCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<crate::egui::BiQuadFilterWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl BiQuadFilterBandStop {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: BiQuadFilterBandStopCore) -> Self {
        Self {
            uid,
            inner,
            #[cfg(feature = "egui")]
            widget_action: Default::default(),
            #[cfg(feature = "egui")]
            action: Default::default(),
        }
    }
}

/// Entity wrapper for [BiQuadFilterLowPass24db]
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
#[entity(HandlesMidi, GeneratesStereoSample, Controls, SkipInner)]
pub struct BiQuadFilterLowPass24db {
    uid: Uid,
    inner: BiQuadFilterLowPass24dbCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<crate::egui::BiQuadFilterWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl BiQuadFilterLowPass24db {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: BiQuadFilterLowPass24dbCore) -> Self {
        Self {
            uid,
            inner,
            #[cfg(feature = "egui")]
            widget_action: Default::default(),
            #[cfg(feature = "egui")]
            action: Default::default(),
        }
    }
}

/// Entity wrapper for [BiQuadFilterHighPass]
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
#[entity(HandlesMidi, GeneratesStereoSample, Controls, SkipInner)]
pub struct BiQuadFilterHighPass {
    uid: Uid,
    inner: BiQuadFilterHighPassCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<crate::egui::BiQuadFilterWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl BiQuadFilterHighPass {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: BiQuadFilterHighPassCore) -> Self {
        Self {
            uid,
            inner,
            #[cfg(feature = "egui")]
            widget_action: Default::default(),
            #[cfg(feature = "egui")]
            action: Default::default(),
        }
    }
}

/// Entity wrapper for [BiQuadFilterAllPass]
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
#[entity(HandlesMidi, GeneratesStereoSample, Controls, SkipInner)]
pub struct BiQuadFilterAllPass {
    uid: Uid,
    inner: BiQuadFilterAllPassCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<crate::egui::BiQuadFilterWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl BiQuadFilterAllPass {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: BiQuadFilterAllPassCore) -> Self {
        Self {
            uid,
            inner,
            #[cfg(feature = "egui")]
            widget_action: Default::default(),
            #[cfg(feature = "egui")]
            action: Default::default(),
        }
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for BiQuadFilterBandPass {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut action = None;
        let response = ui.add(crate::egui::BiQuadFilterBandPassWidget::widget(
            &mut self.inner,
            &mut action,
        ));
        response
    }

    fn set_action(&mut self, action: DisplaysAction) {
        self.action = Some(action);
    }

    fn take_action(&mut self) -> Option<DisplaysAction> {
        self.action.take()
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for BiQuadFilterBandStop {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut action = None;
        let response = ui.add(crate::egui::BiQuadFilterBandStopWidget::widget(
            &mut self.inner,
            &mut action,
        ));
        response
    }

    fn set_action(&mut self, action: DisplaysAction) {
        self.action = Some(action);
    }

    fn take_action(&mut self) -> Option<DisplaysAction> {
        self.action.take()
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for BiQuadFilterHighPass {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut action = None;
        let response = ui.add(crate::egui::BiQuadFilterHighPassWidget::widget(
            &mut self.inner,
            &mut action,
        ));
        response
    }

    fn set_action(&mut self, action: DisplaysAction) {
        self.action = Some(action);
    }

    fn take_action(&mut self) -> Option<DisplaysAction> {
        self.action.take()
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for BiQuadFilterLowPass24db {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut action = None;
        let response = ui.add(crate::egui::BiQuadFilterLowPass24dbWidget::widget(
            &mut self.inner,
            &mut action,
        ));
        if let Some(action) = self.widget_action.take() {
            match action {
                crate::egui::BiQuadFilterWidgetAction::Link(uid, index) => {
                    self.set_action(DisplaysAction::Link(uid, index));
                }
            }
        }
        response
    }

    fn set_action(&mut self, action: DisplaysAction) {
        self.action = Some(action);
    }

    fn take_action(&mut self) -> Option<DisplaysAction> {
        self.action.take()
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for BiQuadFilterAllPass {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut action = None;
        let response = ui.add(crate::egui::BiQuadFilterAllPassWidget::widget(
            &mut self.inner,
            &mut action,
        ));
        response
    }

    fn set_action(&mut self, action: DisplaysAction) {
        self.action = Some(action);
    }

    fn take_action(&mut self) -> Option<DisplaysAction> {
        self.action.take()
    }
}
#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for BiQuadFilterBandPass {}
#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for BiQuadFilterBandStop {}
#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for BiQuadFilterHighPass {}
#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for BiQuadFilterLowPass24db {}
#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for BiQuadFilterAllPass {}
