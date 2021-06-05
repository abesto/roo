-- TODO this should be local, but until object lookup by name is done, I'll just leave it global to access it
--      during player spawning
void = db:create()
void.name = "The Void"
void.description = "You float in nothing."

void:add_verb{"look"}
void:set_verb_code("look", [[
local name = self.name
if name == "" then
    name = self.uuid
end

local description = self.description
if description == nil then
    player:notify("(No description set for " .. name .. ")")
else
    player:notify("= " .. name .. " =\r\n" .. description)
end
]])
