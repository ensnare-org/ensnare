// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `pocket-calculator` example is a simple groovebox. It demonstrates using
//! the `ensnare` crate without a [Project].

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::anyhow;
use calculator::Calculator;
use eframe::CreationContext;
use ensnare::{prelude::*, types::AudioQueue};
use std::sync::{Arc, Mutex};

mod calculator;

struct CalculatorApp {
    calculator: Arc<Mutex<Calculator>>,
    audio_service: AudioService,
    audio_queue: Option<AudioQueue>,
}
impl CalculatorApp {
    const APP_NAME: &'static str = "Pocket Calculator";

    fn new(_cc: &CreationContext) -> Self {
        Self {
            calculator: Arc::new(Mutex::new(Calculator::default())),
            audio_service: AudioService::new(),
            audio_queue: None,
        }
    }
}
impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(event) = self.audio_service.receiver().try_recv() {
            {
                match event {
                    AudioServiceEvent::Reset(sample_rate, _channel_count, audio_queue) => {
                        self.calculator
                            .lock()
                            .unwrap()
                            .update_sample_rate(sample_rate);
                        self.audio_queue = Some(audio_queue);
                    }
                    AudioServiceEvent::NeedsAudio(count) => {
                        let mut buffer = [StereoSample::SILENCE; 64];

                        for _ in 0..(count / buffer.len()) + 1 {
                            if let Ok(mut calculator) = self.calculator.lock() {
                                // This is a lot of redundant calculation for something that
                                // doesn't change much, but it's cheap.
                                let range = TimeRange(
                                    MusicalTime::START
                                        ..MusicalTime::new_with_units(
                                            MusicalTime::frames_to_units(
                                                calculator.tempo(),
                                                calculator.sample_rate(),
                                                buffer.len(),
                                            ),
                                        ),
                                );

                                calculator.update_time_range(&range);
                                calculator.work(&mut |_| {});
                                calculator.generate(&mut buffer);
                                if let Some(queue) = self.audio_queue.as_ref() {
                                    buffer.iter().for_each(|s| {
                                        let _ = queue.force_push(*s);
                                    });
                                }
                            }
                        }
                    }
                    AudioServiceEvent::Underrun => {
                        eprintln!("AudioServiceEvent::Underrun");
                    }
                }
            }
        }
        let center = eframe::egui::CentralPanel::default();

        center.show(ctx, |ui| {
            if let Ok(mut calculator) = self.calculator.lock() {
                calculator.ui(ui);
            }
        });

        // We're being lazy and always requesting a repaint, even though we
        // don't know whether anything changed on-screen.
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.audio_service.sender().send(AudioServiceInput::Quit);
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Pocket Calculator")
            .with_inner_size(eframe::epaint::vec2(348.0, 576.0))
            .to_owned(),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        CalculatorApp::APP_NAME,
        options,
        Box::new(|cc| Box::new(CalculatorApp::new(cc))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}
