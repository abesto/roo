-- TODO full impl
local candidates, name = table.unpack(args)
assert_arg(1, candidates, 'table', nil, is_indexable)
assert_string(2, name)
return List(candidates):contains(name)
