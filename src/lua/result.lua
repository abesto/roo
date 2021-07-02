-- Rough Result<V, E> implementation a 'la Rust (just another take on the Either monad)
local nilvalue = {}

pl.class.Result()

Result:catch(function(self, name)
    error("No '" .. name .. "' field on '" .. self._name .. "'")
end)

function Result:_init(value)
    if value == nil then
        self.value = nilvalue
    else
        self.value = value
    end
    self._checked = false
    self._created_at = debug.traceback(nil, 4)
end

function Result:_getvalue()
    if self.value == nilvalue then
        return nil
    else
        return self.value
    end
end

function Result:__tostring()
    return "%s(%s)" % {self._name, self.value}
end

function Result:__gc()
    if not self._checked then
        local msg = "Value of Result was never checked. Created at: %s" % {self._created_at}
        local r = player:notify(msg)
        if not r:is_ok() then
            print("Result:__gc: player:notify failed. Message: %s. Failure: %s" % {msg, r:err()})
        end
    end
end

function Result:is_ok()
    assert_class_of(0, self, Result)
    self._checked = true
    return self:is_a(Ok)
end

function Result:is_err()
    assert_class_of(0, self, Result)
    self._checked = true
    return self:is_a(Err)
end

function Result:err()
    assert_class_of(0, self, Err, ":err() called on an Ok: %s" % {self.value})
    self._checked = true
    return self:_getvalue()
end

function Result:unwrap()
    assert_class_of(0, self, Ok, ":unwrap() called on an Err: %s" % {self.value})
    self._checked = true
    return self:_getvalue()
end

pl.class.Ok(Result)
pl.class.Err(Result)

function Ok:land(other)
    assert_class_of(0, self, Result)
    assert_class_of(1, other, Result)
    other._checked = true
    return other
end

function Err:land(other)
    assert_class_of(0, self, Result)
    assert_class_of(1, other, Result)
    pl.utils.assert_arg(1, other, Result)
    self._checked = true
    return self
end

function Ok:and_then(f)
    assert_class_of(0, self, Result)
    local f = pl.utils.function_arg(1, f)
    local res = f(self.value)
    if not Result:class_of(res) then
        return pl.utils.raise("Return value of function passed to Result:and_then must be a Result, found: %s" %
                                  {pl.types.type(res)})
    end
    self._checked = true
    return res
end

function Ok:and_then_method(obj, f, ...)
    assert_class_of(0, self, Ok)
    self._checked = true
    local f = pl.utils.function_arg(1, obj[f])
    local args = List {self.value}:extend{...}
    local res = f(obj, unpack(args))
    if not Result:class_of(res) then
        return pl.utils.raise("Return value of function passed to Result:and_then must be a Result, found: %s" %
                                  {pl.types.type(res)})
    end
    return res
end

function Err:and_then(f)
    return self
end

function Ok:map(f)
    assert_class_of(0, self, Ok)
    pl.utils.function_arg(1, f)
    self._checked = true
    return Ok(f(self.value))
end

function Err:map(f)
    assert_class_of(0, self, Err)
    pl.utils.function_arg(1, f)
    return self
end

function Ok:unwrap_or(default)
    assert_class_of(0, self, Ok)
    return self:unwrap()
end

function Err:unwrap_or(default)
    assert_class_of(0, self, Err)
    self._checked = true
    return default
end

function Ok:unwrap_unsafe()
    assert_class_of(0, self, Ok)
    self._checked = true
    return self.value
end

function Err:unwrap_unsafe()
    assert_class_of(0, self, Err)
    self._checked = true
    return self.value
end

function Ok:zip(other)
    assert_class_of(0, self, Ok)
    assert_class_of(1, other, Result)
    if other:is_ok() then
        return ResultZip(List {self:unwrap(), other:unwrap()})
    end
    self._checked = true
    return other
end

function Err:zip(other)
    assert_class_of(0, self, Err)
    assert_class_of(1, other, Result)
    other._checked = true
    return self
end

function Result.zip(...)
    local args = List {...}
    local res = args[1]
    assert_class_of(1, res, Result)
    for i, arg in ipairs(args:slice(2)) do
        assert_class_of(i + 1, arg, Result)
        res = res:zip(arg)
    end
    return res
end

function Ok:map_method_unpacked(obj, f, ...)
    assert_class_of(0, self, Ok)
    local f = pl.utils.function_arg(1, obj[f])
    local args = List(self.value):extend{...}
    self._checked = true
    return Ok(f(obj, unpack(args)))
end

function Err:map_method_unpacked(obj, f, ...)
    assert_class_of(0, self, Err)
    local f = pl.utils.function_arg(1, obj[f])
    return self
end

function Ok:and_then_method_unpacked(obj, f, ...)
    assert_class_of(0, self, Ok)
    self._checked = true
    local f = pl.utils.function_arg(1, obj[f])
    local args = List(self.value):extend{...}
    local res = f(obj, unpack(args))
    if not Result:class_of(res) then
        return pl.utils.raise("Return value of function passed to Result:and_then must be a Result, found: %s" %
                                  {pl.types.type(res)})
    end
    return res
end

function Err:and_then_method_unpacked(obj, f, ...)
    assert_class_of(0, self, Err)
    pl.utils.function_arg(1, obj[f])
    return self
end

function Ok:map_unpacked(f, ...)
    assert_class_of(0, self, Result)
    self._checked = true
    pl.utils.function_arg(1, f)
    local args = List(self.value):extend{...}
    return Ok(f(unpack(args)))
end

function Err:map_unpacked(f, ...)
    assert_class_of(0, self, Result)
    pl.utils.function_arg(1, f)
    return self
end

function Ok:map_method(obj, f, ...)
    assert_class_of(0, self, Result)
    self._checked = true
    local f = pl.utils.function_arg(1, obj[f])
    local args = List {self.value}:extend{...}
    return Ok(f(obj, unpack(args)))
end

function Err:map_method(obj, f, ...)
    assert_class_of(0, self, Result)
    local f = pl.utils.function_arg(1, obj[f])
    return self
end

pl.class.ResultZip(Ok)

function ResultZip:zip(other)
    assert_class_of(0, self, ResultZip)
    assert_class_of(1, other, Result)
    if other:is_err() then
        return other
    elseif ResultZip:class_of(other) then
        return ResultZip(self:unwrap():extend(other:unwrap()))
    else
        return ResultZip(self:unwrap():append(other:unwrap()))
    end
end
