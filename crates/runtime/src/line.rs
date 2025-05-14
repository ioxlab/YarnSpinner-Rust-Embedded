//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner/Dialogue.cs>, which we split off into multiple files
//!
//! ## Implementation notes
//! Introduced `LineId` newtype for better type safety

use crate::prelude::*;

/// A line of dialogue, sent from the [`Dialogue`] to the game.
///
/// A [`Line`] is automatically produced follows:
/// - A localized text was fetched through the [`TextProvider`] registered in the [`Dialogue`].
/// - Any expressions found in the text are evaluated
/// - The text is parsed for markup
///
/// You do not create instances of this struct yourself. They are created by the [`Dialogue`] during program execution.
///
/// ## See also
/// [`DialogueEvent::Line`]
///
/// ## Implementation Notes
///
/// `MarkupParseResult` and `ExpandSubstitutions` were merged into this because we don't require consumers to manually fetch from string tables.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Line {
    /// The ID of the line in the string table.
    pub id: LineId,
}