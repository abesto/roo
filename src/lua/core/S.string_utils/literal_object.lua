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
