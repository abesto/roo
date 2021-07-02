for i, target in ipairs(this.contents:without(player)) do
    pcall(target.tell, target, unpack(args))
end
