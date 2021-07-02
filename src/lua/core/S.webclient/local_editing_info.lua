local object, vname, code = table.unpack(args)
local vargs
if is_type(vname, "table") then
  vargs = " %s %s %s" % {vname[2], S.code_utils:short_prep(vname[3]), vname[4]}
  vname = vname[1]
else
  vargs = ""
end
local name = "%s:%s" % {object.name, vname};
-- TODO swap to full @program invocation once we have proper dobj, prep, iobj support
-- local upload = "@program %s:%s %s" % {object.uuid, vname, vargs}
local upload = "@program %s:%s" % {object.uuid, vname}
return {name, code, upload};
