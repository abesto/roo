for i, target in ipairs(this.contents) do
    pcall(target.tell, target, unpack(args))
end
