local label, upload = table.unpack(args)
if not upload then
    upload = "none"
end
local msg = "#$# edit name: %s upload: %s" % {label, upload}
player:tell(msg)
