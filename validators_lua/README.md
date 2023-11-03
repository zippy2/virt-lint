# virt-lint Lua interface

Validators can be written in Lua. By default, virt-lint looks recursively for
files under `/usr/share/virt-lint/validators_lua/`. A validator file must match
`check_*\.lua` pattern. Other files are not considered nor executed.

## Exposed methods

There is `vl` object exposed with the following methods:

```lua
vl:dom_xpath("/domain/xpath")
vl:caps_xpath("/capabilities/xpath")
vl:domcaps_xpath("/domainCapabilities/xpath")
vl:add_warning(domain, level, "warning message")
```

Each of these functions returns either a table (on success) or nil (on error).
For instance, to get emulator from domain XML (passed to
`VirtLint::validate()`) the following can be used:

```lua
local emulator = vl:dom_xpath("/domain/devices/emulator/text()")

if emulator == nil then
    -- no emulator found in the domain XML
else
    print(emulator[1]) -- contains the emulator path
end
```

Or to get list of NUMA node IDs from capabilities XML:

```lua
local node_ids = vl:caps_xpath("/capabilities/host/topology/cells/cell/@id")

if node_ids == nil
    -- no NUMA IDs found in capabilities XML
else for _, node in ipairs(node_ids) do
    print(node) -- print node ID
end

```


After a suboptimal domain/host configuration was detected a validator should
emit a warning, e.g.:

```lua
vl:add_warning(vl.WarningDomain_Domain, vl.WarningLevel_Error,
               "Not enough free memory on any NUMA node")
```

Here, `add_warning()` method accepts the following arguments, for warning
domain:

```lua
-- The problem lies inside of domain XML
vl.WarningDomain_Domain

-- The problem lies on remote host
vl.WarningDomain_Node
```

and for levels:

```lua
-- Critical error, domain won't start
vl.WarningLevel_Error

-- Suboptimal domain configuration
vl.WarningLevel_Warning

-- Domain configuration is okay, but can use tweaking
vl.WarningLevel_Notice
```

Then there is a set of methods that expose full XMLs and allow users to just
run an XPATH query over any XML:

```lua
vl:caps_xml()
vl:dom_xml()
vl:domcaps_xml()
vl:xpath_eval("<xmlDocument/">, "/some/xpath")
```

### Calling Libvirt API

For now, there's just one Libvirt function exposed:

```lua
vl:get_cells_free_memory(start_cell, max_cells)
```

It too follows the return value logic of aforementioned functions: nil is
returned on error or corresponding value on success (e.g. an array of free
memory on each NUMA node from the specified range).

## Filename patterns

As mentioned above, only files matching `check_*\.lua` are read and executed.
This allows for storing helper modules that can be loaded by individual
validators.

A file location is also important as it determines what tags the validator has
(after stripping the common prefix). For instance:
`/usr/share/virt-lint/validators_lua/a/b/check_something.lua` is going to have
the following tags: `a`, `a/b`, and `a/b/check_something`. To share validators
between several tags, either place it at their common ancestor, or create a
symlink.
