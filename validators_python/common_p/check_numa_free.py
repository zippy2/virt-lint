import sys
import libvirt

def check():
    would_fit = False
    dom_mem = int(vl.dom_xpath("//domain/memory/text()")[0])
    node_ids = vl.caps_xpath("//capabilities/host/topology/cells/cell/@id")

    conn = vl.get_libvirt_conn()
    if not conn:
        return

    for node in node_ids:
        node_free = conn.getCellsFreeMemory(int(node), 1)

        if node_free[0] > dom_mem:
            would_fit = True
            exit

    if not would_fit:
        vl.add_warning(vl.WarningDomain_Domain, vl.WarningLevel_Error,
                       "Not enough free memory on any NUMA node")

check()
