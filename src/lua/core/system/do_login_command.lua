local command = assert_string(1, args[1])
if command ~= "connect" then
    player:notify('Only the "connect" command is currently supported during login')
    return nil
end

local name = assert_string(2, args[2])

for i, candidate in ipairs(players()) do
    if candidate.name == name then
        player:notify("Welcome back, %s" % {name})
        return candidate.uuid
    end
end

player:notify("Welcome, %s" % {name}):unwrap()
local new = create(S.Player, S.nothing):unwrap()
new:set_player_flag(true):unwrap()
new.owner = new
new:move(S.starting_room):unwrap()
new.name = name

return new.uuid