-- Rough translation of https://github.com/SevenEcks/lambda-moo-programming/blob/master/code/LocalEditing.md
local editor = create(S.nothing):unwrap()
S.webclient = editor
editor.key = 0
editor.aliases = {"Webclient Package"}
editor.description =
    "This is a placeholder parent for all the $..._utils packages, to more easily find them and manipulate them. At present this object defines no useful verbs or properties. (Filfre.)"
editor.object_size = {0, 0}

editor:add_verb({S.uuid, "rx", {"local_editing_info"}}, {"this", "none", "this"}):unwrap()
editor:add_verb({S.uuid, "rx", {"invoke"}}, {"this", "none", "this"}):unwrap()
editor:add_verb({S.uuid, "rx", {"parse_invoke"}}, {"this", "none", "this"}):unwrap()
editor:add_verb({S.uuid, "rx", {"fetch_verb_code"}}, {"this", "none", "this"}):unwrap()
editor:add_verb({S.uuid, "rx", {"invoke_local_editor"}}, {"this", "none", "this"}):unwrap()
editor:add_verb({S.uuid, "rx", {"local_instruction"}}, {"this", "none", "this"}):unwrap()
