/* SPDX-License-Identifier: LGPL-3.0-or-later */

package VirtLint_test

import (
	libvirt "libvirt.org/go/libvirt"
	"os"
	"reflect"
	"testing"
	"gitlab.com/MichalPrivoznik/virt-lint/go/virt-lint"
)

func TestMain(t *testing.T) {
	// TODO - switch to an absolute path, replaced at compile time
	os.Setenv("VIRT_LINT_LUA_PATH", "../validators_lua:../../validators_lua")
}

func getConn(t *testing.T) *libvirt.Connect {
	conn, err := libvirt.NewConnect("test:///default")
	if err != nil {
		t.Fatal(err)
	}
	return conn
}

func closeConn(conn *libvirt.Connect, t *testing.T) {
	res, err := conn.Close()
	if err != nil {
		t.Fatal(err)
	}
	if res != 0 {
		t.Errorf("Close() == %d, expected 0", res)
	}
}

func TestEmpty(t *testing.T) {
	conn := getConn(t)
	defer closeConn(conn, t)

	vl, err := VirtLint.New(conn)
	if err != nil {
		t.Error(err)
		return
	}
	defer vl.Close()

	warn, err := vl.GetWarnings()
	if err != nil {
		t.Error(err)
		return
	}

	if len(warn) != 0 {
		t.Errorf("Expected no warnings, got %v", warn)
		return
	}
}

func TestListTags(t *testing.T) {
	tags, err := VirtLint.List_validator_tags()

	if err != nil {
		t.Error(err)
		return
	}

	expect := []string{"TAG_1", "TAG_2", "TAG_3", "TAG_4",
		"common", "common/check_node_kvm", "common/check_numa",
		"common/check_numa_free", "common/check_pcie_root_ports"}
	if !reflect.DeepEqual(tags, expect) {
		t.Errorf("Tags don't match:\nexpected = %v\ngot = %v", expect, tags)
		return
	}
}

func TestSimple(t *testing.T) {
	conn := getConn(t)
	defer closeConn(conn, t)

	dom, err := conn.LookupDomainByName("test")
	if err != nil {
		t.Error(err)
		return
	}
	defer dom.Free()

	domxml, err := dom.GetXMLDesc(0)
	if err != nil {
		t.Error(err)
		return
	}

	vl, err := VirtLint.New(conn)
	if err != nil {
		t.Error(err)
		return
	}
	defer vl.Close()

	err = vl.Validate(domxml, []string{}, false)
	if err != nil {
		t.Error(err)
		return
	}

	warn, err := vl.GetWarnings()
	if err != nil {
		t.Error(err)
		return
	}

	expect := []VirtLint.VirtLintWarning{
		VirtLint.VirtLintWarning{[]string{"TAG_1", "TAG_2"},
			VirtLint.DOMAIN, VirtLint.ERROR, "Domain would not fit into any host NUMA node"},
		VirtLint.VirtLintWarning{[]string{"TAG_2"},
			VirtLint.DOMAIN, VirtLint.ERROR, "Not enough free memory on any NUMA node"},
		VirtLint.VirtLintWarning{[]string{"common", "common/check_numa"},
			VirtLint.DOMAIN, VirtLint.ERROR, "Domain would not fit into any host NUMA node"},
		VirtLint.VirtLintWarning{[]string{"common", "common/check_numa_free"},
			VirtLint.DOMAIN, VirtLint.ERROR, "Not enough free memory on any NUMA node"},
	}
	if !reflect.DeepEqual(warn, expect) {
		t.Errorf("Warnings don't match:\nexpected = %v\ngot = %v", expect, warn)
		return
	}
}

func TestOfflineSimple(t *testing.T) {
	// The connection here is used only to get domain XML and capabilities.
	// Validation is done completely offline.
	conn := getConn(t)
	defer closeConn(conn, t)

	dom, err := conn.LookupDomainByName("test")
	if err != nil {
		t.Error(err)
		return
	}
	defer dom.Free()

	domxml, err := dom.GetXMLDesc(0)
	if err != nil {
		t.Error(err)
		return
	}

	capsxml, err := conn.GetCapabilities()
	if err != nil {
		t.Error(err)
		return
	}

	domcapsxml, err := conn.GetDomainCapabilities("", "", "", "", 0)
	if err != nil {
		t.Error(err)
		return
	}

	vl, err := VirtLint.New(nil)
	if err != nil {
		t.Error(err)
		return
	}
	defer vl.Close()

	err = vl.CapabilitiesSet(capsxml)
	if err != nil {
		t.Error(err)
		return
	}

	err = vl.DomainCapabilitiesAdd(domcapsxml)
	if err != nil {
		t.Error(err)
		return
	}

	err = vl.Validate(domxml, []string{}, false)
	if err != nil {
		t.Error(err)
		return
	}

	warn, err := vl.GetWarnings()
	if err != nil {
		t.Error(err)
		return
	}

	expect := []VirtLint.VirtLintWarning{
		VirtLint.VirtLintWarning{[]string{"TAG_1", "TAG_2"},
			VirtLint.DOMAIN, VirtLint.ERROR, "Domain would not fit into any host NUMA node"},
		VirtLint.VirtLintWarning{[]string{"common", "common/check_numa"},
			VirtLint.DOMAIN, VirtLint.ERROR, "Domain would not fit into any host NUMA node"},
	}
	if !reflect.DeepEqual(warn, expect) {
		t.Errorf("Warnings don't match:\nexpected = %v\ngot = %v", expect, warn)
		return
	}
}

func TestOfflineWithError(t *testing.T) {
	// The connection here is used only to get domain XML and capabilities.
	// Validation is done completely offline.
	conn := getConn(t)
	defer closeConn(conn, t)

	dom, err := conn.LookupDomainByName("test")
	if err != nil {
		t.Error(err)
		return
	}
	defer dom.Free()

	domxml, err := dom.GetXMLDesc(0)
	if err != nil {
		t.Error(err)
		return
	}

	capsxml, err := conn.GetCapabilities()
	if err != nil {
		t.Error(err)
		return
	}

	domcapsxml, err := conn.GetDomainCapabilities("", "", "", "", 0)
	if err != nil {
		t.Error(err)
		return
	}

	vl, err := VirtLint.New(nil)
	if err != nil {
		t.Error(err)
		return
	}
	defer vl.Close()

	err = vl.CapabilitiesSet(capsxml)
	if err != nil {
		t.Error(err)
		return
	}

	err = vl.DomainCapabilitiesAdd(domcapsxml)
	if err != nil {
		t.Error(err)
		return
	}

	// This fails, because there is a validator that requires connection
	err = vl.Validate(domxml, []string{}, true)
	if err == nil {
		t.Errorf("Expected failure, got success")
		return
	}

	// This succeeds, because we deliberately run offline only validators
	err = vl.Validate(domxml, []string{"TAG_1", "TAG_3", "TAG_4",
		"common/check_node_kvm", "common/check_numa", "common/check_pcie_root_ports"}, true)
	if err != nil {
		t.Error(err)
		return
	}

	warn, err := vl.GetWarnings()
	if err != nil {
		t.Error(err)
		return
	}

	expect := []VirtLint.VirtLintWarning{
		VirtLint.VirtLintWarning{[]string{"TAG_1", "TAG_2"},
			VirtLint.DOMAIN, VirtLint.ERROR, "Domain would not fit into any host NUMA node"},
		VirtLint.VirtLintWarning{[]string{"common", "common/check_numa"},
			VirtLint.DOMAIN, VirtLint.ERROR, "Domain would not fit into any host NUMA node"},
	}
	if !reflect.DeepEqual(warn, expect) {
		t.Errorf("Warnings don't match:\nexpected = %v\ngot = %v", expect, warn)
		return
	}
}
