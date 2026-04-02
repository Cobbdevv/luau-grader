-- dead_feature_1.luau
local dead2 = require(script.Parent.dead_feature_2)

return {
    calculate = function(a, b)
        return a + b + dead2.offset
    end
}
