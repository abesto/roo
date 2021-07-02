-- :dump_lines(text) => text `.'-quoted for :read_lines()
--  text is assumed to be a list of strings
--  Returns a corresponding list of strings which, when read via :read_lines, 
--  produces the original list of strings (essentially, any strings beginning 
--  with a period "." have the period doubled).
--  The list returned includes a final "."
-- TODO original implementation has some magic I don't understand, review that
local text = args[1]
return pl.List(text):map(function (line)
    if string.sub(line, 1, 1) == "." then
        return "." .. line
    else
        return line
    end
end):append(".")
