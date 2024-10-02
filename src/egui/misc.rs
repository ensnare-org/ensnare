// Copyright (c) 2024 Mike Tsao

use eframe::egui::Layout;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// A fun egui widget that outputs oblique strategies.
pub struct ObliqueStrategiesWidget {
    seed: Option<usize>,

    // When to pick a new seed.
    next_reseed: Instant,

    // A random number generator.
    rng: oorandom::Rand64,
}
impl ObliqueStrategiesWidget {
    /// This is meant to be called on each egui render, so it's cheap. If it
    /// returns Some(seed), then draw the widget with that seed. If it returns
    /// None, then don't draw the widget (or pass None to the widget if you
    /// don't want to worry about the layout jumping around).
    pub fn check_seed(&mut self) -> Option<usize> {
        let now = Instant::now();
        if self.next_reseed <= now {
            self.seed = if self.rng.rand_float() > 0.2 {
                Some(self.rng.rand_u64() as usize)
            } else {
                None
            };
            self.set_next_reseed_time(30.0, 3600.0);
        }
        self.seed
    }

    fn set_next_reseed_time(&mut self, min_seconds: f64, max_seconds: f64) {
        self.next_reseed = Instant::now()
            + Duration::from_secs_f64(
                min_seconds + self.rng.rand_float() * (max_seconds - min_seconds),
            );
    }

    /// Displays an [Oblique
    /// Strategy](https://en.wikipedia.org/wiki/Oblique_Strategies) to promote
    /// creativity. Thank you, Brian Eno and Peter Schmidt.
    ///
    /// A given seed will always display the same strategy, but not every seed
    /// corresponds to a unique strategy.
    pub fn widget(seed: Option<usize>) -> impl eframe::egui::Widget + 'static {
        move |ui: &mut eframe::egui::Ui| {
            let strategies = vec![
                "", // This should remain the first. It indicates that we want to draw nothing.
            "Abandon normal instruments",
            "Accept advice",
            "Accretion",
            "A line has two sides",
            "Allow an easement (an easement is the abandonment of a stricture)",
            "Are there sections? Consider transitions",
            "Ask people to work against their better judgment",
            "Ask your body",
            "Assemble some of the instruments in a group and treat the group",
            "Balance the consistency principle with the inconsistency principle",
            "Be dirty",
            "Breathe more deeply",
            "Bridges •build •burn",
            "Cascades",
            "Change instrument roles",
            "Change nothing and continue with immaculate consistency",
            "Children's voices •speaking •singing",
            "Cluster analysis",
            "Consider different fading systems",
            "Consult other sources •promising •unpromising",
            "Convert a melodic element into a rhythmic element",
            "Courage!",
            "Cut a vital connection",
            "Decorate, decorate",
            "Define an area as 'safe' and use it as an anchor",
            "Destroy •nothing •the most important thing",
            "Discard an axiom",
            "Disconnect from desire",
            "Discover the recipes you are using and abandon them",
            "Distorting time",
            "Do nothing for as long as possible",
            "Don't be afraid of things because they're easy to do",
            "Don't be frightened of cliches",
            "Don't be frightened to display your talents",
            "Don't break the silence",
            "Don't stress one thing more than another",
            "Do something boring",
            "Do the washing up",
            "Do the words need changing?",
            "Do we need holes?",
            "Emphasize differences",
            "Emphasize repetitions",
            "Emphasize the flaws",
            "Faced with a choice, do both (given by Dieter Roth)",
            "Feedback recordings into an acoustic situation",
            "Fill every beat with something",
            "Get your neck massaged",
            "Ghost echoes",
            "Give the game away",
            "Give way to your worst impulse",
            "Go slowly all the way round the outside",
            "Honor thy error as a hidden intention",
            "How would you have done it?",
            "Humanize something free of error",
            "Imagine the music as a moving chain or caterpillar",
            "Imagine the music as a set of disconnected events",
            "Infinitesimal gradations",
            "Intentions •credibility of •nobility of •humility of",
            "Into the impossible",
            "Is it finished?",
            "Is there something missing?",
            "Is the tuning appropriate?",
            "Just carry on",
            "Left channel, right channel, center channel",
            "Listen in total darkness, or in a very large room, very quietly",
            "Listen to the quiet voice",
            "Look at a very small object; look at its center",
            "Look at the order in which you do things",
            "Look closely at the most embarrassing details and amplify them",
            "Lowest common denominator check •single beat •single note •single",
            "riff",
            "Make a blank valuable by putting it in an exquisite frame",
            "Make an exhaustive list of everything you might do and do the last",
            "thing on the list",
            "Make a sudden, destructive, unpredictable action; incorporate",
            "Mechanicalize something idiosyncratic",
            "Mute and continue",
            "Only one element of each kind",
            "(Organic) machinery",
            "Overtly resist change",
            "Put in earplugs",
            "Remember those quiet evenings",
            "Remove ambiguities and convert to specifics",
            "Remove specifics and convert to ambiguities",
            "Repetition is a form of change",
            "Reverse",
            "Short circuit (a man eating peas with the idea that they will improve his virility shovels them straight into his lap)",
            "Shut the door and listen from outside",
            "Simple subtraction",
            "Spectrum analysis",
            "Take a break",
            "Take away the elements in order of apparent non-importance",
            "Tape your mouth (given by Ritva Saarikko)",
            "The inconsistency principle",
            "The tape is now the music",
            "Think of the radio",
            "Tidy up",
            "Trust in the you of now",
            "Turn it upside down",
            "Twist the spine",
            "Use an old idea",
            "Use an unacceptable color",
            "Use fewer notes",
            "Use filters",
            "Use \"unqualified\" people",
            "Water",
            "What are you really thinking about just now? Incorporate",
            "What is the reality of the situation?",
            "What mistakes did you make last time?",
            "What would your closest friend do?",
            "What wouldn't you do?",
            "Work at a different speed",
            "You are an engineer",
            "You can only make one dot at a time",
            "You don't have to be ashamed of using your own ideas",
            "[This space intentionally left blank]",
        ];
            ui.with_layout(
                Layout::centered_and_justified(eframe::egui::Direction::LeftToRight),
                |ui| ui.label(strategies[seed.unwrap_or_default() % strategies.len()]),
            )
            .inner
        }
    }
}
impl Default for ObliqueStrategiesWidget {
    fn default() -> Self {
        let mut r = Self {
            seed: Default::default(),
            next_reseed: Instant::now(),
            rng: oorandom::Rand64::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
            ),
        };
        r.set_next_reseed_time(90.0, 300.0);
        r
    }
}
