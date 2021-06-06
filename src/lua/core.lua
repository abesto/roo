system = db[system_uuid]

if system.starting_room == nil then
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

    system.starting_room = void.uuid
end

