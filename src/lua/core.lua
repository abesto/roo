(function()
    system:add_verb({system.uuid, "", {"do_login_command"}}, {})
    system:set_verb_code("do_login_command", [=[
        player = db:create()
        player:move(system.starting_room)
        player:chparent(system.Player)
        player.name = "guest"

        return player.uuid
        ]=])

    S.failed_match = db:create()
    S.failed_match.name = "S.failed_match"

    S.nothing = db:create()
    S.nothing.name = "S.nothing"

    S.ambiguous_match = db:create()
    S.ambiguous_match.name = "S.ambiguous_match"

    --- S.code_utils
    local code_utils = db:create()
    S.code_utils = code_utils

    -- TODO impl
    code_utils:add_verb({system.uuid, "rx", {"short_prep"}}, {"any"})
    code_utils:set_verb_code("short_prep", [[
        return args[1]
    ]])

    -- TODO impl
    code_utils:add_verb({system.uuid, "rx", {"full_prep"}}, {"any"})
    code_utils:set_verb_code("full_prep", [[
        return nil
    ]])

    code_utils:add_verb({system.uuid, "rx", {"toobj"}}, {"any"})
    code_utils:set_verb_code("toobj", [[
        return M.toobj(args[1])
    ]])

    code_utils:add_verb({system.uuid, "r", {"parse_verbref"}}, {"this", "none", "this"})
    code_utils:set_verb_code("parse_verbref", [[
        -- S.code_utils:parse_verbref(string)
        -- Parses string as a MOO-code verb reference, returning {object, verb-name-string} for a successful parse and false otherwise.  It always returns the right object-string to pass to, for example, this-room:match_object().
        local s = args[1]
        local colon = string.find(s, ":", 1, true)
        if colon then
            local object = string.sub(s, 1, colon - 1)
            local verbname = string.sub(s, colon + 1)
            if not (object and verbname) then
                return 0
            end
            if string.sub(object, 0, 2) == "S." then
                local pname = string.sub(object, 3)
                local p = S[pname]
                if not ObjectProxy:class_of(p) then
                    return 0
                end
                object = p.uuid
            end
            return {object, verbname}
        else
            return 0
        end
    ]])

    code_utils:add_verb({system.uuid, "r", {"parse_argspec"}}, {"any"})
    code_utils:set_verb_code("parse_argspec", [[
-- :parse_arg_spec(@args)
--  attempts to parse the given sequence of args into a verb_arg specification
--  returns {verb_args,remaining_args} if successful.
--  e.g., :parse_arg_spec(\"this\",\"in\",\"front\",\"of\",\"any\",\"foo\"..)
--           => {{\"this\",\"in front of\",\"any\"},{\"foo\"..}}
--  returns a string error message if parsing fails.
local nargs = #args
local args = pl.List(args)
if nargs < 1 then
  return {{}, {}}
end

local ds = args[1]
if args[1] == "tnt" then
  return {{"this", "none", "this"}, M.listdelete(args, 1)}
elseif not listcontains({"this", "any", "none"}, ds) then
  return M.tostr("\"", ds, "\" is not a valid direct object specifier.")
elseif nargs < 2 or listcontains({"none", "any"}, args[2]) then
  local verbargs = args:slice(1, math.min(3, nargs))
  local rest = args:slice(4, nargs);
end

local gp = pl.List(S.code_utils:get_prep(args:slice(2, nargs)))
if not gp[1] then
  return M.tostr("\"", args[2], "\" is not a valid preposition.");
else
  local nargs = #gp
  local verbargs = {ds, table.unpack(gp:slice(1, math.min(2, nargs)))}
  rest = gp:slice(3, nargs)
end

if #verbargs >= 3 and not listcontains({"this", "any", "none"}, verbargs[3]) then
  return tostr("\"", verbargs[3], "\" is not a valid indirect object specifier.")
end
return {verbargs, rest};
    ]])

    code_utils:add_verb({system.uuid, "r", {"find_verb_named"}}, {"any"})
    code_utils:set_verb_code("find_verb_named", [[
        -- :find_verb_named(object,name[,n])
        --  returns the *number* of the first verb on object matching the given name.
        --  optional argument n, if given, starts the search with verb n,
        --  causing the first n verbs (1..n-1) to be ignored.
        --  nil is returned if no verb is found.
        --  This routine does not find inherited verbs.
        local object, name, start = table.unpack(args)
        if start == nil then
            start = 1
        end
        for i = start, #M.verbs(object) do
          local verbinfo = M.verb_info(object, i);
          if this:verbname_match{verbinfo[3], name} then
            return i
          end
        end
        return nil
    ]])

    -- TODO full impl
    code_utils:add_verb({system.uuid, "rx", {"verbname_match"}}, {"any"})
    code_utils:set_verb_code("verbname_match", [[
        local candidates, name = table.unpack(args)
        return listcontains(candidates, name)
    ]])
    -- EOF S.code_utils

    --- S.object_utils
    local object_utils = db:create()
    S.object_utils = object_utils

    object_utils:add_verb({system.uuid, "r", {"has_verb"}}, {"any"})
    object_utils:set_verb_code("has_verb", [[
        local object, verb = table.unpack(args)
        return db:has_verb_with_name(object.uuid, verb)
    ]])
    -- EOF S.object_utils

    --- S.string_utils
    local string_utils = db:create()
    S.string_utils = string_utils

    string_utils:add_verb({system.uuid, "r", {"words"}}, {"any"})
    string_utils:set_verb_code("words", [[
        return pl.stringx.split(args[1])
    ]])

    string_utils:add_verb({system.uuid, "r", {"from_list"}}, {"any"})
    string_utils:set_verb_code("from_list", [[
        local list, delimiter = table.unpack(args)
        return table.concat(list, delimiter)
    ]])

    string_utils:add_verb({system.uuid, "r", {"match_object"}}, {"this", "none", "this"})
    string_utils:set_verb_code("match_object", [[
        -- :match_object(string,location[,someone])
        -- Returns the object matching the given string for someone, on the assumption that s/he is in the given location.  `someone' defaults to player.
        -- This first tries :literal_object(string), \"me\"=>someone,\"here\"=>location, then player:match(string) and finally location:match(string) if location is valid.
        -- This is the default algorithm for use by room :match_object() and player :my_match_object() verbs.  Player verbs that are calling this directly should probably be calling :my_match_object instead.
        local string, here, who = table.unpack(args)
        if who == nil then
            who = player
        end

        local object = this:literal_object{string}
        if S.failed_match ~= object then
            return object
        elseif string == "me" then
            return who
        elseif string == "here" then
            return here
        end

        local pobject = who:match{string}
        if M.valid(pobject) and listcontains({pobject.name, table.unpack(pobject.aliases)}, string) or not M.valid(here) then
            -- ...exact match in player, or room is bogus...
            return pobject;
        end

        local hobject = here:match{string}
        if M.valid(hobject) and listcontains({hobject.name, table.unpack(hobject.aliases)}, string) or pobject == S.failed_match then
            -- ...exact match in room, or match in player failed completely...
            return hobject
        else
            return pobject
        end
    ]])

    string_utils:add_verb({system.uuid, "r", {"match"}}, {"this", "none", "this"})
    string_utils:set_verb_code("match", [[
    -- Each obj-list should be a list of objects or a single object, which is treated as if it were a list of that object.  Each prop-name should be string naming a property on every object in the corresponding obj-list.  The value of that property in each case should be either a string or a list of strings.
    -- The argument string is matched against all of the strings in the property values.
    -- If it exactly matches exactly one of them, the object containing that property is returned.  If it exactly matches more than one of them, $ambiguous_match is returned.
    -- If there are no exact matches, then partial matches are considered, ones in which the given string is a prefix of some property string.  Again, if exactly one match is found, the object with that property is returned, and if there is more than one match, $ambiguous_match is returned.
    -- Finally, if there are no exact or partial matches, then $failed_match is returned.
    local subject = args[1]
    if subject == "" then
        return S.nothing
    end
    local no_exact_match = nil
    local no_partial_match = nil
    for i = 1, #args / 2 do
        local prop_name = args[2 * i + 1]
        local olist = args[2 * i]
        for j, object in ipairs(type(olist) == "table" and olist or {olist}) do
            if M.valid(object) then
                local str_list = object[prop_name] or {}
                -- TODO handle E_PERM, E_PROPNF => {}
                if type(str_list) ~= "table" then
                    str_list = {str_list}
                end
                if listcontains(str_list, subject) then
                    if no_exact_match == nil then
                        no_exact_match = object
                    elseif no_exact_match ~= object then
                        return S.ambiguous_match
                    end
                else
                    for i, string in ipairs(str_list) do
                        if string.find(string, subject, 1, true) ~= 1 then
                        elseif no_partial_match == nil then
                            no_partial_match = object
                        elseif no_partial_match ~= object then
                            no_partial_match = S.ambiguous_match
                        end
                    end
                end
            end
        end
    end
    return no_exact_match or (no_partial_match or S.failed_match)
    ]])

    string_utils:add_verb({system.uuid, "r", {"literal_object"}}, {"this", "none", "this"})
    string_utils:set_verb_code("literal_object", [[
    -- Matches args[1] against literal objects: #xxxxx, $variables, *mailing-lists, and username.  Returns the object if successful, $failed_match else.
    -- TODO this is currently a partial implementation
    local string = args[1]
    if #string == 0 then
      return S.nothing
    end
    local object = S.code_utils:toobj{string}
    if object ~= nil and not M.Error:class_of(object) then
      return object;
    end
    return S.failed_match;
    -- elseif (string[1] == "~")
    --   return this:match_player(string[2..$], #0);
    -- elseif (string[1] == "*" && length(string) > 1)
    --   return $mail_agent:match_recipient(string);
    -- elseif (string[1] == "$")
    --   string[1..1] = "";
    --   object = #0;
    --   while (pn = string[1..(dot = index(string, ".")) ? dot - 1 | $])
    --     if (!$object_utils:has_property(object, pn) || typeof(object = object.(pn)) != OBJ)
    --       return $failed_match;
    --     endif
    --     string = string[length(pn) + 2..$];
    --   endwhile
    --   if (object == #0 || typeof(object) == ERR)
    --     return $failed_match;
    --   else
    --     return object;
    --   endif
    -- else
    --   return $failed_match;
    -- endif
    ]])
    -- EOF S.string_utils

    --- S.command_utils
    local command_utils = db:create()
    S.command_utils = command_utils

    -- TODO impl
    command_utils:add_verb({system.uuid, "r", {"object_match_failed"}}, {"any"})
    command_utils:set_verb_code("object_match_failed", [[
        -- Usage: object_match_failed(object, string)
        -- Prints a message if string does not match object.  Generally used after object is derived from a :match_object(string).
        local match_result, string = table.unpack(args)
        -- TODO: tell = $perm_utils:controls(caller_perms(), player) ? "notify" | "tell";
        local tell = pl.func.bind1(player.tell, player)
        if is_uuid(string) and S.code_utils:toobj{string} ~= M.E_TYPE then
          -- ...avoid the `I don't know which `#-2' you mean' message...
          if M.valid(match_result) then
            tell{M.tostr(string, " does not exist.")}
          end
          return not M.valid(match_result)
        elseif match_result == S.nothing then
          tell{"You must give the name of some object."}
        elseif match_result == S.failed_match then
          tell{M.tostr("I see no \"", string, "\" here.")}
        elseif match_result == S.ambiguous_match then
          tell{M.tostr("I don't know which \"", string, "\" you mean.")}
        elseif not M.valid(match_result) then
          tell{M.tostr(match_result, " does not exist.")}
        else
          return false
        end
        return true
    ]])

    -- TODO impl
    command_utils:add_verb({system.uuid, "r", {"dump_lines"}}, {"any"})
    command_utils:set_verb_code("dump_lines", [[
        -- :dump_lines(text) => text `.'-quoted for :read_lines()
        --  text is assumed to be a list of strings
        --  Returns a corresponding list of strings which, when read via :read_lines, 
        --  produces the original list of strings (essentially, any strings beginning 
        --  with a period "." have the period doubled).
        --  The list returned includes a final "."
        -- TODO original implementation has some magic I don't understand, review that
        local text = args[1]
        return pl.List(text):map(function (line)
            if string.sub(line, 1, 1) == "." then
                return "." .. line
            else
                return line
            end
        end):append(".")
    ]])
    -- EOF S.command_utils

    --- S.verb_editor
    local verb_editor = db:create()
    S.verb_editor = verb_editor

    -- TODO full impl
    verb_editor:add_verb({system.uuid, "r", {"get_room"}}, {"any"})
    verb_editor:set_verb_code("get_room", [[
        local object = args[1]
        return object.location
    ]])

    -- EOF verb_editor

    local Root = db:create()
    Root.name = "root object"
    system.Root = Root.uuid

    Root:add_verb({system.uuid, "rx", {"match"}}, {"this", "none", "this"})
    Root:set_verb_code("match", [[
        local c = this.contents
        return S.string_utils:match{args[1], c, "name", c, "aliases"}
    ]])

    Root:add_verb({system.uuid, "r", {"get_name"}}, {"any"})
    Root:set_verb_code("get_name", [[
        return this.name
    ]])

    Root:add_verb({system.uuid, "r", {"title"}}, {"any"})
    Root:set_verb_code("title", [[
        local name = this:get_name()
        if type(name) == "string" and #name > 0 then
            return name
        end
        return this.uuid
    ]])

    Root:add_verb({system.uuid, "r", {"tell"}}, {"any"})
    Root:set_verb_code("tell", [[
        this:notify(Moo.tostr(args))
    ]])

    --- S.Player
    local Player = db:create()
    S.Player = Player
    Player.name = "generic player"
    Player:chparent(Root)

    -- TODO impl
    Player:add_verb({system.uuid, "rx", {"my_match_object"}}, {"any"})
    Player:set_verb_code("my_match_object", [[
        -- :my_match_object(string [,location])
        return S.string_utils:match_object(
            pl.List(args):append(this.location):slice(1, 2):append(this)
        )
    ]])

    -- TODO at some point this needs to move to a "generic programmer" object
    Player:add_verb({S.uuid, "rx", {"@edit"}}, {"any", "any", "any"})
    Player:set_verb_code("@edit", [[
-- Calls the verb editor on verbs, the note editor on properties, and on anything else assumes it's an object for which you want to edit the .description.

-- Placeholder until the rest of the machinery is in place
    S.webclient:invoke{argstr, verb}
-- EOF placeholder

--local len = player.linelen
--if not args then
--  (player in $note_editor.active ? $note_editor | $verb_editor):invoke(dobjstr, verb);
--elseif ($code_utils:parse_verbref(args[1]))
--  if (player.programmer)
--    #480:invoke(argstr, verb);
--    player:tell("invoke done");
--  else
--    player:notify("You need to be a programmer to do this.");
--    player:notify("If you want to become a programmer, talk to a wizard.");
--    return;
--  endif
--elseif ($list_editor:is_valid(dobjstr))
--  $list_editor:invoke(dobjstr, verb);
--else
--  $note_editor:invoke(dobjstr, verb);
--endif
--if len then
--    player.linelen = len
--end
--"player.linelen = len;"
]])

    -- TODO impl
    Player:add_verb({S.uuid, "rx", {"edit_option"}}, {})
    -- EOF Player

    local Room = db:create()
    Room.name = "Prototype:Room"
    Room.description = "A nondescript room"
    Room:chparent(Root)

    Room:add_verb({system.uuid, "r", {"announce"}}, {"any"})
    Room:set_verb_code("announce", [[
        for i, target in ipairs(this.contents:without(player)) do
            pcall(target.tell, target, args)
        end
    ]])

    Room:add_verb({system.uuid, "r", {"announce_all"}}, {"any"})
    Room:set_verb_code("announce_all", [[
        for i, target in ipairs(this.contents) do
            pcall(target.tell, target, args)
        end
    ]])

    Room:add_verb({system.uuid, "rx", {"say"}}, {"any"})
    Room:set_verb_code("say", [[
        pcall(function()
            -- TODO player should really be caller here once implemented
            player:tell{'You say, "%s"' % {argstr}}
            this:announce{'$name says, "$msg"' % {name = player.name, msg = argstr}}
        end)
    ]])

    Room:add_verb({system.uuid, "rx", {"emote"}}, {"any"})
    Room:set_verb_code("emote", [[
        -- TODO player should really be caller here once implemented
        this:announce_all{'%s %s' % {player.name, argstr}}
    ]])

    Room:add_verb({system.uuid, "rx", {"describe"}}, {})
    Room:set_verb_code("describe", [[
        local name = this:title()
        local description = this.description or "You see nothing special."
        local msg = '%s\n%s' % {name, description}

        local seen = this.contents:without(player):map(_1.name)
        if #seen > 0 then
            msg = msg .. "\nYou see here: " .. table.concat(seen, ", ")
        end

        return msg
    ]])

    Room:add_verb({system.uuid, "rx", {"look"}}, {})
    Room:set_verb_code("look", [[
        player:notify(this:describe())
    ]])

    local void = db:create()
    void.name = "The Void"
    void.description = "You float in nothing."
    void:chparent(Room)

    system.starting_room = void.uuid

    local test = db:create()
    test.name = "testobj"
    test:move(void)
    test:add_verb({system.uuid, "rx", {"wiggle"}}, {})
    test:set_verb_code("wiggle", [[
        this:emote{"looks around"}
        this:emote{"wiggles"}
    ]])
end)()
