local object, verb = table.unpack(args)
return db:has_verb_with_name(object.uuid, verb):unwrap()