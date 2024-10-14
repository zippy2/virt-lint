/* SPDX-License-Identifier: LGPL-3.0-or-later */

use crate::utils::*;
use crate::*;
use libxml::tree::Document;
use mlua::{Error, ExternalResult, Lua, UserData};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

pub struct ValidatorsLuaUserData<'a> {
    vl: &'a mut VirtLint,
    domxml: &'a str,
    domxml_doc: &'a Document,
    tags: Vec<String>,
}

impl TryFrom<i32> for WarningDomain {
    type Error = VirtLintError;

    fn try_from(value: i32) -> Result<Self, VirtLintError> {
        let ret = match value {
            0 => WarningDomain::Domain,
            1 => WarningDomain::Node,
            _ => {
                return Err(VirtLintError::InvalidArgument("Unknown warning domain"));
            }
        };

        Ok(ret)
    }
}

impl TryFrom<i32> for WarningLevel {
    type Error = VirtLintError;

    fn try_from(value: i32) -> Result<Self, VirtLintError> {
        let ret = match value {
            0 => WarningLevel::Error,
            1 => WarningLevel::Warning,
            2 => WarningLevel::Notice,
            _ => {
                return Err(VirtLintError::InvalidArgument("Unknown warning level"));
            }
        };

        Ok(ret)
    }
}

fn add_warning(
    _: &Lua,
    vlud: &mut ValidatorsLuaUserData,
    (domain, level, msg): (i32, i32, String),
) -> Result<(), Error> {
    let domain = WarningDomain::try_from(domain).into_lua_err()?;
    let level = WarningLevel::try_from(level).into_lua_err()?;

    vlud.vl.add_warning(vlud.tags.clone(), domain, level, msg);
    Ok(())
}

fn caps_xpath(
    _: &Lua,
    vlud: &mut ValidatorsLuaUserData,
    xpath: String,
) -> Result<Option<Vec<String>>, Error> {
    let caps = match vlud.vl.capabilities_get().into_lua_err()? {
        Some(caps) => caps,
        None => {
            return Ok(None);
        }
    };

    let parser = Parser::default();
    let caps_doc = parser.parse_string(caps).into_lua_err()?;

    Ok(xpath_eval_nodeset_or_none(&caps_doc, &xpath))
}

fn dom_xpath(
    _: &Lua,
    vlud: &mut ValidatorsLuaUserData,
    xpath: String,
) -> Result<Option<Vec<String>>, Error> {
    Ok(xpath_eval_nodeset_or_none(vlud.domxml_doc, &xpath))
}

fn domcaps_xpath(
    _: &Lua,
    vlud: &mut ValidatorsLuaUserData,
    xpath: String,
) -> Result<Option<Vec<String>>, Error> {
    let domcaps = match vlud
        .vl
        .domain_capabilities_get(Some(vlud.domxml_doc))
        .into_lua_err()?
    {
        Some(domcaps) => domcaps,
        None => {
            return Ok(None);
        }
    };

    let parser = Parser::default();
    let domcaps_doc = parser.parse_string(domcaps).into_lua_err()?;

    Ok(xpath_eval_nodeset_or_none(&domcaps_doc, &xpath))
}

fn caps_xml(_: &Lua, vlud: &mut ValidatorsLuaUserData, _: ()) -> Result<Option<String>, Error> {
    Ok(vlud.vl.capabilities_get().into_lua_err()?.cloned())
}

fn dom_xml(_: &Lua, vlud: &mut ValidatorsLuaUserData, _: ()) -> Result<String, Error> {
    Ok(String::from(vlud.domxml))
}

fn domcaps_xml(_: &Lua, vlud: &mut ValidatorsLuaUserData, _: ()) -> Result<Option<String>, Error> {
    Ok(vlud
        .vl
        .domain_capabilities_get(Some(vlud.domxml_doc))
        .into_lua_err()?
        .cloned())
}

fn xpath_eval(
    _: &Lua,
    _vlud: &mut ValidatorsLuaUserData,
    (xml, xpath): (String, String),
) -> Result<Option<Vec<String>>, Error> {
    let parser = Parser::default();
    let doc = parser.parse_string(xml).into_lua_err()?;

    Ok(xpath_eval_nodeset_or_none(&doc, &xpath))
}

macro_rules! libvirt_wrap{
    ($func: ident($( $arg:tt : $argtype:tt ),*) ->  $ret:ty ) => {
        fn $func(_: &Lua,
                 vlud: &mut ValidatorsLuaUserData,
                 ($($arg,)*): ($($argtype,)*))
            -> Result<Option<$ret>, Error> {
                let conn = match vlud
                    .vl
                    .get_conn()
                    .map_err(|x| Error::RuntimeError(x.to_string()))?
                    {
                        Some(c) => c,
                        None => return Ok(None),
                    };

                Ok(Some(conn.conn
                            .$func($($arg,)*)
                            .map_err(|x| Error::RuntimeError(x.to_string()))?))
            }
    }
}

libvirt_wrap!(
    get_cells_free_memory(start_cell: i32, max_cells: i32) -> Vec<u64>
);

impl UserData for ValidatorsLuaUserData<'_> {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field("WarningDomain_Domain", WarningDomain::Domain as i32);
        fields.add_field("WarningDomain_Node", WarningDomain::Node as i32);
        fields.add_field("WarningLevel_Error", WarningLevel::Error as i32);
        fields.add_field("WarningLevel_Warning", WarningLevel::Warning as i32);
        fields.add_field("WarningLevel_Notice", WarningLevel::Notice as i32);
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("add_warning", add_warning);
        methods.add_method_mut("caps_xpath", caps_xpath);
        methods.add_method_mut("dom_xpath", dom_xpath);
        methods.add_method_mut("domcaps_xpath", domcaps_xpath);
        methods.add_method_mut("caps_xml", caps_xml);
        methods.add_method_mut("dom_xml", dom_xml);
        methods.add_method_mut("domcaps_xml", domcaps_xml);
        methods.add_method_mut("xpath_eval", xpath_eval);
        methods.add_method_mut("get_cells_free_memory", get_cells_free_memory);
    }
}

fn get_tags_for_path(prefix: &PathBuf, path: &Path) -> Vec<String> {
    let mut ret = Vec::new();

    let p = match path.strip_prefix(prefix) {
        Ok(x) => x,
        Err(_) => return vec![],
    };

    for anc in p.ancestors() {
        match PathBuf::from(anc)
            .with_extension("")
            .into_os_string()
            .into_string()
        {
            Ok(x) => {
                if !x.is_empty() {
                    ret.push(x);
                }
            }
            Err(_) => continue,
        }
    }

    ret
}

fn get_paths_for_tag(
    prefix: &Path,
    tag: &String,
    filename_prefix: &OsString,
    ext: &OsString,
) -> VirtLintResult<Vec<PathBuf>> {
    let path = prefix.join(tag);

    if !path.is_dir() {
        let path = path.with_extension(ext);
        if path.exists() {
            return Ok(vec![path]);
        }
    }

    recurse_files(path, Some(filename_prefix), Some(ext))
}

fn get_validators(
    prefix: &PathBuf,
    tags: &[String],
    filename_prefix: &OsString,
    ext: &OsString,
) -> Vec<PathBuf> {
    let mut ret: HashSet<PathBuf> = HashSet::new();

    if tags.is_empty() {
        return recurse_files(prefix, Some(filename_prefix), Some(ext)).unwrap_or_default();
    } else {
        for tag in tags.iter() {
            let tag_paths =
                get_paths_for_tag(prefix, tag, filename_prefix, ext).unwrap_or_default();

            for tag_path in tag_paths {
                ret.insert(tag_path);
            }
        }
    }

    let mut ret = ret.into_iter().collect::<Vec<PathBuf>>();
    ret.sort();
    ret
}

fn validate_one(
    path: PathBuf,
    prefix: &PathBuf,
    vl: &mut VirtLint,
    domxml: &str,
    domxml_doc: &Document,
) -> VirtLintResult<()> {
    let lua = Lua::new();
    let vlud = ValidatorsLuaUserData {
        vl,
        domxml,
        domxml_doc,
        tags: get_tags_for_path(prefix, &path),
    };

    lua.scope(|scope| {
        let f = scope.create_nonstatic_userdata(vlud)?;

        lua.globals().set("vl", f)?;

        lua.load(path).exec()
    })?;

    Ok(())
}

pub struct ValidatorsLua {
    prefix: Vec<PathBuf>,
    filename_prefix: OsString,
    ext: OsString,
}

impl ValidatorsLua {
    pub fn new(prefix: Vec<PathBuf>, filename_prefix: &'static str, ext: &'static str) -> Self {
        let mut prefix_exists: Vec<PathBuf> = Vec::new();

        for p in prefix {
            if p.exists() {
                prefix_exists.push(p);
            }
        }

        Self {
            prefix: prefix_exists,
            filename_prefix: OsString::from(filename_prefix),
            ext: OsString::from(ext),
        }
    }

    pub fn list_tags(&self) -> VirtLintResult<HashSet<String>> {
        let mut ret: HashSet<String> = HashSet::new();

        for p in self.prefix.iter() {
            let rc = recurse_files(p, Some(&self.filename_prefix), Some(&self.ext))?;
            for path in rc {
                let tags = get_tags_for_path(p, &path);

                for tag in tags {
                    ret.insert(tag);
                }
            }
        }

        Ok(ret)
    }

    pub fn validate(
        &self,
        tags: &[String],
        vl: &mut VirtLint,
        domxml: &str,
        domxml_doc: &Document,
    ) -> VirtLintResult<()> {
        for p in self.prefix.iter() {
            let validators = get_validators(p, tags, &self.filename_prefix, &self.ext);

            for validator in validators {
                validate_one(validator, p, vl, domxml, domxml_doc)?;
            }
        }

        Ok(())
    }
}
