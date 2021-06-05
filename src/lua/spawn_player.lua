me = db:create()
me.name = "a player"
db:move(me.uuid, void.uuid)
return me.uuid