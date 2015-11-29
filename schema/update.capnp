@0xc0cbba4657b12ab7;
using Common = import "common.capnp";

struct EntityInfo @0xea20abdb567ef1f2 {
  identity @0 :UInt64;
  location @1 :Common.Point;  
  appearance @2 :Text;
}

struct Update @0xeba1194f653a39f1 {
  union {
    entityAlive @0 :EntityInfo;
    entityDead @1 :UInt64;
  }
}

struct ClientMessage @0x809ef84ea48a306b {
  updates @0 :List(Update);
}
