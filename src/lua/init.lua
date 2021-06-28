-- Penlight "standard" library
pl = require 'pl.import_into'()
require'pl.text'.format_operator() -- text templates
strict = require 'pl.strict'
strict.make_all_strict(_G)
pl.utils.import 'pl.func' -- argument placeholders (ie. :map(_1.name))
pl.utils.on_error("error")

-- Selective imports from Lua stdlib into global namespace
unpack = table.unpack
concat = table.concat

min = math.min
max = math.max

-- Selective imports from pl into the global namespace
map = pl.tablex.map
imap = pl.tablex.imap

List = pl.List

is_type = pl.types.is_type
is_indexable = pl.types.is_indexable

bind1 = pl.func.bind1
bind = pl.func.bind

assert_string = pl.utils.assert_string
assert_arg = pl.utils.assert_arg
function_arg = pl.utils.function_arg

-- Extensions to pl for commonly used patterns
function assert_class_of(n, x, C, msg)
    return pl.utils.assert_arg(n, x, 'table', function(o)
        return C:class_of(o)
    end, msg, 3)
end

function pl.List:without(item)
    local l = self:clone()
    l:remove_value(item)
    return l
end
--- Low-level helpers

local UuidChecker = pl.class()

function UuidChecker:check(s)
    -- return string.find(s, "[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[4][0-9A-Fa-f]{3}-[89ABab][0-9A-Fa-f]{3}-[0-9A-Fa-f]{12}") == 1
    self.i = 1
    self.s = pl.utils.assert_string(1, s)
    self.failed = false
    self:hexadec(8):dash():hexadec(4):dash():exactly("4"):hexadec(3):dash():matchone("[89ABab]"):hexadec(3):dash()
        :hexadec(12)
    return not self.failed
end

function UuidChecker:c()
    return string.sub(self.s, self.i, self.i)
end

function UuidChecker:hexadec(n)
    if self.failed then
        return self
    end
    for i = 1, n do
        if not self:matchone("[0-9A-Fa-f]") then
            self.failed = true
            return self
        end
    end
    return self
end

function UuidChecker:exactly(c)
    if self.failed then
        return self
    end
    if self:c() ~= c then
        self.failed = true
        return self
    end
    self.i = self.i + 1
    return self
end

function UuidChecker:dash()
    if self.failed then
        return self
    end
    return self:exactly("-")
end

function UuidChecker:matchone(cs)
    if self.failed then
        return self
    end
    if string.find(self:c(), cs) == nil then
        self.failed = true
        return self
    end
    self.i = self.i + 1
    return self
end

local UuidChecker_singleton = UuidChecker()

--- Checks whether a variable holds a string that looks like a valid UUIDv4
---@return boolean
function is_uuid(s)
    if not is_type(s, "string") then
        return false
    end
    if #s ~= 36 then
        return false
    end
    return UuidChecker_singleton:check(s)
end