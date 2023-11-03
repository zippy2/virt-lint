local would_fit = false
local dom_mem = tonumber(vl:dom_xpath("//domain/memory/text()")[1])
local node_ids = vl:caps_xpath("//capabilities/host/topology/cells/cell/@id")

for _, node in ipairs(node_ids) do
    local node_free = vl:get_cells_free_memory(node, 1)

    if node_free == nil then
        -- no connection
        return
    end

    if node_free[1] > dom_mem then
        would_fit = true
        break
    end
end

if not would_fit then
    vl:add_warning(vl.WarningDomain_Domain, vl.WarningLevel_Error,
                   "Not enough free memory on any NUMA node")
end

