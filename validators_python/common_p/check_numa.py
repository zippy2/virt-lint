would_fit = False
dom_mem = int(vl.dom_xpath("//domain/memory/text()")[0])
numa_mems = vl.caps_xpath("//capabilities/host/topology/cells/cell/memory/text()")

for node in numa_mems:
    if int(node) > dom_mem:
        would_fit = True

if not would_fit:
    vl.add_warning(vl.WarningDomain_Domain, vl.WarningLevel_Error,
                   "Domain would not fit into any host NUMA node")
