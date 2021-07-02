-- :match_object(string,location[,someone])
-- Returns the object matching the given string for someone, on the assumption that s/he is in the given location.  `someone' defaults to player.
-- This first tries :literal_object(string), \"me\"=>someone,\"here\"=>location, then player:match(string) and finally location:match(string) if location is valid.
-- This is the default algorithm for use by room :match_object() and player :my_match_object() verbs.  Player verbs that are calling this directly should probably be calling :my_match_object instead.
local string, here, who = table.unpack(args)
if who == nil then
    who = player
end
pl.utils.assert_string(1, string)
assert_class_of(2, here, ObjectProxy)
assert_class_of(3, who, ObjectProxy)

local object = this:literal_object(string)
if S.failed_match ~= object then
    return object
elseif string == "me" then
    return who
elseif string == "here" then
    return here
end

local pobject = who:match(string)
if valid(pobject) and List{pobject.name}:extend(pobject.aliases):contains(string) or not valid(here) then
    -- ...exact match in player, or room is bogus...
    return pobject
end

local hobject = here:match(string)
if valid(hobject) and List{hobject.name}:extend(hobject.aliases):contains(string) or pobject == S.failed_match then
    -- ...exact match in room, or match in player failed completely...
    return hobject
else
    return pobject
end
