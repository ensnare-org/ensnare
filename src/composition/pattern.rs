// Copyright (c) 2024 Mike Tsao

use crate::{
    prelude::*,
    types::{ColorScheme, IsUid, MidiEvent},
};
use anyhow::anyhow;
use delegate::delegate;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use synonym::Synonym;

/// Identifies a [Pattern].
#[derive(Synonym, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PatternUid(pub usize);
impl IsUid for PatternUid {
    fn as_usize(&self) -> usize {
        self.0
    }
}

/// Creates unique [PatternUid]s.
#[derive(Synonym, Debug, Serialize, Deserialize)]
pub struct PatternUidFactory(UidFactory<PatternUid>);
impl Default for PatternUidFactory {
    fn default() -> Self {
        Self(UidFactory::<PatternUid>::new(131072))
    }
}
impl PatternUidFactory {
    delegate! {
        to self.0 {
            /// Creates the next unique uid.
            pub fn mint_next(&self) -> PatternUid;
        }
    }
}

/// A [Pattern] contains a musical sequence that is suitable for
/// pattern-based composition. It is a series of [Note]s and a
/// [TimeSignature]. All the notes should fit into the pattern's duration, and
/// the duration should be a round multiple of the length implied by the time
/// signature.
#[derive(Clone, Debug, Builder, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(build_fn(private, name = "build_from_builder"))]
pub struct Pattern {
    /// The pattern's [TimeSignature].
    #[builder(default)]
    time_signature: TimeSignature,

    /// The notes that make up this pattern. When it is in a [Pattern], a
    /// [Note]'s range is relative to the start of the [Pattern]. For example, a
    /// note that plays immediately would have a range start of zero. TODO:
    /// specify any ordering restrictions.
    #[builder(default, setter(each(name = "note", into)))]
    pub notes: Vec<Note>,

    /// The [TimeRange] that this [Pattern] covers. The extent origin is always
    /// [MusicalTime::TIME_ZERO].
    #[builder(setter(skip))]
    pub extent: TimeRange,

    /// The [ColorScheme] of this [Pattern]. TODO: would this be better as a map
    /// maintained by whoever cares about the color? It's not something that
    /// everyone who uses [Pattern] would care about.
    #[builder(default)]
    #[serde(skip)]
    pub color_scheme: ColorScheme,
}
impl PatternBuilder {
    /// The length of a note generated by the random() method
    pub const DURATION: MusicalTime = MusicalTime::DURATION_QUARTER;

    /// Builds the [Pattern].
    pub fn build(&self) -> Result<Pattern, PatternBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }

    /// Generates a random [Pattern] that's probably useful for prototyping and
    /// testing. Clamped to number of divisions implied by time signature.
    pub fn random(&mut self, rng: &mut Rng) -> &mut Self {
        for _ in 0..rng.rand_range(8..16) {
            let time_signature = self.time_signature.unwrap_or_default();
            let ts_top = time_signature.top as u64;
            let ts_bottom = time_signature.bottom as u64;
            let start_beat = rng.rand_range(0..ts_top);
            let start_division = rng.rand_range(0..ts_bottom);
            let start =
                MusicalTime::new_with_parts((start_beat * ts_bottom + start_division) as usize * 4);
            let duration = Self::DURATION;
            self.note(Note::new_with(
                rng.rand_range(32..96) as u8,
                start,
                duration,
            ));
        }
        self
    }

    /// Given a sequence of MIDI note numbers and an optional grid value that
    /// overrides the one implied by the time signature, adds [Note]s one after
    /// another into the pattern. The value 255 is reserved for rest (no note,
    /// or silence).
    ///
    /// The optional grid_value is similar to the time signature's bottom value
    /// (1 is a whole note, 2 is a half note, etc.). For example, for a 4/4
    /// pattern, None means each note number produces a quarter note, and we
    /// would provide sixteen note numbers to fill the pattern with 4 beats of
    /// four quarter-notes each. For a 4/4 pattern, Some(8) means each note
    /// number should produce an eighth note., and 4 x 8 = 32 note numbers would
    /// fill the pattern.
    ///
    /// If midi_note_numbers contains fewer than the maximum number of note
    /// numbers for the grid value, then the rest of the pattern is silent.
    pub fn note_sequence(
        &mut self,
        midi_note_numbers: Vec<u8>,
        grid_value: Option<usize>,
    ) -> &mut Self {
        let grid_value = grid_value.unwrap_or(self.time_signature.unwrap_or_default().bottom);
        let mut position = MusicalTime::START;
        let position_delta = MusicalTime::new_with_fractional_beats(1.0 / grid_value as f64);
        for note in midi_note_numbers {
            if note != 255 {
                self.note(Note::new_with(note, position, position_delta));
            }
            position += position_delta;
        }
        self
    }
}
impl Default for Pattern {
    fn default() -> Self {
        let mut r = Self {
            time_signature: Default::default(),
            notes: Default::default(),
            extent: Default::default(),
            color_scheme: Default::default(),
        };
        r.after_deser();
        r
    }
}
impl Serializable for Pattern {
    fn after_deser(&mut self) {
        self.refresh_internals();
    }
}
impl HasExtent for Pattern {
    fn extent(&self) -> TimeRange {
        self.extent.clone()
    }

    fn set_extent(&mut self, extent: TimeRange) {
        self.extent = extent;
    }
}
impl Into<Vec<MidiEvent>> for Pattern {
    fn into(self) -> Vec<MidiEvent> {
        self.notes.iter().fold(Vec::default(), |mut v, note| {
            let mut note_as_events: Vec<MidiEvent> = note.clone().into();
            v.append(&mut note_as_events);
            v
        })
    }
}

impl Pattern {
    /// Returns the number of notes in the pattern.
    pub fn note_count(&self) -> usize {
        self.notes.len()
    }

    /// Returns the pattern grid's number of subdivisions, which is calculated
    /// from the time signature. The number is simply the time signature's top x
    /// bottom. For example, a 3/4 pattern will have 12 subdivisions (three
    /// beats per measure, each beat divided into four quarter notes).
    ///
    /// This is just a UI default and doesn't affect the actual granularity of a
    /// note position.
    pub fn default_grid_value(&self) -> usize {
        self.time_signature.top * self.time_signature.bottom
    }

    fn refresh_internals(&mut self) {
        let final_event_time = self
            .notes
            .iter()
            .map(|n| n.extent.0.end)
            .max()
            .unwrap_or_default();

        // This is how we deal with Range<> being inclusive start, exclusive
        // end. It matters because we want the calculated duration to be rounded
        // up to the next measure, but we don't want a note-off event right on
        // the edge to extend that calculation to include another bar.
        let final_event_time = if final_event_time == MusicalTime::START {
            final_event_time
        } else {
            final_event_time - MusicalTime::ONE_UNIT
        };
        let beats = final_event_time.total_beats();
        let top = self.time_signature.top;
        let rounded_up_bars = (beats + top) / top;
        self.extent = TimeRange(
            MusicalTime::START..MusicalTime::new_with_bars(&self.time_signature, rounded_up_bars),
        );
    }

    /// Adds a note to this pattern. Does not check for duplicates. It's OK to
    /// add notes in any time order.
    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
        self.refresh_internals();
    }

    /// Removes all notes matching this one in this pattern.
    pub fn remove_note(&mut self, note: &Note) {
        self.notes.retain(|v| v != note);
        self.refresh_internals();
    }

    /// Adds a note if it doesn't already exist; removes it if it does.
    pub fn toggle_note(&mut self, note: Note) {
        if self.notes.contains(&note) {
            self.remove_note(&note);
        } else {
            self.add_note(note);
        }
    }

    /// Removes all notes in this pattern.
    pub fn clear(&mut self) {
        self.notes.clear();
        self.refresh_internals();
    }

    /// Sets a new start time for all notes in the Pattern matching the given
    /// [Note]. If any are found, returns the new version.
    pub fn move_note(&mut self, note: &Note, new_start: MusicalTime) -> anyhow::Result<Note> {
        let mut new_note = note.clone();
        let new_note_length = new_note.extent.0.end - new_note.extent.0.start;
        new_note.extent = TimeRange(new_start..new_start + new_note_length);
        self.replace_note(note, new_note)
    }

    /// Sets a new start time and duration for all notes in the Pattern matching
    /// the given [Note]. If any are found, returns the new version.
    pub fn move_and_resize_note(
        &mut self,
        note: &Note,
        new_start: MusicalTime,
        duration: MusicalTime,
    ) -> anyhow::Result<Note> {
        let mut new_note = note.clone();
        new_note.extent = TimeRange(new_start..new_start + duration);
        self.replace_note(note, new_note)
    }

    /// Sets a new key for all notes in the Pattern matching the given [Note].
    /// If any are found, returns the new version.
    pub fn change_note_key(&mut self, note: &Note, new_key: u8) -> anyhow::Result<Note> {
        let mut new_note = note.clone();
        new_note.key = new_key;
        self.replace_note(note, new_note)
    }

    /// Replaces all notes in the Pattern matching the given [Note] with a new
    /// [Note]. If any are found, returns the new version.
    pub fn replace_note(&mut self, note: &Note, new_note: Note) -> anyhow::Result<Note> {
        let mut found = false;

        self.notes.iter_mut().filter(|n| n == &note).for_each(|n| {
            *n = new_note.clone();
            found = true;
        });
        if found {
            self.refresh_internals();
            Ok(new_note)
        } else {
            Err(anyhow!("replace_note: couldn't find note {:?}", note))
        }
    }

    #[allow(missing_docs)]
    pub fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }

    /// Returns a read-only slice of all the [Note]s in this pattern. No order
    /// is currently defined.
    pub fn notes(&self) -> &[Note] {
        self.notes.as_ref()
    }

    // TODO: what is this?
    //
    // pub fn colors(&self) -> Option<(u8, u8)> { None }

    /// Adds to both start and end. This is less ambiguous than implementing
    /// `Add<MusicalTime>`, which could reasonably add only to the end.
    pub fn shift_right(&self, rhs: MusicalTime) -> Self {
        let mut r = self.clone();
        r.notes = self
            .notes
            .iter()
            .map(|note| note.shift_right(rhs))
            .collect();
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Note {
        /// half-note
        const TEST_C4: Note = Note::new_with(
            MidiNote::C4 as u8,
            MusicalTime::START,
            MusicalTime::DURATION_HALF,
        );
        /// whole note
        const TEST_D4: Note = Note::new_with(
            MidiNote::D4 as u8,
            MusicalTime::START,
            MusicalTime::DURATION_WHOLE,
        );
        /// two whole notes
        const TEST_E4: Note = Note::new_with(
            MidiNote::E4 as u8,
            MusicalTime::START,
            MusicalTime::DURATION_BREVE,
        );
    }

    #[test]
    fn pattern_defaults() {
        let p = Pattern::default();
        assert_eq!(p.note_count(), 0, "Default pattern should have zero notes");

        let p = PatternBuilder::default().build().unwrap();
        assert_eq!(
            p.note_count(),
            0,
            "Default built pattern should have zero notes"
        );

        assert_eq!(
            p.time_signature(),
            TimeSignature::COMMON_TIME,
            "Default built pattern should have 4/4 time signature"
        );

        assert_eq!(
            p.duration(),
            MusicalTime::new_with_bars(&TimeSignature::COMMON_TIME, 1),
            "Default built pattern's duration should be one measure"
        );
    }

    #[test]
    fn pattern_one_half_note_is_one_bar() {
        let mut p = PatternBuilder::default().build().unwrap();
        p.add_note(Note::TEST_C4);
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            1,
            "Pattern with one half-note should be 1 bar"
        );
    }

    #[test]
    fn pattern_one_breve_is_one_bar() {
        let mut p = PatternBuilder::default().build().unwrap();
        p.add_note(Note::TEST_E4);
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            1,
            "Pattern with one note of length breve should be 1 bar"
        );
    }

    #[test]
    fn pattern_one_long_note_is_one_bar() {
        let p = PatternBuilder::default()
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(0),
                MusicalTime::new_with_beats(4),
            ))
            .build()
            .unwrap();
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            1,
            "Pattern with a single bar-long note is one bar"
        );
    }

    #[test]
    fn pattern_one_beat_with_1_4_time_signature_is_one_bar() {
        let p = PatternBuilder::default()
            .time_signature(TimeSignature::new_with(1, 4).unwrap())
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(0),
                MusicalTime::ONE_BEAT,
            ))
            .build()
            .unwrap();
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            1,
            "Pattern with a single whole note in 1/4 time is one bar"
        );
    }

    #[test]
    fn pattern_three_half_notes_is_one_bar() {
        let p = PatternBuilder::default()
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(0),
                MusicalTime::DURATION_HALF,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::ONE_BEAT,
                MusicalTime::DURATION_HALF,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(2),
                MusicalTime::DURATION_HALF,
            ))
            .build()
            .unwrap();
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            1,
            "Pattern with three half-notes on beat should be 1 bar"
        );
    }

    #[test]
    fn pattern_four_whole_notes_is_one_bar() {
        let p = PatternBuilder::default()
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(0),
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::ONE_BEAT,
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(2),
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(3),
                MusicalTime::DURATION_WHOLE,
            ))
            .build()
            .unwrap();
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            1,
            "Pattern with four whole notes on beat should be 1 bar"
        );
    }

    #[test]
    fn pattern_five_notes_is_two_bars() {
        let p = PatternBuilder::default()
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(0),
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::ONE_BEAT,
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(2),
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(3),
                MusicalTime::DURATION_WHOLE,
            ))
            .note(Note::new_with_midi_note(
                MidiNote::C0,
                MusicalTime::new_with_beats(4),
                MusicalTime::DURATION_SIXTEENTH,
            ))
            .build()
            .unwrap();
        assert_eq!(
            p.duration().total_bars(&p.time_signature()),
            2,
            "Pattern with four whole notes and then a sixteenth should be 2 bars"
        );
    }

    #[test]
    fn default_pattern_builder() {
        let p = PatternBuilder::default().build().unwrap();
        assert_eq!(
            p.notes.len(),
            0,
            "Default PatternBuilder yields pattern with zero notes"
        );
        assert_eq!(
            p.duration(),
            MusicalTime::new_with_bars(&p.time_signature, 1),
            "Default PatternBuilder yields one-measure pattern"
        );
    }

    #[test]
    fn pattern_api_is_ergonomic() {
        let mut p = PatternBuilder::default()
            .note(Note::TEST_C4.clone())
            .note(Note::TEST_D4.clone())
            .build()
            .unwrap();
        assert_eq!(p.notes.len(), 2, "PatternBuilder can add multiple notes");

        p.add_note(Note::TEST_C4.clone());
        assert_eq!(
            p.notes.len(),
            3,
            "Pattern can add duplicate notes. This is probably not desirable to allow."
        );

        assert!(p
            .move_note(&Note::TEST_C4, MusicalTime::new_with_beats(4))
            .is_ok());
        assert_eq!(p.notes.len(), 3, "Moving a note doesn't copy or destroy");
        p.remove_note(&Note::TEST_D4);
        assert_eq!(p.notes.len(), 2, "remove_note() removes notes");
        p.remove_note(&Note::TEST_C4);
        assert_eq!(
            p.notes.len(),
            2,
            "remove_note() must specify the note correctly."
        );
        p.remove_note(&Note::new_with_midi_note(
            MidiNote::C4,
            MusicalTime::new_with_beats(4),
            MusicalTime::DURATION_HALF,
        ));
        assert!(p.notes.is_empty(), "remove_note() removes duplicate notes.");
    }

    #[test]
    fn move_note_inside_pattern() {
        let mut p = PatternBuilder::default().build().unwrap();

        p.add_note(Note::TEST_C4.clone());
        assert!(p
            .move_note(
                &Note::TEST_C4,
                MusicalTime::START + MusicalTime::DURATION_SIXTEENTH,
            )
            .is_ok());
        assert_eq!(
            p.notes[0].extent.0.start,
            MusicalTime::START + MusicalTime::DURATION_SIXTEENTH,
            "moving a note works"
        );
        assert_eq!(
            p.duration(),
            MusicalTime::new_with_beats(4),
            "Moving a note in pattern doesn't change duration"
        );

        assert!(
            p.move_note(&Note::TEST_E4, MusicalTime::default()).is_err(),
            "moving nonexistent note should fail"
        );
    }

    #[test]
    fn move_note_outside_pattern() {
        let mut p = PatternBuilder::default().build().unwrap();

        p.add_note(Note::TEST_C4.clone());
        assert!(p
            .move_note(&Note::TEST_C4, MusicalTime::new_with_beats(4))
            .is_ok());
        assert_eq!(
            p.duration(),
            MusicalTime::new_with_beats(4 * 2),
            "Moving a note out of pattern increases duration"
        );
    }

    #[test]
    fn move_and_resize_note() {
        let mut p = PatternBuilder::default().build().unwrap();

        p.add_note(Note::TEST_C4.clone());

        assert!(p
            .move_and_resize_note(
                &Note::TEST_C4,
                MusicalTime::START + MusicalTime::DURATION_EIGHTH,
                MusicalTime::DURATION_WHOLE,
            )
            .is_ok());
        let expected_range = TimeRange(
            (MusicalTime::START + MusicalTime::DURATION_EIGHTH)
                ..(MusicalTime::START + MusicalTime::DURATION_EIGHTH + MusicalTime::DURATION_WHOLE),
        );
        assert_eq!(
            p.notes[0].extent, expected_range,
            "moving/resizing a note works"
        );
        assert_eq!(
            p.duration(),
            MusicalTime::new_with_beats(4),
            "moving/resizing within pattern doesn't change duration"
        );

        assert!(p
            .move_and_resize_note(
                &Note::new_with_midi_note(
                    MidiNote::C4,
                    expected_range.0.start,
                    expected_range.0.end - expected_range.0.start,
                ),
                MusicalTime::new_with_beats(4),
                MusicalTime::DURATION_WHOLE,
            )
            .is_ok());
        assert_eq!(
            p.duration(),
            MusicalTime::new_with_beats(8),
            "moving/resizing outside current pattern makes the pattern longer"
        );

        assert!(
            p.move_and_resize_note(
                &Note::TEST_E4,
                MusicalTime::default(),
                MusicalTime::default()
            )
            .is_err(),
            "moving/resizing nonexistent note should fail"
        );
    }

    #[test]
    fn change_note_key() {
        let mut p = PatternBuilder::default().build().unwrap();

        p.add_note(Note::TEST_C4.clone());
        assert_eq!(p.notes[0].key, MidiNote::C4 as u8);
        assert!(p
            .change_note_key(&Note::TEST_C4, MidiNote::C5 as u8)
            .is_ok());
        assert_eq!(p.notes[0].key, MidiNote::C5 as u8);

        assert!(
            p.change_note_key(&Note::TEST_C4, 254).is_err(),
            "changing key of nonexistent note should fail"
        );
    }

    #[test]
    fn pattern_dimensions_are_valid() {
        let p = Pattern::default();
        assert_eq!(
            p.time_signature,
            TimeSignature::COMMON_TIME,
            "default pattern should have sensible time signature"
        );

        for ts in [
            TimeSignature::COMMON_TIME,
            TimeSignature::CUT_TIME,
            TimeSignature::new_with(7, 64).unwrap(),
        ] {
            let p = PatternBuilder::default()
                .time_signature(ts)
                .build()
                .unwrap();
            assert_eq!(
                p.duration(),
                MusicalTime::new_with_beats(ts.top),
                "Pattern's beat count matches its time signature"
            );

            // A typical 4/4 pattern has 16 subdivisions, which is a common
            // pattern resolution in other pattern-based sequencers and piano
            // rolls.
            assert_eq!(p.default_grid_value(), ts.bottom * ts.top,
                "Pattern's default grid value should be the time signature's beat count times its note value");
        }
    }

    #[test]
    fn pattern_note_insertion_is_easy() {
        let sixteen_notes = vec![
            60, 61, 62, 63, 64, 65, 66, 67, 60, 61, 62, 63, 64, 65, 66, 67,
        ];
        let len_16 = sixteen_notes.len();
        let p = PatternBuilder::default()
            .note_sequence(sixteen_notes, None)
            .build()
            .unwrap();
        assert_eq!(p.note_count(), len_16, "sixteen quarter notes");
        assert_eq!(p.notes[15].key, 67);
        assert_eq!(
            p.notes[15].extent,
            TimeRange(
                MusicalTime::DURATION_QUARTER * 15
                    ..MusicalTime::DURATION_WHOLE * p.time_signature.top
            )
        );
        assert_eq!(
            p.duration(),
            MusicalTime::DURATION_WHOLE * p.time_signature.top
        );

        let seventeen_notes = vec![
            60, 61, 62, 63, 64, 65, 66, 67, 60, 61, 62, 63, 64, 65, 66, 67, 68,
        ];
        let p = PatternBuilder::default()
            .note_sequence(seventeen_notes, None)
            .build()
            .unwrap();
        assert_eq!(
            p.duration(),
            MusicalTime::DURATION_WHOLE * p.time_signature.top * 2,
            "17 notes in 4/4 pattern produces two bars"
        );

        let four_notes = vec![60, 61, 62, 63];
        let len_4 = four_notes.len();
        let p = PatternBuilder::default()
            .note_sequence(four_notes, Some(4))
            .build()
            .unwrap();
        assert_eq!(p.note_count(), len_4, "four quarter notes");
        assert_eq!(
            p.duration(),
            MusicalTime::DURATION_WHOLE * p.time_signature.top
        );

        let three_notes_and_silence = vec![60, 0, 62, 63];
        let len_3_1 = three_notes_and_silence.len();
        let p = PatternBuilder::default()
            .note_sequence(three_notes_and_silence, Some(4))
            .build()
            .unwrap();
        assert_eq!(p.note_count(), len_3_1, "three quarter notes with one rest");
        assert_eq!(
            p.duration(),
            MusicalTime::DURATION_WHOLE * p.time_signature.top
        );

        let eight_notes = vec![60, 61, 62, 63, 64, 65, 66, 67];
        let len_8 = eight_notes.len();
        let p = PatternBuilder::default()
            .time_signature(TimeSignature::CUT_TIME)
            .note_sequence(eight_notes, None)
            .build()
            .unwrap();
        assert_eq!(
            p.note_count(),
            len_8,
            "eight eighth notes in 2/2 time is two bars long"
        );
        assert_eq!(
            p.duration(),
            MusicalTime::DURATION_WHOLE * p.time_signature.top * 2
        );

        let one_note = vec![60];
        let len_1 = one_note.len();
        let p = PatternBuilder::default()
            .note_sequence(one_note, None)
            .build()
            .unwrap();
        assert_eq!(
            p.note_count(),
            len_1,
            "one quarter note, and the rest is silence"
        );
        assert_eq!(p.notes[0].key, 60);
        assert_eq!(
            p.notes[0].extent,
            TimeRange(MusicalTime::START..MusicalTime::DURATION_QUARTER)
        );
        assert_eq!(
            p.duration(),
            MusicalTime::DURATION_WHOLE * p.time_signature.top
        );
    }

    #[test]
    fn cut_time_duration() {
        let p = PatternBuilder::default()
            .time_signature(TimeSignature::CUT_TIME)
            .build()
            .unwrap();
        assert_eq!(p.duration(), MusicalTime::new_with_beats(2));
    }
}
