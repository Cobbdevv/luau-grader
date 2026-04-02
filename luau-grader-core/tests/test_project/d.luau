-- d.luau
local b = require(script.Parent.b)

return {
    ping = function()
        return "Ping... " .. b.pong()
    end
}
