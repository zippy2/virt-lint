/* SPDX-License-Identifier: LGPL-3.0-or-later */

use crate::utils::*;
use crate::*;
use std::collections::HashSet;
use sxd_document::dom::Document;
use sxd_xpath::evaluate_xpath;

type ValidatorCB = dyn Fn(&mut VirtLint, &Document, &Validator) -> VirtLintResult<()>;

struct Validator {
    cb: &'static ValidatorCB,
    tags: Vec<&'static str>,
}

impl PartialEq for Validator {
    fn eq(&self, other: &Validator) -> bool {
        let ours = (self.cb) as *const ValidatorCB;
        let theirs = (other.cb) as *const ValidatorCB;

        #[allow(clippy::vtable_address_comparisons)]
        if ours != theirs || self.tags.len() != other.tags.len() {
            return false;
        }

        self.tags.iter().all(|t| other.tags.contains(t))
    }
}

pub struct Validators {
    validators: Vec<Validator>,
}

impl Validators {
    pub fn new() -> Self {
        let validators = vec![
            Validator {
                cb: &check_numa,
                tags: vec!["TAG_1", "TAG_2"],
            },
            Validator {
                cb: &check_numa_free,
                tags: vec!["TAG_2"],
            },
            Validator {
                cb: &check_node_kvm,
                tags: vec!["TAG_1", "TAG_3"],
            },
            Validator {
                cb: &check_pcie_root_ports,
                tags: vec!["TAG_4"],
            },
        ];

        Self { validators }
    }

    pub fn list_tags(&mut self) -> Vec<String> {
        let mut tags: HashSet<String> = HashSet::new();

        for v in &self.validators {
            v.tags.iter().for_each(|t| {
                tags.insert(t.to_string());
            });
        }

        let mut tags = tags.into_iter().collect::<Vec<String>>();
        tags.sort();
        tags
    }

    fn filter_validators(&self, tags: &Vec<String>) -> VirtLintResult<Vec<&Validator>> {
        let mut ret: Vec<&Validator> = Vec::new();

        if tags.is_empty() {
            self.validators.iter().for_each(|v| {
                ret.push(v);
            });
        } else {
            for tag in tags {
                let mut found: bool = false;

                for validator in &self.validators {
                    if validator.tags.contains(&tag.as_str()) {
                        found = true;

                        if !ret.contains(&validator) {
                            ret.push(validator);
                        }
                    }
                }

                if !found {
                    return Err(VirtLintError::UnknownValidatorTag(tag.to_string()));
                }
            }
        }
        Ok(ret)
    }

    pub fn validate(
        self,
        tags: &Vec<String>,
        vl: &mut VirtLint,
        domxml: &Document,
    ) -> VirtLintResult<()> {
        let validators = self.filter_validators(tags)?;

        for validator in validators.iter() {
            (validator.cb)(vl, domxml, validator)?;
        }

        Ok(())
    }
}

fn check_numa(vl: &mut VirtLint, domxml: &Document, va: &Validator) -> VirtLintResult<()> {
    let mut numa_mems: Vec<u64> = Vec::new();
    let mut dom_mem: u64 = 0;
    let mut would_fit: bool = false;

    let caps = match vl.capabilities_get()? {
        Some(caps) => caps,
        None => {
            return Ok(());
        }
    };
    let caps_doc = caps.as_document();

    let r = evaluate_xpath(
        &caps_doc,
        "//capabilities/host/topology/cells/cell/memory/text()",
    );
    if let Ok(sxd_xpath::Value::Nodeset(ns)) = r {
        for node in ns.iter() {
            if let sxd_xpath::nodeset::Node::Text(val) = node {
                numa_mems.push(val.text().parse().unwrap())
            }
        }
    }

    let r = evaluate_xpath(domxml, "//domain/memory");
    if let Ok(v) = r {
        dom_mem = v.number() as u64;
    }

    for node in numa_mems.iter() {
        if node > &dom_mem {
            would_fit = true;
            break;
        }
    }

    if !would_fit {
        vl.add_warning(
            &va.tags,
            WarningDomain::Domain,
            WarningLevel::Error,
            String::from("Domain would not fit into any host NUMA node"),
        );
    }

    Ok(())
}

fn check_numa_free(vl: &mut VirtLint, domxml: &Document, va: &Validator) -> VirtLintResult<()> {
    let mut numa_ids: Vec<i32> = Vec::new();
    let mut numa_mems_free: Vec<u64> = Vec::new();
    let mut dom_mem: u64 = 0;
    let mut would_fit: bool = false;

    let conn = match vl.get_conn()? {
        Some(c) => c,
        None => return Ok(()),
    };

    let caps = match vl.capabilities_get()? {
        Some(caps) => caps,
        None => {
            return Ok(());
        }
    };
    let caps_doc = caps.as_document();

    let r = evaluate_xpath(&caps_doc, "//capabilities/host/topology/cells/cell/@id");
    if let Ok(sxd_xpath::Value::Nodeset(ns)) = r {
        for node in ns.iter() {
            if let sxd_xpath::nodeset::Node::Attribute(val) = node {
                numa_ids.push(val.value().parse().unwrap())
            }
        }
    }

    for node in numa_ids.iter() {
        conn.conn
            .get_cells_free_memory(*node, 1)
            .unwrap()
            .into_iter()
            .for_each(|x| numa_mems_free.push(x));
    }

    let r = evaluate_xpath(domxml, "//domain/memory");
    if let Ok(v) = r {
        dom_mem = v.number() as u64;
    }

    numa_mems_free.into_iter().for_each(|x| {
        if x > dom_mem {
            would_fit = true;
        }
    });

    if !would_fit {
        vl.add_warning(
            &va.tags,
            WarningDomain::Domain,
            WarningLevel::Error,
            String::from("Not enough free memory on any NUMA node"),
        );
    }

    Ok(())
}

fn check_node_kvm(vl: &mut VirtLint, domxml: &Document, va: &Validator) -> VirtLintResult<()> {
    let mut emit_warning = false;

    if vl.domain_capabilities_get(Some(domxml)).is_err() {
        emit_warning = true;
        /* Plain fact we failed to look up domain capabilities for given XML warrants a warning.
         * But let's try harder. */
    }

    if let Some(caps) = vl.capabilities_get()? {
        let caps_doc = caps.as_document();
        let mut xpath: String = String::new();

        let emulator = xpath_eval_or_none(domxml, "//domain/devices/emulator");
        let arch = xpath_eval_or_none(domxml, "//domain/os/type/@arch");
        let machine = xpath_eval_or_none(domxml, "//domain/os/type/@machine");
        let virttype = xpath_eval_or_none(domxml, "//domain/@type");

        if let Some(s) = arch {
            xpath += &format!("@name='{s}'")
        }

        if let Some(s) = emulator {
            xpath += &format!(
                "{}emulator/text()='{s}'",
                if !xpath.is_empty() { " and " } else { "" }
            )
        }

        if let Some(s) = machine {
            xpath += &format!(
                "{}machine/text()='{s}'",
                if !xpath.is_empty() { " and " } else { "" }
            )
        }

        if let Some(s) = virttype {
            xpath += &format!(
                "{}domain/@type='{s}'",
                if !xpath.is_empty() { " and " } else { "" }
            )
        }

        let mut top_xpath = String::from("//capabilities/guest/arch");
        if !xpath.is_empty() {
            top_xpath += &format!("[{xpath}]")
        };

        emit_warning = xpath_eval_or_none(&caps_doc, &top_xpath).is_none();
    }

    if emit_warning {
        vl.add_warning(
            &va.tags,
            WarningDomain::Node,
            WarningLevel::Warning,
            String::from("No suitable emulator found"),
        );
    }

    Ok(())
}

fn check_pcie_root_ports(
    vl: &mut VirtLint,
    domxml: &Document,
    va: &Validator,
) -> VirtLintResult<()> {
    let mut pcie_chassis: Vec<u64> = Vec::new();

    let virttype = match xpath_eval_or_none(domxml, "//domain/@type") {
        Some(x) => x,
        None => {
            return Ok(());
        }
    };

    if virttype != "kvm" && virttype != "qemu" {
        return Ok(());
    }

    let machine = match xpath_eval_or_none(domxml, "//domain/os/type/@machine") {
        Some(x) => x,
        None => {
            return Ok(());
        }
    };

    if !machine.contains("q35") {
        return Ok(());
    }

    let r = evaluate_xpath(
        domxml,
        "//domain/devices/controller[@type='pci']/target/@chassis",
    );

    if let Ok(sxd_xpath::Value::Nodeset(ns)) = r {
        for node in ns.iter() {
            if let sxd_xpath::nodeset::Node::Attribute(val) = node {
                pcie_chassis.push(val.value().parse().unwrap());
            }
        }
    }

    // Firstly, remove obviously taken root ports
    if !pcie_chassis.is_empty() {
        let r = evaluate_xpath(domxml, "//domain/devices//address[@type='pci']/@bus");

        if let Ok(sxd_xpath::Value::Nodeset(ns)) = r {
            for node in ns.iter() {
                if let sxd_xpath::nodeset::Node::Attribute(val) = node {
                    let bus: u64 = parse_int(val.value()).unwrap();
                    let mut i = 0;

                    while i < pcie_chassis.len() {
                        if pcie_chassis[i] != bus {
                            i += 1;
                            continue;
                        }

                        pcie_chassis.remove(i);
                    }
                }
            }
        }
    }

    // Then, remove those, which would be taken by PCI address auto assignment
    // TODO

    if pcie_chassis.is_empty() {
        vl.add_warning(
            &va.tags,
            WarningDomain::Domain,
            WarningLevel::Notice,
            String::from("No free PCIe root ports found, hotplug might be not possible"),
        );
    }

    Ok(())
}
