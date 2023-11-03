local would_fit = false
local dom_mem = tonumber(vl:dom_xpath("//domain/memory/text()")[1])
local numa_mems = vl:caps_xpath("//capabilities/host/topology/cells/cell/memory/text()")

for _, node in ipairs(numa_mems) do
    if tonumber(node) > dom_mem then
        would_fit = true
    end
end

if not would_fit then
    vl:add_warning(vl.WarningDomain_Domain, vl.WarningLevel_Error,
                   "Domain would not fit into any host NUMA node")
end
