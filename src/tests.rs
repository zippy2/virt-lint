/* SPDX-License-Identifier: LGPL-3.0-or-later */

// The rustc is complaining about dead code because only used when
// ignored tests are executed.

#[cfg(test)]
use crate::*;
use std::sync::Once;
use virt::connect::Connect;
use virt::domain::Domain;

static TEST_INIT: Once = Once::new();

fn test_init() {
    TEST_INIT.call_once(|| {
        // Set
        std::env::set_var(
            "VIRT_LINT_LUA_PATH",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../validators_lua"),
        );
        std::env::set_var(
            "VIRT_LINT_PYTHON_PATH",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../validators_python"),
        );
    });
}

fn conn() -> Connect {
    let c = Connect::open(Some("test:///default"));
    assert!(c.is_ok());
    c.unwrap()
}

fn close(mut conn: Connect) {
    assert_eq!(Ok(0), conn.close(), "close(), expected 0")
}

#[test]
fn test_empty() {
    test_init();

    let c = conn();
    {
        let vl = VirtLint::new(Some(&c));
        assert!(vl.warnings().is_empty());
    }
    close(c);
}

#[test]
fn test_list_tags() {
    test_init();

    let tags = VirtLint::list_validator_tags().unwrap();
    assert_eq!(
        tags,
        [
            "TAG_1",
            "TAG_2",
            "TAG_3",
            "TAG_4",
            "common",
            "common/check_node_kvm",
            "common/check_numa",
            "common/check_numa_free",
            "common/check_pcie_root_ports",
            "common_p",
            "common_p/check_node_kvm",
            "common_p/check_numa",
            "common_p/check_numa_free",
            "common_p/check_pcie_root_ports",
        ]
    );
}

#[test]
fn test_simple() {
    test_init();

    let c = conn();
    {
        let dom = match Domain::lookup_by_name(&c, "test") {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };

        let domxml = dom.get_xml_desc(0).unwrap_or_default();

        let mut vl = VirtLint::new(Some(&c));

        assert!(vl.validate(&domxml, &Vec::new(), false).is_ok());

        let mut warnings = vl.warnings();
        warnings.sort();

        assert_eq!(
            warnings,
            vec![
                VirtLintWarning::new(
                    vec![String::from("TAG_1"), String::from("TAG_2")],
                    WarningDomain::Domain,
                    WarningLevel::Error,
                    String::from("Domain would not fit into any host NUMA node")
                ),
                VirtLintWarning::new(
                    vec![String::from("TAG_2")],
                    WarningDomain::Domain,
                    WarningLevel::Error,
                    String::from("Not enough free memory on any NUMA node")
                ),
                VirtLintWarning::new(
                    vec![String::from("common"), String::from("common/check_numa")],
                    WarningDomain::Domain,
                    WarningLevel::Error,
                    String::from("Domain would not fit into any host NUMA node")
                ),
                VirtLintWarning::new(
                    vec![
                        String::from("common"),
                        String::from("common/check_numa_free")
                    ],
                    WarningDomain::Domain,
                    WarningLevel::Error,
                    String::from("Not enough free memory on any NUMA node")
                ),
                VirtLintWarning::new(
                    vec![String::from("common_p"), String::from("common_p/check_numa")],
                    WarningDomain::Domain,
                    WarningLevel::Error,
                    String::from("Domain would not fit into any host NUMA node")
                ),
                VirtLintWarning::new(
                    vec![
                        String::from("common_p"),
                        String::from("common_p/check_numa_free")
                    ],
                    WarningDomain::Domain,
                    WarningLevel::Error,
                    String::from("Not enough free memory on any NUMA node")
                ),
            ]
        );
    }

    close(c);
}

#[test]
fn test_offline_simple() {
    test_init();

    // The connection here is used only to get domain XML and capabilities. Validation is done
    // completely offline.
    let c = conn();
    let domxml;
    let capsxml;
    let domcapsxml;
    {
        let dom = match Domain::lookup_by_name(&c, "test") {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };

        domxml = dom.get_xml_desc(0).unwrap_or_default();
        capsxml = c.get_capabilities().unwrap_or_default();
        domcapsxml = c
            .get_domain_capabilities(None, None, None, None, 0)
            .unwrap_or_default();
    }

    close(c);

    let mut vl = VirtLint::new(None);

    assert!(vl.capabilities_set(Some(capsxml)).is_ok());
    assert!(vl.domain_capabilities_add(domcapsxml).is_ok());
    assert!(vl.validate(&domxml, &Vec::new(), false).is_ok());

    let mut warnings = vl.warnings();

    warnings.sort();

    assert_eq!(
        warnings,
        vec![
            VirtLintWarning::new(
                vec![String::from("TAG_1"), String::from("TAG_2")],
                WarningDomain::Domain,
                WarningLevel::Error,
                String::from("Domain would not fit into any host NUMA node")
            ),
            VirtLintWarning::new(
                vec![String::from("common"), String::from("common/check_numa")],
                WarningDomain::Domain,
                WarningLevel::Error,
                String::from("Domain would not fit into any host NUMA node")
            ),
            VirtLintWarning::new(
                vec![String::from("common_p"), String::from("common_p/check_numa")],
                WarningDomain::Domain,
                WarningLevel::Error,
                String::from("Domain would not fit into any host NUMA node")
            ),
        ]
    );
}

#[test]
fn test_offline_with_error() {
    test_init();

    // The connection here is used only to get domain XML and capabilities. Validation is done
    // completely offline.
    let c = conn();
    let domxml;
    let capsxml;
    let domcapsxml;
    {
        let dom = match Domain::lookup_by_name(&c, "test") {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };

        domxml = dom.get_xml_desc(0).unwrap_or_default();
        capsxml = c.get_capabilities().unwrap_or_default();
        domcapsxml = c
            .get_domain_capabilities(None, None, None, None, 0)
            .unwrap_or_default();
    }

    close(c);

    let mut vl = VirtLint::new(None);

    assert!(vl.capabilities_set(Some(capsxml)).is_ok());
    assert!(vl.domain_capabilities_add(domcapsxml).is_ok());

    // This fails, because there is a validator that requires connection
    assert!(vl.validate(&domxml, &Vec::new(), true).is_err());

    // This succeeds, because we deliberately run offline only validators
    assert!(vl
        .validate(
            &domxml,
            &vec![
                String::from("TAG_1"),
                String::from("TAG_3"),
                String::from("TAG_4"),
                String::from("common/check_node_kvm"),
                String::from("common/check_numa"),
                String::from("common/check_pcie_root_ports"),
                String::from("common_p/check_node_kvm"),
                String::from("common_p/check_numa"),
                String::from("common_p/check_pcie_root_ports"),
            ],
            true
        )
        .is_ok());

    let mut warnings = vl.warnings();

    warnings.sort();

    assert_eq!(
        warnings,
        vec![
            VirtLintWarning::new(
                vec![String::from("TAG_1"), String::from("TAG_2")],
                WarningDomain::Domain,
                WarningLevel::Error,
                String::from("Domain would not fit into any host NUMA node")
            ),
            VirtLintWarning::new(
                vec![String::from("common"), String::from("common/check_numa")],
                WarningDomain::Domain,
                WarningLevel::Error,
                String::from("Domain would not fit into any host NUMA node")
            ),
            VirtLintWarning::new(
                vec![String::from("common_p"), String::from("common_p/check_numa")],
                WarningDomain::Domain,
                WarningLevel::Error,
                String::from("Domain would not fit into any host NUMA node")
            ),
        ]
    );
}
