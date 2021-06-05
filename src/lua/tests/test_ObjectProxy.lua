require("lua.ObjectProxy")
local luaunit = require("luaunit")

TestObjectProxy = {}
function TestObjectProxy:setUp()
    local nextId = 0
    -- Dummy DB implementation for testing
    db = {}

    function db:create()
        local id = nextId
        nextId = nextId + 1
        db[id] = {}
        return ObjectProxy:new(id)
    end

    function db:set_property(uuid, k, v)
        print("DB SET")
        db[uuid][k] = v
    end

    function db:get_property(uuid, k)
        return db[uuid][k]
    end

    function db:move(what, where)
        db.lastMoveCall = {what, where}
    end
end

function TestObjectProxy:testGetSetProperty()
    local o = db:create()
    o.x = "foo"
    luaunit.assertEquals(rawget(o, "x"), nil)
    luaunit.assertEquals(o.x, "foo")

    local o2 = ObjectProxy:new(o.uuid)
    luaunit.assertEquals(o2.x, o.x)
end

function TestObjectProxy:testMove()
    local what = db:create()
    local where = db:create()
    what:move(where)
    luaunit.assertEquals(db.lastMoveCall, {what.uuid, where.uuid})
end

os.exit( luaunit.LuaUnit.run() )
