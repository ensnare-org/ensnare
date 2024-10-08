// Copyright (c) 2024 Mike Tsao

//! A set of things that the user can select.

use crate::types::IsUid;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::collections::hash_set::Iter;

/// A utility class to help manage selection sets of things that implement the
/// [IsUid] trait.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SelectionSet<T: IsUid> {
    selected_uids: FxHashSet<T>,
}

impl<T: IsUid> SelectionSet<T> {
    /// Adds an item to the selection set.
    pub fn insert(&mut self, uid: T) {
        self.selected_uids.insert(uid);
    }

    /// Removes an item from the selection set.
    pub fn remove(&mut self, uid: &T) {
        self.selected_uids.remove(uid);
    }

    /// Changes the selection state of an item.
    pub fn set_selected(&mut self, uid: T, selected: bool) {
        if selected {
            self.insert(uid);
        } else {
            self.remove(&uid);
        }
    }

    /// Indicates whether the given item is selected.
    pub fn contains(&self, uid: &T) -> bool {
        self.selected_uids.contains(uid)
    }

    /// Select none.
    pub fn clear(&mut self) {
        self.selected_uids.clear();
    }

    /// Returns an iterator of all selected items.
    pub fn iter(&self) -> Iter<'_, T> {
        self.selected_uids.iter()
    }

    /// Returns the number of selected items.
    pub fn len(&self) -> usize {
        self.selected_uids.len()
    }

    #[allow(missing_docs)]
    pub fn is_empty(&self) -> bool {
        self.selected_uids.is_empty()
    }

    /// Convenience method to handle a click on an item that's meant as a
    /// selection action. `modify_selection_set` is typically set when the user
    /// is holding down the control or Command key while clicking.
    ///
    /// TODO: this struct isn't smart enough to handle the shift modifier key.
    /// It doesn't know about any item in the set that isn't selected, nor does
    /// it know the topology of the set, so it doesn't have any way of
    /// determining how to select all the items between two items. If this is
    /// interesting in the future, then add it.
    pub fn click(&mut self, uid: &T, modify_selection_set: bool) {
        let is_selected = self.contains(uid);
        if modify_selection_set {
            // The user is holding down the control key. This means that the
            // indicated item's selection state should be toggled, but the rest
            // of the items in the set shouldn't change.
            if is_selected {
                self.remove(uid);
            } else {
                self.insert(uid.clone());
            }
        } else {
            // A plain click with no modifier keys. Just toggle this item's
            // selection state.
            self.clear();
            if !is_selected {
                self.insert(uid.clone());
            }
        }
    }

    /// If a single item is selected, returns it. Otherwise returns None.
    pub fn single_selection(&self) -> Option<&T> {
        if self.selected_uids.len() == 1 {
            self.selected_uids.iter().next()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn select_mainline() {
        let mut st = SelectionSet::default();

        assert!(st.is_empty());
        assert_eq!(st.len(), 0);

        let uid2048 = TrackUid(2048);
        let uid2049 = TrackUid(2049);
        assert!(!st.contains(&uid2048));

        st.insert(uid2048);
        assert!(st.contains(&uid2048));

        st.clear();
        assert!(st.is_empty());

        st.click(&uid2048, false);
        assert_eq!(st.len(), 1);
        assert!(st.contains(&uid2048));
        assert!(!st.contains(&uid2049));

        st.click(&uid2049, true);
        assert_eq!(st.len(), 2);
        assert!(st.contains(&uid2048));
        assert!(st.contains(&uid2049));

        st.click(&uid2049, true);
        assert!(st.contains(&uid2048));
        assert!(!st.contains(&uid2049));

        st.click(&uid2048, true);
        assert!(st.is_empty());

        assert!(st.single_selection().is_none());
        st.set_selected(uid2048, true);
        assert_eq!(st.single_selection().unwrap(), &uid2048);
        st.set_selected(uid2049, true);
        assert!(st.single_selection().is_none());
    }
}
