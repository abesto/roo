ObjectProxy = {}

ObjectProxy.__index = function(t, k)
    local opv = rawget(ObjectProxy, k)
    if opv ~= nil then
        return opv
    end
    return db:get_property(t.uuid, k)
end

ObjectProxy.__newindex = function(t, k, v)
    return db:set_property(t.uuid, k, v)
end

function ObjectProxy:new(uuid)
    local p = {uuid = uuid}
    setmetatable(p, self)
    return p
end

function ObjectProxy:move(where)
    db:move(self.uuid, where.uuid)
end
