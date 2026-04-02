-- c.luau
local e = require(script.Parent.e)
local f = require(script.Parent.f)

return {
    run = function()
        return "C running: " .. e.status() .. " | " .. f.boot()
    end
}
