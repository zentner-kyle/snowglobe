@0x92229da7fa2256a2;
using Common = import "common.capnp";

struct Command @0xc1ac26dba21792f8 {
  union {
    move @0 :Common.Point;
  }
}
