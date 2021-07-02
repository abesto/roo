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
