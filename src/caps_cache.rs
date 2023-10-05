/* SPDX-License-Identifier: LGPL-3.0-or-later */

use crate::*;
use std::collections::HashMap;
use sxd_document::parser;
use sxd_document::Package;

#[derive(Debug)]
pub(crate) struct CapsCache {
    caps: Option<Package>,
}

impl CapsCache {
    pub(crate) fn new() -> Self {
        Self { caps: None }
    }

    pub(crate) fn set(&mut self, capsxml: Option<&str>) -> VirtLintResult<()> {
        self.caps = match capsxml {
            Some(c) => Some(parser::parse(c)?),
            None => None,
        };
        Ok(())
    }

    pub(crate) fn get(
        &mut self,
        conn: Option<&VirtLintConnect>,
        error_on_no_connect: bool,
    ) -> VirtLintResult<Option<&Package>> {
        if self.caps.is_none() {
            match conn {
                Some(c) => {
                    let caps = c.conn.get_capabilities()?;
                    let caps = parser::parse(&caps)?;
                    self.caps = Some(caps);
                }
                None => {
                    if error_on_no_connect {
                        return Err(VirtLintError::InvalidArgument("no connection"));
                    }

                    return Ok(None);
                }
            }
        }

        Ok(self.caps.as_ref())
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct DomCapsKey {
    emulator: Option<String>,
    arch: Option<String>,
    machine: Option<String>,
    virttype: Option<String>,
}

#[derive(Debug)]
pub(crate) struct DomCapsCache {
    cache: HashMap<DomCapsKey, Package>,
}

impl DomCapsCache {
    pub(crate) fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.cache.clear();
    }

    pub(crate) fn add(&mut self, domcapsxml: &str) -> VirtLintResult<()> {
        let domcapsxml_package = parser::parse(domcapsxml)?;
        let domcapsxml_doc = domcapsxml_package.as_document();

        let emulator = xpath_eval_or_none(&domcapsxml_doc, "//domainCapabilities/path");
        let arch = xpath_eval_or_none(&domcapsxml_doc, "//domainCapabilities/arch");
        let machine = xpath_eval_or_none(&domcapsxml_doc, "//domainCapabilities/machine");
        let virttype = xpath_eval_or_none(&domcapsxml_doc, "//domainCapabilities/domain");

        let key = DomCapsKey {
            emulator: emulator.clone(),
            arch: arch.clone(),
            machine: machine.clone(),
            virttype: virttype.clone(),
        };

        self.cache.insert(key, domcapsxml_package);
        Ok(())
    }

    pub(crate) fn get(
        &mut self,
        conn: Option<&VirtLintConnect>,
        error_on_no_connect: bool,
        emulator: Option<String>,
        arch: Option<String>,
        machine: Option<String>,
        virttype: Option<String>,
    ) -> VirtLintResult<Option<&Package>> {
        let key = DomCapsKey {
            emulator: emulator.clone(),
            arch: arch.clone(),
            machine: machine.clone(),
            virttype: virttype.clone(),
        };

        // TODO: come up with a better lookup algorithm
        // If there's (Some(emulator), Some(arch), Some(machine), Some(virttype)) in the cache,
        // and we're called with (None, None, None, None) (i.e. caller doesn't care about a
        // specific domcaps, just any will do), then .contains_key() returns false and we query
        // new domcaps.
        if !self.cache.contains_key(&key) {
            if conn.is_none() {
                if error_on_no_connect {
                    return Err(VirtLintError::InvalidArgument("no connection"));
                }
                return Ok(None);
            }

            if let Some(c) = conn {
                let domcaps = c.conn.get_domain_capabilities(
                    emulator.as_deref(),
                    arch.as_deref(),
                    machine.as_deref(),
                    virttype.as_deref(),
                    0,
                )?;

                let domcaps_package = parser::parse(&domcaps)?;
                self.cache.insert(key.clone(), domcaps_package);
            }
        }

        Ok(self.cache.get(&key))
    }
}
