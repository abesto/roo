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
  notify(player, line):unwrap()
end
