@0xc0cbba4657b12ab7;
using Common = import "common.capnp";

struct EntityInfo @0xea20abdb567ef1f2 {
  location @0 :Common.Point;  
  appearance @0 :Text;
}

struct Update @0xeba1194f653a39f1 {
  union {
    entity_exists @0 :EntityInfo;
  }
}
