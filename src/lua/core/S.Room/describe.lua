local name = this:title()
local description = this.description or "You see nothing special."
local msg = '%s\n%s' % {name, description}

local seen = this.contents:without(player):map(_1.name)
if #seen > 0 then
    msg = msg .. "\nYou see here: " .. table.concat(seen, ", ")
end

return msg
