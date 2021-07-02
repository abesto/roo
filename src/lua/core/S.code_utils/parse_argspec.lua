-- :parse_arg_spec(@args)
--  attempts to parse the given sequence of args into a verb_arg specification
--  returns {verb_args,remaining_args} if successful.
--  e.g., :parse_arg_spec(\"this\",\"in\",\"front\",\"of\",\"any\",\"foo\"..)
--           => {{\"this\",\"in front of\",\"any\"},{\"foo\"..}}
--  returns a string error message if parsing fails.
local nargs = #args
local args = List(args)
if nargs < 1 then
  return {{}, {}}
end

local ds = args[1]
if args[1] == "tnt" then
  return {{"this", "none", "this"}, args:slice(2)}
elseif not List{"this", "any", "none"}:contains(ds) then
  return '"%s" is not a valid direct object specifier.' % {ds}
elseif nargs < 2 or List{"none", "any"}:contains(args[2]) then
  local verbargs = args:slice(1, min(3, nargs))
  local rest = args:slice(4, nargs);
end

local gp = List(S.code_utils:get_prep(unpack(args:slice(2, nargs))))
if not gp[1] then
  return '"%s" is not a valid preposition.' % {args[2]}
else
  local nargs = #gp
  local verbargs = List{ds}:extend(gp:slice(1, min(2, nargs)))
  rest = gp:slice(3, nargs)
end

if #verbargs >= 3 and not List{"this", "any", "none"}:contains(verbargs[3]) then
  return '"%s" is not a valid indirect object specifier.' % {verbargs[3]}
end
return {verbargs, rest};
