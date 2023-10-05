/* SPDX-License-Identifier: LGPL-3.0-or-later */

// The rustc is complaining about dead code because only used when
// ignored tests are executed.
#![allow(dead_code, unused_imports)]

#[cfg(test)]
use crate::*;
use virt::connect::Connect;
use virt::domain::Domain;

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
    let c = conn();
    {
        let vl = VirtLint::new(Some(&c));
        assert!(vl.warnings.is_empty());
    }
    close(c);
}

#[test]
fn test_list_tags() {
    let tags = VirtLint::list_validator_tags();
    assert_eq!(tags, ["TAG_1", "TAG_2", "TAG_3", "TAG_4"]);
}

#[test]
fn test_simple() {
    let c = conn();
    {
        let dom = match Domain::lookup_by_name(&c, "test") {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };

        let domxml = dom.get_xml_desc(0).unwrap_or_default();

        let mut vl = VirtLint::new(Some(&c));

        assert!(vl.validate(&domxml, &Vec::new(), false).is_ok());

        assert_eq!(
            vl.warnings,
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
            ]
        );
    }

    close(c);
}

#[test]
fn test_offline_simple() {
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

    assert!(vl.capabilities_set(Some(&capsxml)).is_ok());
    assert!(vl.domain_capabilities_add(&domcapsxml).is_ok());
    assert!(vl.validate(&domxml, &Vec::new(), false).is_ok());

    assert_eq!(
        vl.warnings,
        vec![VirtLintWarning::new(
            vec![String::from("TAG_1"), String::from("TAG_2")],
            WarningDomain::Domain,
            WarningLevel::Error,
            String::from("Domain would not fit into any host NUMA node")
        ),]
    );
}

#[test]
fn test_offline_with_error() {
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

    assert!(vl.capabilities_set(Some(&capsxml)).is_ok());
    assert!(vl.domain_capabilities_add(&domcapsxml).is_ok());

    // This fails, because there is a validator that requires connection
    assert!(vl.validate(&domxml, &Vec::new(), true).is_err());

    // This succeeds, because we deliberately run offline only validators
    assert!(vl
        .validate(
            &domxml,
            &vec!["TAG_1".to_owned(), "TAG_3".to_owned(), "TAG_4".to_owned()],
            true
        )
        .is_ok());

    assert_eq!(
        vl.warnings,
        vec![VirtLintWarning::new(
            vec![String::from("TAG_1"), String::from("TAG_2")],
            WarningDomain::Domain,
            WarningLevel::Error,
            String::from("Domain would not fit into any host NUMA node")
        ),]
    );
}
