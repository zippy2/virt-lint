/* SPDX-License-Identifier: LGPL-3.0-or-later */

use sxd_document::dom::Document;
use sxd_xpath::evaluate_xpath;

pub(crate) fn xpath_eval_or_none(document: &Document, xpath: &str) -> Option<String> {
    evaluate_xpath(document, xpath)
        .ok()
        .map(|x| x.into_string())
        .and_then(|x| if x.is_empty() { None } else { Some(x) })
}

pub(crate) fn parse_int(s: &str) -> std::result::Result<u64, std::num::ParseIntError> {
    if let Some(s) = s.strip_prefix("0x") {
        u64::from_str_radix(s, 16)
    } else if let Some(s) = s.strip_prefix("0o") {
        u64::from_str_radix(s, 8)
    } else if let Some(s) = s.strip_prefix("0b") {
        u64::from_str_radix(s, 2)
    } else {
        s.parse::<u64>()
    }
}
