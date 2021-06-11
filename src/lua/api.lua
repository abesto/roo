-- Penlight "standard" library
pl = require 'pl.import_into'()

-- Selective imports from pl into the global namespace, these are part of the ROO API
map = pl.tablex.map
imap = pl.tablex.imap

--- Equivalents for the MOO built-in functions

function setremove(haystack, needle)
    -- Return haystack (a table) without needle in it
    local retval = {}
    for i, candidate in ipairs(haystack) do
        if candidate ~= needle then
            table.insert(retval, candidate)
        end
    end
    return retval
end

function tostr(args)
    local strings = imap(tostring, args)
    return table.concat(strings, "")
end

--- Minimal check for the below proxy objects

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

-- TODO make internals safer by explicitly annotating properties, maybe bringing in PropertyValue as userdata
--      .contents is especially problematic
-- TODO use pl.List instead of raw tables for list-like properties
-- TODO use pl.Set for .contents and .children
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