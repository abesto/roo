me = db:create()
me.name = "a player"
me:move(void)

me:add_verb{"wave"}
me:set_verb_code("wave", [[
    local also_here = {}

    for k, other_uuid in ipairs(location.contents) do
        if other_uuid ~= me.uuid then
            local other = db[other_uuid]
            table.insert(also_here, other.name)
            other:notify(me.name .. " waves at you")
        end
    end

    if next(also_here) ~= nil then
        me:notify("You wave at " .. table.concat(also_here, ", "))
    else
        me:notify("You wave at empty space")
    end
]])

return me.uuid
