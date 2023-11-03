/* SPDX-License-Identifier: LGPL-3.0-or-later */

use crate::VirtLintError;
use crate::VirtLintResult;
use libxml::tree::Document;
use libxml::xpath::Context;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::Path;
use std::path::PathBuf;

pub(crate) fn xpath_eval_or_none(doc: &Document, xpath: &str) -> Option<String> {
    let mut ret = String::new();

    let ctxt = Context::new(doc).unwrap();
    match ctxt.evaluate(xpath) {
        Err(_) => {
            return None;
        }
        Ok(nodes) => {
            for node in nodes.get_nodes_as_vec() {
                ret += &node.get_content()
            }
        }
    }

    Some(ret)
}

pub(crate) fn xpath_eval_nodeset_or_none(doc: &Document, xpath: &str) -> Option<Vec<String>> {
    let mut ret: Vec<String> = Vec::new();

    let ctxt = Context::new(doc).unwrap();
    let nodes = ctxt.evaluate(xpath);

    if nodes.is_err() {
        return None;
    }

    let nodes = nodes.unwrap();

    for node in nodes.get_nodes_as_vec() {
        let val = match node.get_type() {
            Some(libxml::tree::NodeType::AttributeNode) => node.get_content(),
            _ => doc.node_to_string(&node),
        };

        ret.push(val);
    }

    if ret.is_empty() {
        return None;
    }

    Some(ret)
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

pub(crate) fn recurse_files(
    path: impl AsRef<Path> + std::fmt::Debug,
    filename_prefix: Option<&OsStr>,
    ext: Option<&OsStr>,
) -> VirtLintResult<Vec<PathBuf>> {
    let mut buf = vec![];
    let entries = read_dir(&path).map_err(|x| VirtLintError::IOError(format!("{path:?}: {x}")))?;

    for entry in entries {
        let entry = entry
            .as_ref()
            .map_err(|x| VirtLintError::IOError(format!("{entry:?}: {x}")))?;
        let meta = entry
            .metadata()
            .map_err(|x| VirtLintError::IOError(format!("{entry:?}: {x}")))?;

        if meta.is_dir() {
            let mut subdir = recurse_files(entry.path(), filename_prefix, ext)?;
            buf.append(&mut subdir);
        } else if meta.is_file() || meta.is_symlink() {
            if ext.is_some() && entry.path().extension() != ext {
                continue;
            }

            if let Some(f_prefix) = filename_prefix {
                let fname = entry.file_name();
                let fname_str = fname.to_str();

                if let Some(f) = fname_str {
                    let fname = &f[..f_prefix.len()];
                    if fname != f_prefix {
                        continue;
                    }
                }
            }

            buf.push(entry.path());
        }
    }

    Ok(buf)
}
