/* SPDX-License-Identifier: LGPL-3.0-or-later */

use thiserror::Error;

pub(crate) type VirtLintResult<T> = Result<T, VirtLintError>;

#[derive(Debug, Error)]
pub enum VirtLintError {
    #[error("Libvirt error: {0}")]
    Libvirt(#[from] virt::error::Error),

    #[error("Unable to parse XML: {0}")]
    XMLParsing(#[from] sxd_document::parser::Error),

    #[error("Unknown validator tag: {0}")]
    UnknownValidatorTag(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(&'static str),
}
