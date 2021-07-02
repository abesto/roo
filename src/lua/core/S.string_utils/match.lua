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
