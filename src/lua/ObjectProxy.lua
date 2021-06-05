ObjectProxy = {}
function ObjectProxy:new(uuid)
    p = {
        uuid = uuid
    }
    p.__index = self
    setmetatable(p, ObjectProxyMT)
    return p
end

ObjectProxyMT = {
    __newindex = function(t, k, v)
        return db:set_property(t.uuid, k, v)
    end,

    __index = function(t, k)
        return db:get_property(t.uuid, k)
    end,
}

