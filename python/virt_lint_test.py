#!/usr/bin/env python3

import libvirt
import virt_lint

for tag in virt_lint.VirtLint.list_validator_tags():
    print(tag)

conn = libvirt.open("test:///default")

dom = conn.lookupByName("test")

vl = virt_lint.VirtLint(conn)

vl.validate(dom.XMLDesc(), [], False)

for warning in vl.warnings():
    print(warning)
