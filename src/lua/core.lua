(function()
    system:add_verb{"do_login_command"}
    system:set_verb_code("do_login_command", [=[
        player = db:create()
        player:move(system.starting_room)
        player:chparent(system.Player)

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

            player:emote{"connected to the server"}
        ]])

        return player.uuid
        ]=])

    local Root = db:create()
    Root.name = "Prototype:Root"
    system.Root = Root.uuid

    Root:add_verb{"tell", "any"}

    local Player = db:create()
    system.Player = Player.uuid
    Player.name = "Prototype:Player"

    Player:add_verb{"tell", "any"}
    Player:set_verb_code("tell", [[
        local msg = args[1]
        this:notify(msg)
    ]])

    Player:add_verb{"say", "any"}
    Player:set_verb_code("say", [[
        local msg = args[1]

        for k, uuid in ipairs(this.location.contents) do
            if uuid ~= this.uuid then
                local other = db[uuid]

                local tell = other.tell
                if tell ~= nil then
                    tell{this.name .. " says, \"" .. msg .. "\""}
                end
            end
        end

        this:notify("You say, \"" .. msg .. "\"")
    ]])

    Player:add_verb{"emote", "any"}
    Player:set_verb_code("emote", [[
        local msg = args[1]

        for k, uuid in ipairs(this.location.contents) do
            if uuid ~= this.uuid then
                local other = db[uuid]

                local tell = other.tell
                if tell ~= nil then
                    tell{this.name .. " " .. msg}
                end
            end
        end
    ]])

    local Room = db:create()
    Room.name = "Prototype:Room"
    Room.description = "A nondescript room"

    Room:add_verb{"describe"}
    Room:set_verb_code("describe", [[
        local name = this.name
        if name == "" then
            name = this.uuid
        end
        local msg = "= " .. name .. " ="

        local description = this.description
        if description then
            msg = msg .. "\r\n" .. description
        end

        local seen = {}
        for k, uuid in ipairs(this.contents) do
            if uuid ~= player.uuid then
                local other = db[uuid]
                table.insert(seen, other.name)
            end
        end

        if #seen > 0 then
            msg = msg .. "\r\nYou see here: " .. table.concat(seen, ", ")
        end

        return msg
    ]])

    Room:add_verb{"look"}
    Room:set_verb_code("look", [[
        player:notify(this.describe())
    ]])

    if system.starting_room == nil then
        local void = db:create()
        void.name = "The Void"
        void.description = "You float in nothing."
        void:chparent(Room)

        system.starting_room = void.uuid

    end
end)()
