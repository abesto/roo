(function()
    system:add_verb({system.uuid, "", {"do_login_command"}}, {})
    system:set_verb_code("do_login_command", [=[
        player = db:create()
        player:move(system.starting_room)
        player:chparent(system.Player)
        player.name = "guest"

        return player.uuid
        ]=])

    local Root = db:create()
    Root.name = "root object"
    system.Root = Root.uuid

    Root:add_verb({system.uuid, "r", {"tell"}}, {"any"})
    Root:set_verb_code("tell", [[
        this:notify(tostr(args))
    ]])

    local Player = db:create()
    system.Player = Player.uuid
    Player.name = "generic player"
    Player:chparent(Root)

    local Room = db:create()
    Room.name = "Prototype:Room"
    Room.description = "A nondescript room"
    Room:chparent(Root)

    Room:add_verb({system.uuid, "r", {"announce"}}, {"any"})
    Room:set_verb_code("announce", [[
        for i, target in ipairs(setremove(this.contents, player)) do
            pcall(target.tell, args)
        end
    ]])

    Room:add_verb({system.uuid, "r", {"announce_all"}}, {"any"})
    Room:set_verb_code("announce_all", [[
        for i, target in ipairs(this.contents) do
            pcall(target.tell, args)
        end
    ]])

    Room:add_verb({system.uuid, "rx", {"say"}}, {"any"})
    Room:set_verb_code("say", [[
        pcall(function()
            -- TODO player should really be caller here once implemented
            player.tell{'You say, "', argstr, '"'}
            this.announce{player.name, ' says, "', argstr, '"'}
        end)
    ]])

    Room:add_verb({system.uuid, "rx", {"emote"}}, {"any"})
    Room:set_verb_code("emote", [[
        -- TODO player should really be caller here once implemented
        this.announce_all{player.name, ' ', argstr}
    ]])

    Room:add_verb({system.uuid, "rx", {"describe"}}, {})
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

        local seen = setremove(this.contents, player)

        if #seen > 0 then
            local seen_names = imap(function (o) return o.name end, seen)
            msg = msg .. "\r\nYou see here: " .. table.concat(seen_names, ", ")
        end

        return msg
    ]])

    Room:add_verb({system.uuid, "rx", {"look"}}, {})
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
