local emit_warning = false

local domcaps = vl:domcaps_xpath("/")
-- Plain fact there exists domain caps for this domain means there exists
-- satisfactory (emulator, arch, machine, virttype) tuple and libvirt found it.
-- But okay, try harder.
if domcaps == nil or domcaps[1] == nil then
    local xpath = ""
    local emulator = vl:dom_xpath("//domain/devices/emulator/text()")
    local arch = vl:dom_xpath("//domain/os/type/@arch")
    local machine = vl:dom_xpath("//domain/os/type/@machine")
    local virttype = vl:dom_xpath("//domain/@type")

    if arch ~= nil then
        xpath = xpath .. string.format("@name='%s'", arch[1])
    end

    if emulator ~= nil then
        xpath = xpath .. string.format("%semulator/text()='%s'", xpath == nil and "" or " and ", emulator[1])
    end

    if machine ~= nil then
        xpath = xpath .. string.format("%smachine/text()='%s'", xpath == nil and "" or " and ", machine[1])
    end

    if virttype ~= nil then
        xpath = xpath .. string.format("%sdomain/@type='%s'", xpath == nil and "" or " and ", virttype[1])
    end

    top_xpath = "//capabilities/guest/arch"
    if xpath ~= "" then
        top_xpath = top_xpath .. string.format("[%s]", xpath)
    end

    emit_warning = vl:caps_xpath(top_xpath) == nil
end

if emit_warning then
    vl:add_warning(vl.WarningDomain_Domain, vl.WarningLevel_Error,
                   "No suitable emulator found")
end
