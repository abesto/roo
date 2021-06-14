-- Rough translation of https://github.com/SevenEcks/lambda-moo-programming/blob/master/code/LocalEditing.md
local editor = create(S.nothing):unwrap()
S.webclient = editor
editor.key = 0
editor.aliases = {"Webclient Package"}
editor.description =
    "This is a placeholder parent for all the $..._utils packages, to more easily find them and manipulate them. At present this object defines no useful verbs or properties. (Filfre.)"
editor.object_size = {0, 0}

editor:add_verb({S.uuid, "rx", {"local_editing_info"}}, {"this", "none", "this"})
editor:set_verb_code("local_editing_info", [[
local object, vname, code = table.unpack(args)
local vargs
if is_type(vname, "table") then
  vargs = " %s %s %s" % {vname[2], S.code_utils:short_prep(vname[3]), vname[4]}
  vname = vname[1]
else
  vargs = ""
end
local name = "%s:%s" % {object.name, vname};
-- TODO swap to full @program invocation once we have proper dobj, prep, iobj support
-- local upload = "@program %s:%s %s" % {object.uuid, vname, vargs}
local upload = "@program %s:%s" % {object.uuid, vname}
return {name, code, upload};
]])

editor:add_verb({S.uuid, "rx", {"invoke"}}, {"this", "none", "this"})
editor:set_verb_code("invoke", [[
-- :invoke(...)
-- to find out what arguments this verb expects,
-- see this editor's parse_invoke verb.
local new = args[1]
local spec = this:parse_invoke(unpack(args))
if type(spec) == "table" and not is_error(spec) then
  local info = this:local_editing_info(unpack(spec))
  -- TODO impl has_verb, then uncomment
  --if S.object_utils:has_verb(this, "local_editing_info") and info then
    player:tell("Invoking local editor")
    this:invoke_local_editor(unpack(info))
  --else
    --player:tell("This is for editing in a web client, if you don't wanna do that, use a different verb.");
  --end
end
]])

editor:add_verb({S.uuid, "rx", {"parse_invoke"}}, {"this", "none", "this"})
editor:set_verb_code("parse_invoke", [[
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
]])

editor:add_verb({S.uuid, "rx", {"fetch_verb_code"}}, {"this", "none", "this"})
editor:set_verb_code("fetch_verb_code", [[
set_task_perms(player)
return verb_code(args[1], args[2], not player:edit_option("no_parens")):unwrap_or("")
]])

editor:add_verb({S.uuid, "rx", {"invoke_local_editor"}}, {"this", "none", "this"})
editor:set_verb_code("invoke_local_editor", [[
-- :invoke_local_editor(name, text, upload)
-- Spits out the magic text that invokes the local editor in the player's client."
-- NAME is a good human-readable name for the local editor to use for this particular piece of text."
-- TEXT is a string or list of strings, the initial body of the text being edited."
-- UPLOAD, a string, is a MOO command that the local editor can use to save the text when the user is done editing.  The local editor is going to send that command on a line by itself, followed by the new text lines, followed by a line containing only `.'.  The UPLOAD command should therefore call $command_utils:read_lines() to get the new text as a list of strings."

-- TODO re-enable caller checking once caller is implemented
--if caller ~= this then
--   return
--end

local name, text, upload = table.unpack(args)
assert_string(1, name)
if is_type(text, "string") then
  text = {text}
end
this:local_instruction(name, upload)
-- :dump_lines() takes care of the final `.' ...
for i, line in ipairs(S.command_utils:dump_lines(text)) do
  notify(player, line)
end
]])

editor:add_verb({S.uuid, "rx", {"local_instruction"}}, {"this", "none", "this"})
editor:set_verb_code("local_instruction", [[
local label, upload = table.unpack(args)
if not upload then
    upload = "none"
end
local msg = "#$# edit name: %s upload: %s" % {label, upload}
player:tell(msg)
]])
