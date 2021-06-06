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
]])

return player.uuid
