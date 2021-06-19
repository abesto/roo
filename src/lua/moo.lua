-- This file contains the equivalents of LambdaMoo builtins
-- (and the rare addition that lines up with them very closely)
Error = pl.class()

function Error:_init(name)
    self.name = name
end

function Error:__tostring()
    return self.name
end

function is_error(x)
    return Error:class_of(x)
end

local function make_error(name)
    rawset(_G, name, Error(name))
end
imap(make_error,
    {"E_NONE", "E_TYPE", "E_DIV", "E_PERM", "E_PROPNF", "E_VERBNF", "E_VARNF", "E_INVIND", "E_RECMOVE", "E_MAXREC",
     "E_RANGE", "E_ARGS", "E_NACC", "E_INVARG", "E_QUOTA", "E_FLOAT"})
-- EOF Moo Errors

-- Placeholders for values to be injected by the server
function _server_notify(...)
end

-- Moo functions
function tostr(...)
    local strings = imap(function(x)
        if is_type(x, "string") then
            return x
        elseif is_type(x, "table") and not (ObjectProxy:class_of(x) or ListProxy:class_of(x)) then
            return toliteral(x)
        else
            return tostring(x)
        end
    end, ...)
    return concat(strings, "")
end

function toliteral(x)
    -- Not perfect, but close enough probably?
    return pl.pretty.write(x)
end

function toobj(x)
    if is_type(x, ObjectProxy) then
        return Ok(x)
    elseif not is_type(x, "string") or not is_uuid(x) then
        return Err(E_TYPE)
    end
    return Ok(ObjectProxy:new(x))
end

-- Not really a Moo function, but it's roughly the inverse of toobj so /shrug
function touuid(what)
    if is_type(what, "string") then
        if not is_uuid(what) then
            return Err(E_TYPE)
        end
        return Ok(what)
    elseif is_type(what, ObjectProxy) then
        return Ok(what.uuid)
    else
        return Err(E_TYPE)
    end
end

function set_task_perms(o)
    -- TODO impl
    -- Properly scoped implementation may require deeper integration with tokio
end

---@return Result<List<String>, ?>
function verb_code(object, desc, fully_parent, indent)
    -- TODO full impl
    return touuid(object):map_method(db, 'verb_code', desc):map(List)
end

---@return Result<?, ?>
function set_verb_code(object, name, code)
    if is_type(code, "string") then
        code = pl.stringx.split(code, "\n")
    end
    return touuid(object):map_method(db, 'set_verb_code', name, code)
end

---@return Result<?, ?>
function notify(object, msg)
    if pl.types.is_callable(_server_notify) then
        return touuid(object):map(function(uuid)
            return _server_notify(uuid, msg)
        end)
    else
        return Ok()
    end
end

---@return Result<ObjectProxy, ?>
function create(parent, owner)
    local parent_res = touuid(parent)
    local owner_res = touuid(owner or S.nothing)

    return Result.zip(parent_res, owner_res):map_method_unpacked(db, 'create'):and_then(toobj):map(function(object)
        -- Call object:initialize() if it exists
        local initialize = object.initialize
        if initialize ~= nil then
            initialize(object)
        end

        return object
    end)
end

---@return boolean
function valid(object)
    -- TODO full impl
    return touuid(object):map_method(db, 'valid'):unwrap_or(false)
end

---@return Result<List<string>, ?>
function verbs(object)
    return touuid(object):map_method(db, 'verbs'):map(List)
end

---@return Result<table, ?>
function verb_info(object, desc)
    return touuid(object):map(function(uuid)
        return db:verb_info(uuid, desc)
    end)
end

---@return Result<?, ?>
function move(what, where)
    local what_uuid = touuid(what)
    local where_uuid = touuid(where)
    return Result.zip(what_uuid, where_uuid):map_method_unpacked(db, 'move')
end

---@return Result<?, ?>
function chparent(what, new_parent)
    local what_uuid = touuid(what)
    local new_parent_uuid = touuid(where)
    return Result.zip(what_uuid, new_parent_uuid):map_method_unpacked(db, 'chparent')
end

---@return Result<?, ?>
function add_verb(object, info, args)
    return touuid(object):map_method(db, 'add_verb', info, args)
end

---@return Result<?, ?>
function verb_args(object, desc)
    -- TODO impl
    return Ok({})
end

---@return Result<nil, ?>
function recycle(object)
    return touuid(object):map_method(db, 'recycle')
end

---@return Result<bool, ?>
function set_player_flag(object, val)
    return touuid(object):map_method(db, 'set_player_flag', val)
end

---@return Result<bool, ?>
function is_player(object)
    return touuid(object):map_method(db, 'is_player')
end

---@return List<ObjectProxy>
function players()
    return List(db:players()):map(function(uuid)
        return db[uuid]
    end)
end

function read()
    local line = _server_read()
    while line == nil do
        sleep(100)
        coroutine.yield()
        line = _server_read()
    end
    return line
end
