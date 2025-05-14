//! The runtime components of Yarn Spinner. These mostly follow the same structure as the original Yarn Spinner runtime.
//!
//! You probably don't want to use this crate directly.
//! - If you're a game developer, you'll want to use a crate that is already designed for your game engine of choice,
//!   such as [`bevy_yarnspinner`](https://crates.io/crates/bevy_yarnspinner) for the [Bevy engine](https://bevyengine.org/).
//! - If you wish to write an adapter crate for an engine yourself, use the [`yarnspinner`](https://crates.io/crates/yarnspinner) crate.

#![warn(missing_docs, missing_debug_implementations)]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod command;
mod dialogue;
mod dialogue_option;
mod events;
mod language;
mod line;
pub mod markup;
mod variable_storage;
mod virtual_machine;

pub use dialogue::Result;

/// Everything you need to get starting using the Yarn Spinner runtime.
pub mod prelude {
    // Re-export alloc types for internal use only
    pub(crate) use alloc::{
        borrow::ToOwned,
        boxed::Box,
        format,
        string::{String, ToString},
        vec,
        vec::Vec,
    };

    pub use crate::{
        command::*,
        dialogue::{Dialogue, DialogueError},
        dialogue_option::*,
        events::*,
        language::*,
        line::*,
        markup::MarkupParseError,
        variable_storage::*,
    };
    pub(crate) use crate::{virtual_machine::*};
    pub(crate) use yarnspinner_core::prelude::*;
}
