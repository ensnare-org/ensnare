// Copyright (c) 2024 Mike Tsao

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use synonym::Synonym;

/// [ModSerial] is a simple counter that lets us inform subscribers that
/// something has changed. Subscribers should keep a usize and compare to see
/// whether it differs from the one that we're currently reporting. If it does,
/// then they should update it and deal with the change.
#[derive(Synonym, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[synonym(skip(Default))]
#[serde(rename_all = "kebab-case")]
pub struct ModSerial(
    // We start at something other than usize::default() so that
    // everyone else can use the default value and fire their update
    // code on the first call to has_changed().
    #[derivative(Default(value = "1000"))] pub usize,
);
