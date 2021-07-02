pcall(function()
    -- TODO player should really be caller here once implemented
    player:tell('You say, "%s"' % {argstr})
    this:announce('$name says, "$msg"' % {name = player.name, msg = argstr})
end)
