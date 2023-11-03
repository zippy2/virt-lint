/* SPDX-License-Identifier: LGPL-3.0-or-later */

use thiserror::Error;

pub(crate) type VirtLintResult<T> = Result<T, VirtLintError>;

#[derive(Debug, Error)]
pub enum VirtLintError {
    #[error("Libvirt error: {0}")]
    Libvirt(#[from] virt::error::Error),

    #[error("Unable to parse XML: {0}")]
    XMLParsing(#[from] libxml::parser::XmlParseError),

    #[error("Unknown validator tag: {0}")]
    UnknownValidatorTag(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(&'static str),

    #[error("I/O error: {0}")]
    IOError(String),

    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::prelude::LuaError),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}
