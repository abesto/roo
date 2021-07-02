-- :parse_invoke(string, v)
-- string is the commandline string to parse to obtain the obj:verb to edit
--  v is the actual command verb used to invoke the editor
-- => {object, verbname, verb_code} or error

local vref = S.string_utils:words(args[1])
local spec = S.code_utils:parse_verbref(vref[1])
if not vref or spec == 0 then
  player:tell("Usage: %s %s" % {args[2], " object:verb"})
  return
end
local argspec = List(vref):slice(2)
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
player:tell(toliteral(spec))
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
  local code
  if vnum ~= nil then
    code = this:fetch_verb_code(object, vnum)
  else
    code = E_VERBNF
  end
  if is_error(code) then
    player:tell((code ~= E_VERBNF) and code or "That object does not define that verb", argspec and " with those args." or ".")
    return code
  else
    return {object, argspec and {vname, table.unpack(argspec)} or vname, code}
  end
end
return 0
