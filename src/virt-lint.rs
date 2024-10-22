/* SPDX-License-Identifier: LGPL-3.0-or-later */

#[cfg(feature = "capi")]
mod capi;
mod caps_cache;
pub mod errors;
mod helpers;
#[cfg(test)]
mod tests;
mod utils;
mod validators;
mod validators_lua;
mod validators_python;

use crate::caps_cache::*;
use crate::errors::*;
use crate::utils::*;
use crate::validators::*;
use libxml::parser::Parser;
use libxml::tree::Document;
use virt::connect::Connect;

#[macro_use]
extern crate enum_display_derive;
use std::fmt::Display;
use std::sync::{Arc, Mutex};

#[repr(C)]
#[derive(Clone, Copy, Debug, Display, PartialEq, PartialOrd, Ord, Eq)]
pub enum WarningDomain {
    /// The problem lies inside of domain XML
    Domain,

    /// The problem lies on remote host
    Node,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Display, PartialEq, PartialOrd, Ord, Eq)]
pub enum WarningLevel {
    /// Critical error, domain won't start
    Error,

    /// Suboptimal domain configuration
    Warning,

    /// Domain configuration is okay, but can use tweaking
    Notice,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct VirtLintWarning {
    tags: Vec<String>,
    domain: WarningDomain,
    level: WarningLevel,
    msg: String,
}

impl VirtLintWarning {
    fn new(tags: Vec<String>, domain: WarningDomain, level: WarningLevel, msg: String) -> Self {
        Self {
            tags,
            domain,
            level,
            msg,
        }
    }

    pub fn get(&self) -> (&[String], &WarningDomain, &WarningLevel, &String) {
        (&self.tags, &self.domain, &self.level, &self.msg)
    }
}

// Connect does not implement Copy trait. Use Clone with automatic refcounting for now.
#[derive(Debug, Clone)]
struct VirtLintConnect {
    conn: Connect,
}

impl Drop for VirtLintConnect {
    fn drop(&mut self) {
        let _ = self.conn.close();
    }
}

impl VirtLintConnect {
    fn new(conn: &Connect) -> Self {
        Self { conn: conn.clone() }
    }
}

#[derive(Debug, Clone)]
pub struct VirtLint {
    conn: Option<VirtLintConnect>,
    caps_cache: Arc<Mutex<CapsCache>>,
    domcaps_cache: Arc<Mutex<DomCapsCache>>,
    warnings: Arc<Mutex<Vec<VirtLintWarning>>>,
    error_on_no_connect: bool,
}

impl VirtLint {
    /// Creates a VirtLint object.
    ///
    /// Optionally, pass a connection `conn` which is then used internally by linting rules to
    /// query information on the virtualization host. If None is passed (so called "offline
    /// validation") then use [`capabilities_set()`] and [`domain_capabilities_add()`] to feed the
    /// rules with libvirt capabilities and domain capabilities XMLs (acquired earlier).
    ///
    /// [`capabilities_set()`]: VirtLint::capabilities_set
    /// [`domain_capabilities_add()`]: VirtLint::domain_capabilities_add
    ///
    /// # Examples
    ///
    /// ````
    /// use virt::connect::Connect;
    /// use virt_lint::VirtLint;;
    ///
    /// let mut conn = Connect::open(Some("test:///default")).unwrap();
    /// let vl = VirtLint::new(Some(&conn));
    /// let _ = conn.close(); // vl holds its own clone of the connection
    /// ````
    pub fn new(conn: Option<&Connect>) -> Self {
        Self {
            conn: conn.map(VirtLintConnect::new),
            caps_cache: Arc::new(Mutex::new(CapsCache::new())),
            domcaps_cache: Arc::new(Mutex::new(DomCapsCache::new())),
            warnings: Arc::new(Mutex::new(Vec::new())),
            error_on_no_connect: false,
        }
    }

    /// Obtain the connection.
    ///
    /// Intended to be used by validators.
    fn get_conn(&mut self) -> VirtLintResult<Option<VirtLintConnect>> {
        if self.conn.is_none() && self.error_on_no_connect {
            return Err(VirtLintError::InvalidArgument("no connection"));
        }
        Ok(self.conn.clone())
    }

    /// Obtain the libvirt direct connection (virt::Connect).
    ///
    /// Intended to be used by validators.
    fn get_virt_conn(&mut self) -> VirtLintResult<Option<Connect>> {
        if self.conn.is_none() && self.error_on_no_connect {
            return Err(VirtLintError::InvalidArgument("no connection"));
        }
        Ok(self.conn.as_ref().map(|c| c.conn.clone()))
    }

    /// Get capabilities.
    ///
    /// Intended to be used by validators.
    /// If the offline mode was requested and no capabilities were set beforehand (via
    /// [`capabilities_set()`]) an error is returned.
    fn capabilities_get(&mut self) -> VirtLintResult<Option<String>> {
        Ok(self
            .caps_cache
            .lock()
            .expect("Mutex poisoned")
            .get(self.conn.as_ref(), self.error_on_no_connect)?
            .map(String::to_string))
    }

    /// Set capabilities.
    ///
    /// If connection was provided in [`new()`] then there's no need to call this function as
    /// correct capabilities will be obtained automatically. Calling this function multiple times
    /// clears previously set capabilities.
    ///
    /// Pass `None` to clear any previously set capabilities.
    ///
    /// [`new()`]: VirtLint::new
    pub fn capabilities_set(&mut self, capsxml: Option<String>) -> VirtLintResult<()> {
        self.caps_cache.lock().expect("Mutex poisoned").set(capsxml);
        Ok(())
    }

    /// Get domain capabilities.
    ///
    /// Returns domain capabilities as specific to given domain XML `domxml` as possible.
    ///
    /// Intended to be used by validators.
    /// If the offline mode was requested and no domain capabilities were set beforehand (via
    /// [`domain_capabilities_add()`]) an error is returned.
    fn domain_capabilities_get(
        &mut self,
        domxml: Option<&Document>,
    ) -> VirtLintResult<Option<String>> {
        let mut emulator = None;
        let mut arch = None;
        let mut machine = None;
        let mut virttype = None;

        if let Some(domxml_doc) = domxml {
            emulator = xpath_eval_or_none(domxml_doc, "//domain/devices/emulator");
            arch = xpath_eval_or_none(domxml_doc, "//domain/os/type/@arch");
            machine = xpath_eval_or_none(domxml_doc, "//domain/os/type/@machine");
            virttype = xpath_eval_or_none(domxml_doc, "//domain/@type");
        }

        let mut cache = self.domcaps_cache.lock().expect("Mutex poisoned");

        let caps = cache
            .get(
                self.conn.as_ref(),
                self.error_on_no_connect,
                emulator,
                arch,
                machine,
                virttype,
            )?
            .map(String::to_string);

        Ok(caps)
    }

    /// Clear any previously set domain capabilities.
    pub fn domain_capabilities_clear(&mut self) {
        self.domcaps_cache.lock().expect("Mutex poisoned").clear();
    }

    /// Add new domain capabilities into internal cache.
    ///
    /// If connection was provided in [`new()`] then there's no need to call this function as
    /// correct domain capabilities will be obtained automatically. Call this function multiple
    /// times to add alternative domain capabilities. Domain capabilities 'key' (tuple consisting
    /// of emulator binary, architecture, machine type and virt type) is deducted automatically
    /// from passed XML.
    ///
    /// [`new()`]: VirtLint::new
    pub fn domain_capabilities_add(&mut self, domcapsxml: String) -> VirtLintResult<()> {
        self.domcaps_cache
            .lock()
            .expect("Mutex poisoned")
            .add(domcapsxml)
    }

    /// Add new warning
    ///
    /// Intended to be used by validators.
    fn add_warning(
        &mut self,
        tags: Vec<String>,
        domain: WarningDomain,
        level: WarningLevel,
        msg: String,
    ) {
        let mut tags = tags.clone();
        tags.sort();

        let w = VirtLintWarning::new(tags, domain, level, msg);

        self.warnings.lock().expect("Mutex poisoned").push(w);
    }

    /// Validate given domain XML against set of internal rules.
    ///
    /// Execute internal validators (linting rules) attempting to find problems with domain
    /// configuration (possibly) in combination of current state of virtualization host (as
    /// represented by connection in [`new()`] or (domain) capabilities fed earlier
    /// ([`capabilities_set()`] and [`domain_capabilities_add()`])
    ///
    /// The linting warnings can be then obtained via [`warnings()`].
    ///
    /// Each linting rule has a tag associated with it. To validate domain XML against just a
    /// subset of rules, pass vector of selected tags in `validator_tags`. To obtain the list of
    /// all possible tags use [`list_validator_tags()`]. If no tags are specified then all linting
    /// rules are executed.
    ///
    /// For offline mode validation (i.e. no connection was provided in [`new()`]), use
    /// `error_on_no_connect` to either skip linting rules that require connection (`false`), or
    /// make the whole validation fail (`true`).
    ///
    /// [`new()`]: VirtLint::new
    /// [`capabilities_set()`]: VirtLint::capabilities_set
    /// [`domain_capabilities_add()`]: VirtLint::domain_capabilities_add
    /// [`warnings()`]: VirtLint::warnings
    /// [`list_validator_tags()`]: VirtLint::list_validator_tags
    pub fn validate(
        &mut self,
        domxml: &str,
        validator_tags: &[String],
        error_on_no_connect: bool,
    ) -> VirtLintResult<()> {
        let mut validators = Validators::new();

        // Clear warnings from previous runs
        self.warnings.lock().expect("Mutex poisoned").clear();

        self.error_on_no_connect = error_on_no_connect;

        validators.validate(validator_tags, self, domxml)
    }

    /// List all validator tags.
    ///
    /// Each linting rule has one or more tags associated with it. Tags can be then used to run
    /// only a subset of linting rules. See [`validate()`].
    ///
    /// [`validate()`]: VirtLint::validate
    pub fn list_validator_tags() -> VirtLintResult<Vec<String>> {
        let mut tags: Vec<String> = Validators::new().list_tags()?.into_iter().collect();
        tags.sort();
        Ok(tags)
    }

    /// Obtain linting warnings.
    ///
    /// See [`validate()`].
    ///
    /// [`validate()`]: VirtLint::validate
    pub fn warnings(&self) -> Vec<VirtLintWarning> {
        let mut warnings = self.warnings.lock().expect("Mutex poisoned").clone();
        warnings.sort();
        warnings
    }
}
