-- TODO rewrite with pl.class
-- TODO make internals safer by explicitly annotating properties, maybe bringing in PropertyValue as userdata
ObjectProxy = {
    compiled_verbs = {}
}

function ObjectProxy.__index(t, k)
    -- First check if this is a normal field on ObjectProxy
    local opv = rawget(ObjectProxy, k)
    if opv ~= nil then
        return opv
    end

    -- Read the value from the DB
    local v = db:get_property(t.uuid, k)

    -- Wrap verbs so that the DB can do arg matching at invocation time
    if db:has_verb_with_name(t.uuid, k) then
        return VerbProxy:new(t, k)
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

function ObjectProxy:resolve_verb(name, arity)
    return db:resolve_verb(self.uuid, name, arity)
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

VerbProxy = {}

function VerbProxy.__call(p, args)
    if args == nil then
        args = {}
    end
    return p.this:call_verb(p.verb, args)
end

function VerbProxy:__tostring()
    return "VerbProxy(" .. self.this.uuid .. ":" .. self.verb .. ")"
end

function VerbProxy:new(this, name)
    local p = {
        this = this,
        verb = name
    }
    setmetatable(p, self)
    return p
end
-- end of VerbProxy

ListProxy = {}

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
    if type(v) == "table" and v.uuid == nil then
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

-- TODO break out into separate "global Lua functions" module

function to_uuid(what)
    if type(what) == "table" then
        if what.uuid ~= nil then
            return what.uuid
        else
            return map(what, to_uuid)
        end
    else
        return what
    end
end

function inflate_uuid(x)
    if type(x) == "string" then
        local status, result = pcall(function()
            return db[x]
        end)
        if status and result ~= nil then
            return result
        end
    elseif type(x) == "table" then
        return map(x, inflate_uuid)
    end
    return x
end

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

function map(t, f)
    local r = {}
    for i, x in ipairs(t) do
        table.insert(r, i, f(x))
    end
    return r
end

function tostr(args)
    local strings = map(args, tostring)
    return table.concat(strings, "")
end

function keyset(t)
    local keyset = {}
    for k, v in pairs(t) do
        table.insert(keyset, k)
    end
    return keyset
end

system = db[system_uuid]
