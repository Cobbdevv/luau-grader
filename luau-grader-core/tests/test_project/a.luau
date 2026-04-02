-- a.luau
local c = require(script.Parent.c)

return {
    init = function()
        return "A initialized: " .. c.run()
    end
}
