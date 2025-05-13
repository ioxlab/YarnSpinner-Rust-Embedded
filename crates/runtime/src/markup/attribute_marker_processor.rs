//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner/YarnSpinner.Markup/IAttributeMarkerProcessor.cs>

use crate::prelude::*;
use core::fmt::Debug;

mod dialogue_text_processor;
mod no_markup_text_processor;

/// Provides a mechanism for producing replacement text for a marker.
pub(crate) trait AttributeMarkerProcessor: Debug + Send + Sync {
    fn clone_box(&self) -> Box<dyn AttributeMarkerProcessor>;
}

impl Clone for Box<dyn AttributeMarkerProcessor> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
