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

    -- Unpack UUIDs into ObjectProxies, unless we're actually trying to
    -- read the UUID
    if k ~= "uuid" then
        v = inflate_uuid(v)
    end

    return v
end

ObjectProxy.__newindex = function(t, k, v)
    return db:set_property(t.uuid, k, v)
end

ObjectProxy.__eq = function(a, b)
    return a.uuid == b.uuid
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

function VerbProxy:new(this, name)
    local p = {
        this = this,
        verb = name
    }
    setmetatable(p, self)
    return p
end
-- end of VerbProxy

-- TODO break out into separate "global Lua functions" module

function to_uuid(what)
    if type(what) == "table" then
        return what.uuid
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
    for i, x in pairs(t) do
        table.insert(r, i, f(x))
    end
    return r
end

function tostr(args)
    local strings = map(args, tostring)
    return table.concat(strings, "")
end

system = db[system_uuid]
