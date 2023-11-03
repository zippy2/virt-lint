/* SPDX-License-Identifier: LGPL-3.0-or-later */

use crate::*;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct CapsCache {
    caps: Option<String>,
}

impl CapsCache {
    pub(crate) fn new() -> Self {
        Self { caps: None }
    }

    pub(crate) fn set(&mut self, capsxml: Option<String>) {
        self.caps = capsxml
    }

    pub(crate) fn get(
        &mut self,
        conn: Option<&VirtLintConnect>,
        error_on_no_connect: bool,
    ) -> VirtLintResult<Option<&String>> {
        if self.caps.is_none() {
            match conn {
                Some(c) => {
                    let caps = c.conn.get_capabilities()?;
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
    cache: HashMap<DomCapsKey, String>,
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

    pub(crate) fn add(&mut self, domcapsxml: String) -> VirtLintResult<()> {
        let parser = Parser::default();
        let domcapsxml_doc = parser.parse_string(&domcapsxml)?;

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

        self.cache.insert(key, domcapsxml);
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
    ) -> VirtLintResult<Option<&String>> {
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

                self.cache.insert(key.clone(), domcaps);
            }
        }

        Ok(self.cache.get(&key))
    }
}
