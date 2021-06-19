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
end

function Result:_getvalue()
    if self.value == nilvalue then
        return nil
    else
        return self.value
    end
end

function Result:is_ok()
    return self:is_a(Ok)
end

function Result:is_err()
    return self:is_a(Err)
end

function Result:err()
    assert_class_of(0, self, Err, ":err() called on an Ok")
    return self:_getvalue()
end

function Result:unwrap()
    assert_class_of(0, self, Ok, ":unwrap() called on an Err")
    return self:_getvalue()
end

pl.class.Ok(Result)
pl.class.Err(Result)

function Ok:land(other)
    assert_class_of(0, self, Result)
    assert_class_of(1, other, Result)
    return other
end

function Err:land(other)
    assert_class_of(0, self, Result)
    assert_class_of(1, other, Result)
    pl.utils.assert_arg(1, other, Result)
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
    return res
end

function Err:and_then(f)
    return self
end

function Ok:map(f)
    assert_class_of(0, self, Ok)
    pl.utils.function_arg(1, f)
    return Ok(f(self.value))
end

function Err:map(f)
    assert_class_of(0, self, Err)
    pl.utils.function_arg(1, f)
    return self
end

function Ok:unwrap_or(default)
    assert_class_of(0, self, Ok)
    return self.value
end

function Err:unwrap_or(default)
    assert_class_of(0, self, Err)
    return default
end

function Ok:unwrap_unsafe()
    assert_class_of(0, self, Ok)
    return self.value
end

function Err:unwrap_unsafe()
    assert_class_of(0, self, Err)
    return self.value
end

function Ok:zip(other)
    assert_class_of(0, self, Ok)
    assert_class_of(1, other, Result)
    if other:is_ok() then
        return ResultZip(List {self:unwrap(), other:unwrap()})
    end
    return other
end

function Err:zip(other)
    assert_class_of(0, self, Result)
    assert_class_of(1, other, Result)
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
    assert_class_of(0, self, Result)
    local f = pl.utils.function_arg(1, obj[f])
    local args = List(self.value):extend{...}
    return Ok(f(obj, unpack(args)))
end

function Err:map_method_unpacked(obj, f, ...)
    assert_class_of(0, self, Result)
    local f = pl.utils.function_arg(1, obj[f])
    return self
end

function Ok:map_unpacked(f, ...)
    assert_class_of(0, self, Result)
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
