-- h.luau
local f = require(script.Parent.f)

return {
    execute = function()
        if f.check() then
            return "H executed successfully!"
        end
        return "H failed!"
    end
}
