// Copyright (c) 2024 Mike Tsao

use strum_macros::Display;

/// The General MIDI instruments. <https://en.wikipedia.org/wiki/General_MIDI>
#[allow(missing_docs)]
#[derive(Display, Debug)]
pub enum GeneralMidiProgram {
    AcousticGrand = 0,
    BrightAcoustic = 1,
    ElectricGrand = 2,
    HonkyTonk = 3,
    ElectricPiano1 = 4,
    ElectricPiano2 = 5,
    Harpsichord = 6,
    Clav = 7,
    Celesta = 8,
    Glockenspiel = 9,
    MusicBox = 10,
    Vibraphone = 11,
    Marimba = 12,
    Xylophone = 13,
    TubularBells = 14,
    Dulcimer = 15,
    DrawbarOrgan = 16,
    PercussiveOrgan = 17,
    RockOrgan = 18,
    ChurchOrgan = 19,
    ReedOrgan = 20,
    Accordion = 21,
    Harmonica = 22,
    TangoAccordion = 23,
    AcousticGuitarNylon = 24,
    AcousticGuitarSteel = 25,
    ElectricGuitarJazz = 26,
    ElectricGuitarClean = 27,
    ElectricGuitarMuted = 28,
    OverdrivenGuitar = 29,
    DistortionGuitar = 30,
    GuitarHarmonics = 31,
    AcousticBass = 32,
    ElectricBassFinger = 33,
    ElectricBassPick = 34,
    FretlessBass = 35,
    SlapBass1 = 36,
    SlapBass2 = 37,
    SynthBass1 = 38,
    SynthBass2 = 39,
    Violin = 40,
    Viola = 41,
    Cello = 42,
    Contrabass = 43,
    TremoloStrings = 44,
    PizzicatoStrings = 45,
    OrchestralHarp = 46,
    Timpani = 47,
    StringEnsemble1 = 48,
    StringEnsemble2 = 49,
    Synthstrings1 = 50,
    Synthstrings2 = 51,
    ChoirAahs = 52,
    VoiceOohs = 53,
    SynthVoice = 54,
    OrchestraHit = 55,
    Trumpet = 56,
    Trombone = 57,
    Tuba = 58,
    MutedTrumpet = 59,
    FrenchHorn = 60,
    BrassSection = 61,
    Synthbrass1 = 62,
    Synthbrass2 = 63,
    SopranoSax = 64,
    AltoSax = 65,
    TenorSax = 66,
    BaritoneSax = 67,
    Oboe = 68,
    EnglishHorn = 69,
    Bassoon = 70,
    Clarinet = 71,
    Piccolo = 72,
    Flute = 73,
    Recorder = 74,
    PanFlute = 75,
    BlownBottle = 76,
    Shakuhachi = 77,
    Whistle = 78,
    Ocarina = 79,
    Lead1Square = 80,
    Lead2Sawtooth = 81,
    Lead3Calliope = 82,
    Lead4Chiff = 83,
    Lead5Charang = 84,
    Lead6Voice = 85,
    Lead7Fifths = 86,
    Lead8BassLead = 87,
    Pad1NewAge = 88,
    Pad2Warm = 89,
    Pad3Polysynth = 90,
    Pad4Choir = 91,
    Pad5Bowed = 92,
    Pad6Metallic = 93,
    Pad7Halo = 94,
    Pad8Sweep = 95,
    Fx1Rain = 96,
    Fx2Soundtrack = 97,
    Fx3Crystal = 98,
    Fx4Atmosphere = 99,
    Fx5Brightness = 100,
    Fx6Goblins = 101,
    Fx7Echoes = 102,
    Fx8SciFi = 103,
    Sitar = 104,
    Banjo = 105,
    Shamisen = 106,
    Koto = 107,
    Kalimba = 108,
    Bagpipe = 109,
    Fiddle = 110,
    Shanai = 111,
    TinkleBell = 112,
    Agogo = 113,
    SteelDrums = 114,
    Woodblock = 115,
    TaikoDrum = 116,
    MelodicTom = 117,
    SynthDrum = 118,
    ReverseCymbal = 119,
    GuitarFretNoise = 120,
    BreathNoise = 121,
    Seashore = 122,
    BirdTweet = 123,
    TelephoneRing = 124,
    Helicopter = 125,
    Applause = 126,
    Gunshot = 127,
}

/// The General MIDI percussion instruments.
/// <https://en.wikipedia.org/wiki/General_MIDI#Percussion>
///
/// These numbers correspond to MIDI notes that assume a certain sound-per-key
/// set is loaded. For example, AcousticBassDrum is MIDI note #35, which is MIDI note B1, which is the B key below C2
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum GeneralMidiPercussionCode {
    AcousticBassDrum = 35,
    ElectricBassDrum = 36,
    SideStick = 37,
    AcousticSnare = 38,
    HandClap = 39,
    ElectricSnare = 40,
    LowFloorTom = 41,
    ClosedHiHat = 42,
    HighFloorTom = 43,
    PedalHiHat = 44,
    LowTom = 45,
    OpenHiHat = 46,
    LowMidTom = 47,
    HiMidTom = 48,
    CrashCymbal1 = 49,
    HighTom = 50,
    RideCymbal1 = 51,
    ChineseCymbal = 52,
    RideBell = 53,
    Tambourine = 54,
    SplashCymbal = 55,
    Cowbell = 56,
    CrashCymbal2 = 57,
    Vibraslap = 58,
    RideCymbal2 = 59,
    HighBongo = 60,
    LowBongo = 61,
    MuteHighConga = 62,
    OpenHighConga = 63,
    LowConga = 64,
    HighTimbale = 65,
    LowTimbale = 66,
    HighAgogo = 67,
    LowAgogo = 68,
    Cabasa = 69,
    Maracas = 70,
    ShortWhistle = 71,
    LongWhistle = 72,
    ShortGuiro = 73,
    LongGuiro = 74,
    Claves = 75,
    HighWoodblock = 76,
    LowWoodblock = 77,
    MuteCuica = 78,
    OpenCuica = 79,
    MuteTriangle = 80,
    OpenTriangle = 81,
}
