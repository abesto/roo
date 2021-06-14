(function()
    if S.minimal_core_loaded then
        return
    end
    S.minimal_core_loaded = true

    system:add_verb({system.uuid, "", {"do_login_command"}}, {})
    system:set_verb_code("do_login_command", [=[
        player = create(S.Player):unwrap()
        player.owner = player
        player:move(S.starting_room)
        player.name = "guest"

        return player.uuid
        ]=])

    --- S.code_utils
    S.code_utils = create(S.nothing):unwrap()

    -- TODO impl
    S.code_utils:add_verb({system.uuid, "rx", {"short_prep"}}, {"any"})
    S.code_utils:set_verb_code("short_prep", [[
        return args[1]
    ]])

    -- TODO impl
    S.code_utils:add_verb({system.uuid, "rx", {"full_prep"}}, {"any"})
    S.code_utils:set_verb_code("full_prep", [[
        return nil
    ]])

    S.code_utils:add_verb({system.uuid, "rx", {"toobj"}}, {"any"})
    S.code_utils:set_verb_code("toobj", [[
        -- TODO this may need some extra logic
        return toobj(args[1]):unwrap_unsafe()
    ]])

    S.code_utils:add_verb({system.uuid, "r", {"parse_verbref"}}, {"this", "none", "this"})
    S.code_utils:set_verb_code("parse_verbref", [[
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
                if not is_type(p, ObjectProxy) then
                    return 0
                end
                object = p.uuid
            end
            return {object, verbname}
        else
            return 0
        end
    ]])

    S.code_utils:add_verb({system.uuid, "r", {"parse_argspec"}}, {"any"})
    S.code_utils:set_verb_code("parse_argspec", [[
-- :parse_arg_spec(@args)
--  attempts to parse the given sequence of args into a verb_arg specification
--  returns {verb_args,remaining_args} if successful.
--  e.g., :parse_arg_spec(\"this\",\"in\",\"front\",\"of\",\"any\",\"foo\"..)
--           => {{\"this\",\"in front of\",\"any\"},{\"foo\"..}}
--  returns a string error message if parsing fails.
local nargs = #args
local args = List(args)
if nargs < 1 then
  return {{}, {}}
end

local ds = args[1]
if args[1] == "tnt" then
  return {{"this", "none", "this"}, args:slice(2)}
elseif not List{"this", "any", "none"}:contains(ds) then
  return '"%s" is not a valid direct object specifier.' % {ds}
elseif nargs < 2 or List{"none", "any"}:contains(args[2]) then
  local verbargs = args:slice(1, min(3, nargs))
  local rest = args:slice(4, nargs);
end

local gp = List(S.code_utils:get_prep(unpack(args:slice(2, nargs))))
if not gp[1] then
  return '"%s" is not a valid preposition.' % {args[2]}
else
  local nargs = #gp
  local verbargs = List{ds}:extend(gp:slice(1, min(2, nargs)))
  rest = gp:slice(3, nargs)
end

if #verbargs >= 3 and not List{"this", "any", "none"}:contains(verbargs[3]) then
  return '"%s" is not a valid indirect object specifier.' % {verbargs[3]}
end
return {verbargs, rest};
    ]])

    S.code_utils:add_verb({system.uuid, "r", {"find_verb_named"}}, {"any"})
    S.code_utils:set_verb_code("find_verb_named", [[
        -- :find_verb_named(object,name[,n])
        --  returns the *number* of the first verb on object matching the given name.
        --  optional argument n, if given, starts the search with verb n,
        --  causing the first n verbs (1..n-1) to be ignored.
        --  nil is returned if no verb is found.
        --  This routine does not find inherited verbs.
        local object, name, start = unpack(args)
        assert_object(1, object)
        assert_string(2, name)

        if start == nil then
            start = 1
        end
        assert_arg(3, start, "number")

        return verbs(object):map(function (object_verbs)
            for i = start, #object_verbs do
                local verbinfo = verb_info(object, i):unwrap()
                if this:verbname_match(verbinfo[3], name) then
                    return i
                end
            end
        end):unwrap_or(nil)
    ]])

    -- TODO full impl
    S.code_utils:add_verb({system.uuid, "rx", {"verbname_match"}}, {"any"})
    S.code_utils:set_verb_code("verbname_match", [[
        local candidates, name = table.unpack(args)
        assert_arg(1, candidates, 'table', nil, is_indexable)
        assert_string(2, name)
        return List(candidates):contains(name)
    ]])
    -- EOF S.code_utils

    --- S.object_utils
    S.object_utils = create(S.nothing):unwrap()

    S.object_utils:add_verb({system.uuid, "r", {"has_verb"}}, {"any"})
    S.object_utils:set_verb_code("has_verb", [[
        local object, verb = table.unpack(args)
        return db:has_verb_with_name(object.uuid, verb)
    ]])
    -- EOF S.object_utils

    --- S.string_utils
    S.string_utils = create(S.nothing):unwrap()

    S.string_utils:add_verb({system.uuid, "r", {"words"}}, {"any"})
    S.string_utils:set_verb_code("words", [[
        return pl.stringx.split(args[1])
    ]])

    S.string_utils:add_verb({system.uuid, "r", {"from_list"}}, {"any"})
    S.string_utils:set_verb_code("from_list", [[
        local list, delimiter = table.unpack(args)
        return table.concat(list, delimiter)
    ]])

    S.string_utils:add_verb({system.uuid, "r", {"match_object"}}, {"this", "none", "this"})
    S.string_utils:set_verb_code("match_object", [[
        -- :match_object(string,location[,someone])
        -- Returns the object matching the given string for someone, on the assumption that s/he is in the given location.  `someone' defaults to player.
        -- This first tries :literal_object(string), \"me\"=>someone,\"here\"=>location, then player:match(string) and finally location:match(string) if location is valid.
        -- This is the default algorithm for use by room :match_object() and player :my_match_object() verbs.  Player verbs that are calling this directly should probably be calling :my_match_object instead.
        local string, here, who = table.unpack(args)
        if who == nil then
            who = player
        end
        pl.utils.assert_string(1, string)
        assert_class_of(2, here, ObjectProxy)
        assert_class_of(3, who, ObjectProxy)

        local object = this:literal_object(string)
        if S.failed_match ~= object then
            return object
        elseif string == "me" then
            return who
        elseif string == "here" then
            return here
        end

        local pobject = who:match(string)
        if valid(pobject) and List{pobject.name}:extend(pobject.aliases):contains(string) or not valid(here) then
            -- ...exact match in player, or room is bogus...
            return pobject
        end

        local hobject = here:match(string)
        if valid(hobject) and List{hobject.name}:extend(hobject.aliases):contains(string) or pobject == S.failed_match then
            -- ...exact match in room, or match in player failed completely...
            return hobject
        else
            return pobject
        end
    ]])

    S.string_utils:add_verb({system.uuid, "r", {"match"}}, {"this", "none", "this"})
    S.string_utils:set_verb_code("match", [[
    -- Each obj-list should be a list of objects or a single object, which is treated as if it were a list of that object.  Each prop-name should be string naming a property on every object in the corresponding obj-list.  The value of that property in each case should be either a string or a list of strings.
    -- The argument string is matched against all of the strings in the property values.
    -- If it exactly matches exactly one of them, the object containing that property is returned.  If it exactly matches more than one of them, $ambiguous_match is returned.
    -- If there are no exact matches, then partial matches are considered, ones in which the given string is a prefix of some property string.  Again, if exactly one match is found, the object with that property is returned, and if there is more than one match, $ambiguous_match is returned.
    -- Finally, if there are no exact or partial matches, then $failed_match is returned.
    local subject = args[1]
    assert_class_of(0, this, ObjectProxy)
    assert_string(1, subject)
    
    if subject == "" then
        return S.nothing
    end
    local no_exact_match = nil
    local no_partial_match = nil
    for i = 1, #args / 2 do
        local prop_name = args[2 * i + 1]
        local olist = args[2 * i]
        for j, object in ipairs(is_indexable(olist) and olist or {olist}) do
            if valid(object) then
                local str_list = object[prop_name]
                if is_indexable(str_list) and not List:class_of(str_list) then
                    str_list = List(str_list)
                elseif not str_list then
                    str_list = List()
                end
                -- TODO handle E_PERM, E_PROPNF => {}
                if not is_indexable(str_list) then
                    str_list = List{str_list}
                end
                if str_list:contains(subject) then
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

    S.string_utils:add_verb({system.uuid, "r", {"literal_object"}}, {"this", "none", "this"})
    S.string_utils:set_verb_code("literal_object", [[
    -- Matches args[1] against literal objects: #xxxxx, $variables, *mailing-lists, and username.  Returns the object if successful, $failed_match else.
    -- TODO this is currently a partial implementation
    local string = args[1]
    if #string == 0 then
      return S.nothing
    end
    local object = S.code_utils:toobj(string)
    if object ~= nil and not is_error(object) then
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
    S.command_utils = create(S.nothing):unwrap()

    -- TODO impl
    S.command_utils:add_verb({system.uuid, "r", {"object_match_failed"}}, {"any"})
    S.command_utils:set_verb_code("object_match_failed", [[
        -- Usage: object_match_failed(object, string)
        -- Prints a message if string does not match object.  Generally used after object is derived from a :match_object(string).
        local match_result, string = table.unpack(args)
        assert_class_of(1, match_result, ObjectProxy)
        assert_string(2, string)

        -- TODO: tell = $perm_utils:controls(caller_perms(), player) ? "notify" | "tell";
        local tell = bind1(player.tell, player)
        if is_uuid(string) and S.code_utils:toobj(string) ~= E_TYPE then
          -- ...avoid the `I don't know which `#-2' you mean' message...
          if not valid(match_result) then
            tell("%s does not exist." % {string})
          end
          return not valid(match_result)
        elseif match_result == S.nothing then
          tell("You must give the name of some object.")
        elseif match_result == S.failed_match then
          tell('I see no "%s" here.' % {string})
        elseif match_result == S.ambiguous_match then
          tell('I don\'t know which "%s" you mean.' % {string})
        elseif not valid(match_result) then
          tell("%s does not exist." % {match_result})
        else
          return false
        end
        return true
    ]])

    S.command_utils:add_verb({system.uuid, "r", {"dump_lines"}}, {"any"})
    S.command_utils:set_verb_code("dump_lines", [[
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
    S.verb_editor = create(S.nothing):unwrap()

    -- TODO full impl
    S.verb_editor:add_verb({system.uuid, "r", {"get_room"}}, {"any"})
    S.verb_editor:set_verb_code("get_room", [[
        local object = args[1]
        return object.location
    ]])

    -- EOF verb_editor

    S.Root = create(S.nothing):unwrap()
    S.Root.name = "root object"

    S.Root:add_verb({system.uuid, "rx", {"match"}}, {"this", "none", "this"})
    S.Root:set_verb_code("match", [[
        local c = this.contents
        return S.string_utils:match(args[1], c, "name", c, "aliases")
    ]])

    S.Root:add_verb({system.uuid, "r", {"get_name"}}, {"any"})
    S.Root:set_verb_code("get_name", [[
        return this.name
    ]])

    S.Root:add_verb({system.uuid, "r", {"title"}}, {"any"})
    S.Root:set_verb_code("title", [[
        local name = this:get_name()
        if is_type(name, "string") and #name > 0 then
            return name
        end
        return this.uuid
    ]])

    S.Root:add_verb({system.uuid, "r", {"tell"}}, {"any"})
    S.Root:set_verb_code("tell", [[
        this:notify(tostr(args))
    ]])

    --- S.Player
    S.Player = create(S.Root):unwrap()
    S.Player.name = "generic player"

    -- TODO impl
    S.Player:add_verb({system.uuid, "rx", {"my_match_object"}}, {"any"})
    S.Player:set_verb_code("my_match_object", [[
        -- :my_match_object(string [,location])
        return S.string_utils:match_object(unpack(
            pl.List(args):append(this.location):slice(1, 2):append(this)
        ))
    ]])

    -- TODO at some point this needs to move to a "generic programmer" object
    S.Player:add_verb({S.uuid, "rx", {"@edit"}}, {"any", "any", "any"})
    S.Player:set_verb_code("@edit", [[
-- Calls the verb editor on verbs, the note editor on properties, and on anything else assumes it's an object for which you want to edit the .description.

-- Placeholder until the rest of the machinery is in place
    S.webclient:invoke(argstr, verb)
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
    S.Player:add_verb({S.uuid, "rx", {"edit_option"}}, {})
    -- EOF Player

    -- TODO full impl
    S.Player:add_verb({S.uuid, "rx", {"@program"}}, {"this"})
    S.Player:set_verb_code("@program", [[
local vref = List(args)

local spec = S.code_utils:parse_verbref(vref[1])
if not vref or spec == 0 then
  player:tell("Usage: %s %s" % {"@program object:verb argspec"})
  return
end

local argspec = vref:slice(2)
if #argspec > 0 then
  local pas = S.code_utils:parse_argspec(unpack(argspec))
  if type(pas) == "table" then
    if pas[2] and #pas[2] > 0 then
      player:tell('I don\'t understand "%s"' % {S.string_utils:from_list(pas[2], " ")})
      return
    end
    argspec = pl.List(pas[1]):extend{"none", "none"}:slice(1, 3)
    argspec[2] = S.code_utils:full_prep(argspec[2]) or argspec[2]
  else
    player:tell(toliteral(pas))
    return
  end
end

local object = player:my_match_object(spec[1], S.verb_editor:get_room(player))
if not S.command_utils:object_match_failed(object, spec[1]) then
  local vname = spec[2]
  local vnum = S.code_utils:find_verb_named(object, vname)
  if #argspec > 0 then
    -- TODO may need deep table comparison here
    while vnum and (object:verb_args(vnum) ~= argspec) do
      vnum = S.code_utils:find_verb_named(object, vname, vnum + 1)
    end
  end

  player:tell("Now programming %s:%d" % {object.uuid, vnum})

  local done = false
  local lines = List()
  while not done do
      local line = read()
      if line == '.' then
          done = true
      else
          lines:append(line)
      end
  end

  local result = set_verb_code(object, vnum, lines)
  if result:is_ok() then
    player:tell("Program saved.")
  else
    player:tell(toliteral(result:err()))
  end
end
    ]])

    S.Room = create(S.Root):unwrap()
    S.Room.name = "Prototype:Room"
    S.Room.description = "A nondescript room"

    S.Room:add_verb({system.uuid, "r", {"announce"}}, {"any"})
    S.Room:set_verb_code("announce", [[
        for i, target in ipairs(this.contents:without(player)) do
            pcall(target.tell, target, unpack(args))
        end
    ]])

    S.Room:add_verb({system.uuid, "r", {"announce_all"}}, {"any"})
    S.Room:set_verb_code("announce_all", [[
        for i, target in ipairs(this.contents) do
            pcall(target.tell, target, unpack(args))
        end
    ]])

    S.Room:add_verb({system.uuid, "rx", {"say"}}, {"any"})
    S.Room:set_verb_code("say", [[
        pcall(function()
            -- TODO player should really be caller here once implemented
            player:tell('You say, "%s"' % {argstr})
            this:announce('$name says, "$msg"' % {name = player.name, msg = argstr})
        end)
    ]])

    S.Room:add_verb({system.uuid, "rx", {"emote"}}, {"any"})
    S.Room:set_verb_code("emote", [[
        -- TODO player should really be caller here once implemented
        this:announce_all('%s %s' % {player.name, argstr})
    ]])

    S.Room:add_verb({system.uuid, "rx", {"describe"}}, {})
    S.Room:set_verb_code("describe", [[
        local name = this:title()
        local description = this.description or "You see nothing special."
        local msg = '%s\n%s' % {name, description}

        local seen = this.contents:without(player):map(_1.name)
        if #seen > 0 then
            msg = msg .. "\nYou see here: " .. table.concat(seen, ", ")
        end

        return msg
    ]])

    S.Room:add_verb({system.uuid, "rx", {"look"}}, {})
    S.Room:set_verb_code("look", [[
        player:notify(this:describe())
    ]])

    S.starting_room = create(S.Room):unwrap()
    S.starting_room.name = "The Void"
    S.starting_room.description = "You float in nothing."

    S.starting_room:add_verb({system.uuid, "rx", {"wiggle"}}, {})
    S.starting_room:set_verb_code("wiggle", [[
        this:announce_all("%s wiggles" % {this.name})
    ]])
end)()
