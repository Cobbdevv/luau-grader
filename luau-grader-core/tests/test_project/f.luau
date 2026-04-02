-- f.luau
local g = require(script.Parent.g)

return {
    boot = function()
        return "F booted: " .. g.start()
    end,
    check = function()
        return true
    end
}
