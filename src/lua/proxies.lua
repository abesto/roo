--- Replaces ObjectProxy instances with their UUID in an arbitrarily nested list-like table
--- Returns a new table.
local function deflate_uuids(what)
    if is_type(what, ObjectProxy) then
        return what.__uuid
    elseif is_indexable(what) then
        return imap(deflate_uuids, what)
    else
        return what
    end
end

--- Replaces UUID strings with corresponding ObjectProxy instances in an arbitrarily nested list-like table.
--- Returns a new table.
local function inflate_uuids(x)
    if is_type(x, "string") then
        return toobj(x):unwrap_or(x)
    elseif is_indexable(x) then
        return imap(inflate_uuids, x)
    end
    return x
end

ObjectProxy = {
    _name = "ObjectProxy"
}
ObjectProxy.class_of = bind(is_type, _1, ObjectProxy)
assert_object = bind(assert_class_of, _1, _2, ObjectProxy, 4)

function ObjectProxy.__index(t, k)
    -- First check if this is a normal field on ObjectProxy
    local opv = rawget(ObjectProxy, k)
    if opv ~= nil then
        return opv
    end

    -- If we're asked for our UUID, just return it
    if k == "uuid" then
        return t.__uuid
    end

    -- If it's a verb, return it
    local verb = t:resolve_verb(k)
    if verb:is_ok() then
        return verb:unwrap()
    end

    -- Read the value from the DB
    local v = db:get_property(t.__uuid, k):unwrap()
    if k == "children" or k == "contents" then
        return List(inflate_uuids(v))
    end

    -- Spawn list wrapper so that we can do syntactically nice updates into
    -- nested lists
    if is_indexable(v) then
        return ListProxy:new(t.__uuid, k, {}, v)
    end

    -- Unpack UUIDs into ObjectProxies, unless we're actually trying to
    -- read the UUID
    if k ~= "uuid" and is_uuid(v) then
        v = toobj(v):unwrap()
    end

    return v
end

function ObjectProxy.__newindex(t, k, v)
    return db:set_property(t.__uuid, k, deflate_uuids(v)):unwrap()
end

function ObjectProxy.__eq(a, b)
    return Result.zip(touuid(a), touuid(b)):map_unpacked(pl.operator.eq):unwrap_or(false)
end

function ObjectProxy:__tostring()
    return "ObjectProxy(" .. self.__uuid .. ")"
end

function ObjectProxy:new(uuid)
    local p = {
        __uuid = uuid
    }
    setmetatable(p, self)
    return p
end

-- Expose object-related functions as methods on ObjectProxy for convenience
ObjectProxy.move = move
ObjectProxy.notify = notify
ObjectProxy.chparent = chparent
ObjectProxy.add_verb = add_verb
ObjectProxy.set_verb_code = set_verb_code
ObjectProxy.verb_code = verb_code
ObjectProxy.verb_args = verb_args
ObjectProxy.is_player = is_player
ObjectProxy.set_player_flag = set_player_flag
ObjectProxy.recycle = recycle

-- Roo-specific extensions

function ObjectProxy:resolve_verb(name)
    return db:resolve_verb(self.__uuid, name)
end

function ObjectProxy:call_verb(verb, args)
    local f = self:resolve_verb(verb)
    return f(self, args)
end

-- end of ObjectProxy

ListProxy = {
    _name = "ListProxy"
}
ListProxy.class_of = bind(is_type, _1, ListProxy)

function ListProxy.path_and(t, n)
    local new_path = {table.unpack(t._path)}
    table.insert(new_path, n)
    return new_path
end

function ListProxy.__index(t, k)
    local v = t._inner[k]

    local obj = toobj(v)
    if obj:is_ok() then
        v = obj:unwrap()
    end

    if is_indexable(v) and not is_type(v, ObjectProxy) then
        return ListProxy:new(t._uuid, t._prop, ListProxy.path_and(t, k - 1), v)
    end
    return v
end

function ListProxy.__newindex(t, k, v)
    return db:set_into_list(t._uuid, t._prop, ListProxy.path_and(t, k - 1), deflate_uuids(v)):unwrap()
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
-- To avoid code being confused by plyear not being set until 
-- the server injects the real player value
player = S.nothing