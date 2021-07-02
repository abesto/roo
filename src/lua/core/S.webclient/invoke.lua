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
