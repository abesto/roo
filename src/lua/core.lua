(function()
    system:add_verb({system.uuid, "", {"do_login_command"}}, {}):unwrap()

    -- S.code_utils
    S.code_utils = create(S.nothing, S.nothing):unwrap()
    S.code_utils:add_verb({system.uuid, "rx", {"short_prep"}}, {"any"}):unwrap()
    S.code_utils:add_verb({system.uuid, "rx", {"full_prep"}}, {"any"}):unwrap()
    S.code_utils:add_verb({system.uuid, "rx", {"toobj"}}, {"any"}):unwrap()
    S.code_utils:add_verb({system.uuid, "r", {"parse_verbref"}}, {"this", "none", "this"}):unwrap()
    S.code_utils:add_verb({system.uuid, "r", {"parse_argspec"}}, {"any"}):unwrap()
    S.code_utils:add_verb({system.uuid, "r", {"find_verb_named"}}, {"any"}):unwrap()
    S.code_utils:add_verb({system.uuid, "rx", {"verbname_match"}}, {"any"}):unwrap()
    -- EOF S.code_utils

    -- S.object_utils
    S.object_utils = create(S.nothing, S.nothing):unwrap()
    S.object_utils:add_verb({system.uuid, "r", {"has_verb"}}, {"any"}):unwrap()
    -- EOF S.object_utils

    -- S.string_utils
    S.string_utils = create(S.nothing, S.nothing):unwrap()
    S.string_utils:add_verb({system.uuid, "r", {"words"}}, {"any"}):unwrap()
    S.string_utils:add_verb({system.uuid, "r", {"from_list"}}, {"any"}):unwrap()
    S.string_utils:add_verb({system.uuid, "r", {"match_object"}}, {"this", "none", "this"}):unwrap()
    S.string_utils:add_verb({system.uuid, "r", {"match"}}, {"this", "none", "this"}):unwrap()
    S.string_utils:add_verb({system.uuid, "r", {"literal_object"}}, {"this", "none", "this"}):unwrap()
    -- EOF S.string_utils

    -- S.command_utils
    S.command_utils = create(S.nothing, S.nothing):unwrap()
    S.command_utils:add_verb({system.uuid, "r", {"object_match_failed"}}, {"any"}):unwrap()
    S.command_utils:add_verb({system.uuid, "r", {"dump_lines"}}, {"any"}):unwrap()
    -- EOF S.command_utils

    -- S.verb_editor
    S.verb_editor = create(S.nothing, S.nothing):unwrap()
    S.verb_editor:add_verb({system.uuid, "r", {"get_room"}}, {"any"}):unwrap()
    -- EOF verb_editor

    -- S.Root
    S.Root = create(S.nothing, S.nothing):unwrap()
    S.Root.name = "root object"
    S.Root:add_verb({system.uuid, "rx", {"match"}}, {"this", "none", "this"}):unwrap()
    S.Root:add_verb({system.uuid, "r", {"get_name"}}, {"any"}):unwrap()
    S.Root:add_verb({system.uuid, "r", {"title"}}, {"any"}):unwrap()
    S.Root:add_verb({system.uuid, "r", {"tell"}}, {"any"}):unwrap()
    -- EOF S.Root

    -- S.Player
    S.Player = create(S.Root, S.nothing):unwrap()
    S.Player.name = "generic player"
    S.Player:add_verb({system.uuid, "rx", {"my_match_object"}}, {"any"}):unwrap()
    -- TODO at some point this needs to move to a "generic programmer" object
    S.Player:add_verb({S.uuid, "rx", {"@edit"}}, {"any", "any", "any"}):unwrap()
    -- TODO impl
    S.Player:add_verb({S.uuid, "rx", {"edit_option"}}, {}):unwrap()
    -- TODO full impl
    S.Player:add_verb({S.uuid, "rx", {"@program"}}, {"this"}):unwrap()
    -- EOF S.Player

    -- S.Room
    S.Room = create(S.Root, S.nothing):unwrap()
    S.Room.name = "Prototype:Room"
    S.Room.description = "A nondescript room"
    S.Room:add_verb({system.uuid, "r", {"announce"}}, {"any"}):unwrap()
    S.Room:add_verb({system.uuid, "r", {"announce_all"}}, {"any"}):unwrap()
    S.Room:add_verb({system.uuid, "rx", {"say"}}, {"any"}):unwrap()
    S.Room:add_verb({system.uuid, "rx", {"emote"}}, {"any"}):unwrap()
    S.Room:add_verb({system.uuid, "rx", {"describe"}}, {}):unwrap()
    S.Room:add_verb({system.uuid, "rx", {"look"}}, {}):unwrap()
    -- EOF S.Room

    S.starting_room = create(S.Room, S.nothing):unwrap()
    S.starting_room.name = "The Void"
    S.starting_room.description = "There is nothing, and you are in it."

    S.starting_room:add_verb({system.uuid, "rx", {"wiggle"}}, {}):unwrap()
    S.starting_room:set_verb_code("wiggle", [[
        this:announce_all("%s wiggles" % {this.name})
    ]]):unwrap()
end)()
