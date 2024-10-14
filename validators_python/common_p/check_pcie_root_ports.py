def check():
    has_free_root_port = False

    virttype = vl.dom_xpath("//domain/@type")

    if not virttype or virttype[0] not in ("kvm", "qemu"):
        # no kvm/qemu
        return

    machine = vl.dom_xpath("//domain/os/type/@machine")

    if not machine or not machine[0].find("q35"):
        # no q35
        return

    taken = {}

    pcie_chassis = vl.dom_xpath("//domain/devices/controller[@type='pci' and @model='pcie-root-port']/target/@chassis")
    if pcie_chassis:
        for v in pcie_chassis:
            taken[int(v)] = -1

    # Firstly, remove obviously taken root ports
    devices = vl.dom_xpath("//domain/devices//address[@type='pci']/@bus")
    if devices:
        for v in devices:
            taken[int(v, 16)] = 1

    # Then, remove those, which would be taken by PCI address auto assignment
    # TODO

    # And finally see if there is at least one root port free
    for v in taken.values():
        if v < 0:
            has_free_root_port = True
            break

    if not has_free_root_port:
        vl.add_warning(vl.WarningDomain_Domain, vl.WarningLevel_Notice,
                       "No free PCIe root ports found, hotplug might be not possible")

check()
