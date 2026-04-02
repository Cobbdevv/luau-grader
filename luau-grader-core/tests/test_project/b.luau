-- b.luau
local d = require(script.Parent.d)

return {
    init = function()
        return "B initialized: " .. d.ping()
    end,
    pong = function()
        return "Pong!"
    end
}
