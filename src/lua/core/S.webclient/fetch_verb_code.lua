set_task_perms(player)
player:notify(toliteral(args)):unwrap()
return verb_code(args[1], args[2], not player:edit_option("no_parens")):unwrap_or({})
