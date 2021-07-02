-- :my_match_object(string [,location])
return S.string_utils:match_object(unpack(
    pl.List(args):append(this.location):slice(1, 2):append(this)
))
