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
    if object == "S" then
        object = S.uuid
    end
    return {object, verbname}
else
    return 0
end
