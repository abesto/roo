ObjectProxy = {
    compiled_verbs = {}
}

ObjectProxy.__index = function(t, k)
    local opv = rawget(ObjectProxy, k)
    if opv ~= nil then
        return opv
    end

    local v = db:get_property(t.uuid, k)
    if type(v) == "function" then
        this = t
        if this.location ~= nil then
            location = db[this.location]
        else
            location = nil
        end
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

function ObjectProxy:add_verb(signature)
    db:add_verb(self.uuid, signature)
end

function ObjectProxy:set_verb_code(name, code)
    db:set_verb_code(self.uuid, name, code)
end

function ObjectProxy:notify(msg)
    notify(self.uuid, msg)
end

-- TODO break out into separate "global Lua functions" module
function to_uuid(what)
    if type(what) == "table" then
        return what.uuid
    else
        return what
    end
end
