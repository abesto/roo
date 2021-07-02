local name = this:get_name()
if is_type(name, "string") and #name > 0 then
    return name
end
return this.uuid
