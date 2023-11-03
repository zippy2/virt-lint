local has_free_root_port = false

local virttype = vl:dom_xpath("//domain/@type")

if virttype == nil or (virttype[1] ~= "kvm" and virttype[1] ~= "qemu") then
    -- no kvm/qemu
    return
end

local machine = vl:dom_xpath("//domain/os/type/@machine")

if machine == nil or string.find(machine[1], "q35") == nil then
    -- no q35
    return
end

local taken = {}

local pcie_chassis = vl:dom_xpath("//domain/devices/controller[@type='pci' and @model='pcie-root-port']/target/@chassis")
if pcie_chassis ~= nil then
    for _, v in ipairs(pcie_chassis) do
        taken[tonumber(v)] = -1 end
end

-- Firstly, remove obviously taken root ports
local devices = vl:dom_xpath("//domain/devices//address[@type='pci']/@bus")
if devices ~= nil then
    for _, v in ipairs(devices) do
        local x = tonumber(v:gsub("0x", ""), 16)
        taken[tonumber(x)] = 1
    end
end

-- Then, remove those, which would be taken by PCI address auto assignment
-- TODO

-- And finally see if there is at least one root port free
for _, v in pairs(taken) do
    if v < 0 then has_free_root_port = true; break; end
end

if not has_free_root_port then
    vl:add_warning(vl.WarningDomain_Domain, vl.WarningLevel_Notice,
                   "No free PCIe root ports found, hotplug might be not possible")
end
