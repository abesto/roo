system = db[system_uuid]

if system.starting_room == nil then
    void = db:create()
    void.name = "The Void"
    void.description = "You float in nothing."

    void:add_verb{"look"}
    void:set_verb_code("look", [[
    local name = this.name
    if name == "" then
        name = this.uuid
    end

    local description = this.description
    if description == nil then
        player:notify("(No description set for " .. name .. ")")
    else
        player:notify("= " .. name .. " =\r\n" .. description)
    end
    ]])

    system.starting_room = void.uuid

    system:add_verb{"do_login_command"}
    system:set_verb_code("do_login_command", [=[
    player = db:create()
    player.name = "a player"
    player:move(system.starting_room)

    player:add_verb{"wave"}
    player:set_verb_code("wave", [[
        local also_here = {}

        for k, other_uuid in ipairs(location.contents) do
            if other_uuid ~= player.uuid then
                local other = db[other_uuid]
                table.insert(also_here, other.name)
                other:notify(player.name .. " waves at you")
            end
        end

        if next(also_here) ~= nil then
            player:notify("You wave at " .. table.concat(also_here, ", "))
        else
            player:notify("You wave at empty space")
        end

        -- TODO emote that player connected
    ]])

    return player.uuid
    ]=])
end

