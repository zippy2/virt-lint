emit_warning = False

domcaps = vl.domcaps_xpath("/")

# Plain fact there exists domain caps for this domain means there exists
# satisfactory (emulator, arch, machine, virttype) tuple and libvirt found it.
# But okay, try harder.
if not domcaps or not domcaps[0]:
    xpath = ""
    emulator = vl.dom_xpath("//domain/devices/emulator/text()")
    arch = vl.dom_xpath("//domain/os/type/@arch")
    machine = vl.dom_xpath("//domain/os/type/@machine")
    virttype = vl.dom_xpath("//domain/@type")

    if arch:
        xpath += f"@name='{arch[0]}'"

    if emulator:
        if not xpath:
            xpath += " and "
        xpath += f"%semulator/text()='{emulator[0]}'"

    if machine:
        if not xpath:
            xpath += " and "
        xpath += f"machine/text()='{machine[0]}'"

    if virttype:
        if not xpath:
            xpath += " and "
        xpath += f"domain/@type='{virttype[0]}'"

    top_xpath = "//capabilities/guest/arch"
    if not xpath:
        top_xpath += f"[{xpath}]"

    emit_warning = vl.caps_xpath(top_xpath)

if emit_warning:
    vl.add_warning(vl.WarningDomain_Domain, vl.WarningLevel_Error,
                   "No suitable emulator found")
