-- Calls the verb editor on verbs, the note editor on properties, and on anything else assumes it's an object for which you want to edit the .description.

-- Placeholder until the rest of the machinery is in place
    S.webclient:invoke(argstr, verb)
-- EOF placeholder

--local len = player.linelen
--if not args then
--  (player in $note_editor.active ? $note_editor | $verb_editor):invoke(dobjstr, verb);
--elseif ($code_utils:parse_verbref(args[1]))
--  if (player.programmer)
--    #480:invoke(argstr, verb);
--    player:tell("invoke done");
--  else
--    player:notify("You need to be a programmer to do this.");
--    player:notify("If you want to become a programmer, talk to a wizard.");
--    return;
--  endif
--elseif ($list_editor:is_valid(dobjstr))
--  $list_editor:invoke(dobjstr, verb);
--else
--  $note_editor:invoke(dobjstr, verb);
--endif
--if len then
--    player.linelen = len
--end
--"player.linelen = len;"
