//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner/YarnSpinner.Markup/LineParser.cs>

use crate::markup::MarkupParseError;
use crate::prelude::*;
use unicode_normalization::UnicodeNormalization;

/// A result type for the line parser
pub type Result<T> = core::result::Result<T, MarkupParseError>;

/// Returns a new string whose textual value is the same as this string, but whose binary representation is in Unicode normalization form C.
pub(crate) fn normalize(string: &str) -> String {
    string.nfc().to_string()
}


/// The name of the implicitly-generated `character` attribute.
pub const CHARACTER_ATTRIBUTE: &str = "character";

/// The name of the 'name' property, on the implicitly-generated `character` attribute.
pub const CHARACTER_ATTRIBUTE_NAME_PROPERTY: &str = "name";

/// The name of the property to use to signify that trailing whitespace should be trimmed
/// if a tag had preceding whitespace or begins the line. This property must be a bool value.
pub const TRIM_WHITESPACE_PROPERTY: &str = "trimwhitespace";