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
