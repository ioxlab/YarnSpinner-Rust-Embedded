//! # Yarn Spinner for Rust
//! The friendly dialogue creation tool for Rust.
//!
//! This crate provides a compiler and runtime that can be used standalone, but will most likely be used by a crate providing the functionality
//! to a game engine. For example, [Bevy](https://bevyengine.org/) engine support is given by the [`bevy_yarnspinner`](https://crates.io/crates/bevy_yarnspinner) crate.
#![warn(missing_docs, missing_debug_implementations)]

pub use log;

pub mod prelude {
    //! Everything you need to get started using Yarn Spinner.
    pub use crate::core::{
        yarn_library, IntoYarnValueFromNonYarnValue, Library as YarnLibrary, LineId,
        Program as YarnProgram, YarnFn, YarnValue,
    };
    pub use crate::runtime::{
        Command as YarnCommand,
        Dialogue, DialogueError, DialogueEvent, DialogueOption,
        Language, Line as YarnLine, OptionId,
        Result as YarnRuntimeResult, VariableStorage,
    };
}

pub mod core {
    //! Core types and traits that are used by both the compiler and runtime.
    pub use yarnspinner_core::prelude::{
        optionality, yarn_fn_type, yarn_library, Header, Instruction,
        IntoYarnValueFromNonYarnValue, InvalidOpCodeError, Library, LineId, Node, Position,
        Program, Type, UntypedYarnFn, YarnFn, YarnFnParam, YarnFnParamItem, YarnValue,
        YarnValueCastError, YarnValueWrapper, YarnValueWrapperIter,
    };
}
pub mod runtime {
    //! Types and traits used by the runtime, in particular the [`Dialogue`] struct.
    pub use yarnspinner_runtime::markup::{
        CHARACTER_ATTRIBUTE,
        CHARACTER_ATTRIBUTE_NAME_PROPERTY, TRIM_WHITESPACE_PROPERTY,
    };
    pub use yarnspinner_runtime::prelude::*;
    pub use yarnspinner_runtime::Result;
}
