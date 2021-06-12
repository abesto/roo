-- Penlight "standard" library
pl = require 'pl.import_into'()
require'pl.text'.format_operator() -- text templates
pl.utils.import 'pl.func' -- argument placeholders (ie. :map(_1.name))

--- Selective imports from pl into the global namespace, these are part of the ROO API
map = pl.tablex.map
imap = pl.tablex.imap

-- Extensions to pl for commonly used patterns
RooList = pl.class(pl.List)

function RooList:without(item)
    local l = self:clone()
    l:remove_value(item)
    return l
end
-- EOF pl extensions

--- Low-level helpers

local UuidChecker = pl.class()

function UuidChecker:check(s)
    -- return string.find(s, "[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[4][0-9A-Fa-f]{3}-[89ABab][0-9A-Fa-f]{3}-[0-9A-Fa-f]{12}") == 1
    self.i = 1
    self.s = s
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
    if type(s) ~= "string" then
        return false
    end
    if #s ~= 36 then
        return false
    end
    return UuidChecker_singleton:check(s)
end

--- Replaces ObjectProxy instances with their UUID in an arbitrarily nested list-like table
--- Returns a new table.
local function to_uuid(what)
    if type(what) == "table" then
        if ObjectProxy:class_of(what) then
            return what.uuid
        else
            return imap(to_uuid, what)
        end
    else
        return what
    end
end

--- Replaces UUID strings with corresponding ObjectProxy instances in an arbitrarily nested list-like table.
--- Returns a new table.
local function inflate_uuid(x)
    if type(x) == "string" then
        local status, result = pcall(function()
            return db[x]
        end)
        if status and result ~= nil then
            return result
        end
    elseif type(x) == "table" and not ObjectProxy:class_of(x) then
        return imap(inflate_uuid, x)
    end
    return x
end

local function class_of(klass, obj)
    if type(obj) ~= "table" then
        return false
    end
    return getmetatable(obj) == klass
end

-- Generic helper functions
function listwith(t, ...)
    local new = pl.tablex.copy(t)
    pl.tablex.insertvalues(new, {...})
    return new
end

function listcontains(t, x)
    return pl.tablex.find(t, x) ~= nil
end
-- EOF Generic helper functions

--- Equivalents for the MOO library
Moo = {}

-- Moo Errors
Moo.Error = pl.class()
function Moo.Error:_init(name)
    self.name = name
end
function Moo.Error:__tostring()
    return self.name
end

local function make_error(name)
    Moo[name] = Moo.Error(name)
end
imap(make_error, {"E_VERBNF", "E_TYPE"})
-- EOF Moo Errors

-- Moo functions
function Moo.tostr(...)
    local strings = imap(function(x)
        if type(x) == "string" then
            return x
        elseif ObjectProxy:class_of(x) or ListProxy:class_of(x) or type(x) ~= "table" then
            return tostring(x)
        else
            return pl.pretty.write(x)
        end
    end, ...)
    return table.concat(strings, "")
end

function Moo.listdelete(t, i)
    local new = pl.tablex.copy(t)
    table.remove(new, i)
    return new
end

function Moo.toliteral(x)
    -- Not perfect, but close enough probably?
    return pl.pretty.write(x)
end

function Moo.toobj(x)
    if type(x) ~= "string" or not is_uuid(x) then
        return Moo.E_TYPE
    end
    return ObjectProxy:new(x)
end

function Moo.set_task_perms(o)
    -- TODO impl
    -- Properly scoped implementation may require deeper integration with tokio
end

function Moo.verb_code(object, desc, fully_parent, indent)
    -- TODO full impl
    return db:verb_code(to_uuid(object), desc)
end

function Moo.notify(object, msg)
    object:notify(msg)
end

function Moo.valid(object)
    -- TODO impl
    local uuid = to_uuid(object)
    if not is_uuid(uuid) then
        return false
    end
    return db:valid(uuid)
end

function Moo.verbs(object)
    local uuid = to_uuid(object)
    return db:verbs(uuid)
end

function Moo.verb_info(object, desc)
    return db:verb_info(to_uuid(object), desc)
end
--- EOF Moo

ObjectProxy = {}
ObjectProxy.class_of = class_of

function ObjectProxy.__index(t, k)
    -- First check if this is a normal field on ObjectProxy
    local opv = rawget(ObjectProxy, k)
    if opv ~= nil then
        return opv
    end

    -- If it's a verb, return it
    local verb = t:resolve_verb(k)
    if verb ~= nil then
        return verb
    end

    -- Read the value from the DB
    local v = db:get_property(t.uuid, k)
    if k == "children" or k == "contents" then
        return RooList(inflate_uuid(v))
    end

    -- Spawn list wrapper so that we can do syntactically nice updates into
    -- nested lists
    if type(v) == "table" then
        return ListProxy:new(t.uuid, k, {}, v)
    end

    -- Unpack UUIDs into ObjectProxies, unless we're actually trying to
    -- read the UUID
    if k ~= "uuid" and type(v) == "string" then
        v = inflate_uuid(v)
    end

    return v
end

function ObjectProxy.__newindex(t, k, v)
    return db:set_property(t.uuid, k, to_uuid(v))
end

function ObjectProxy.__eq(a, b)
    return a.uuid == b.uuid
end

function ObjectProxy:__tostring()
    return "ObjectProxy(" .. self.uuid .. ")"
end

function ObjectProxy:new(uuid)
    local p = {
        uuid = uuid
    }
    setmetatable(p, self)
    return p
end

function ObjectProxy:move(where)
    db:move(self.uuid, to_uuid(where))
end

function ObjectProxy:chparent(new_parent)
    db:chparent(self.uuid, to_uuid(new_parent))
end

function ObjectProxy:add_verb(info, args)
    db:add_verb(self.uuid, info, args)
end

function ObjectProxy:set_verb_code(name, code)
    db:set_verb_code(self.uuid, name, code)
end

function ObjectProxy:verb_args(desc)
    -- TODO impl
    return {}
end

function ObjectProxy:resolve_verb(name)
    return db:resolve_verb(self.uuid, name)
end

function ObjectProxy:call_verb(verb, args)
    local f = self:resolve_verb(verb)
    return f(self, args)
end

function ObjectProxy:notify(msg)
    if notify ~= nil then
        notify(self.uuid, msg)
    end
end
-- end of ObjectProxy

ListProxy = {}
ListProxy.class_of = class_of

function ListProxy.path_and(t, n)
    local new_path = {table.unpack(t._path)}
    table.insert(new_path, n)
    return new_path
end

function ListProxy.__index(t, k)
    local v = t._inner[k]
    if type(v) == "string" then
        v = inflate_uuid(v)
    end
    if type(v) == "table" and not ObjectProxy:class_of(v) then
        return ListProxy:new(t._uuid, t._prop, ListProxy.path_and(t, k - 1), v)
    end
    return v
end

function ListProxy.__newindex(t, k, v)
    return db:set_into_list(t._uuid, t._prop, ListProxy.path_and(t, k - 1), to_uuid(v))
end

function ListProxy.__len(t)
    return #t._inner
end

function ListProxy:__tostring()
    return "ListProxy(" .. self._uuid .. "." .. self._prop .. "[" .. table.concat(self._path, "][") .. "])"
end

function ListProxy:new(uuid, prop, path, inner)
    local p = {
        _uuid = uuid,
        _prop = prop,
        _path = path,
        _inner = inner
    }
    setmetatable(p, self)
    return p
end

-- Equivalent for #0 on MOO
system = db[system_uuid]
S = system
M = Moo
