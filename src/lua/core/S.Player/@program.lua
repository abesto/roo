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
