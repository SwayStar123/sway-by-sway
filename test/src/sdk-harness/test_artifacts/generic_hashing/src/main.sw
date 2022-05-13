script;

use std::hash::sha256;
use std::assert::assert;

struct StructData {
    field_1: bool,
    field_2: u64,
    field_3: b256
}


fn main() -> bool {
    let my_struct = StructData {
        field_1: true,
        field_2: 42,
        field_3: 0x3333333333333333333333333333333333333333333333333333333333333333,
    };
    assert(sha256(42) == 0x73475cb40a568e8da8a045ced110137e159f890ac4da883b6b17dc651b3a8049);
    // assert(sha256(true) == 0xb5bea41b6c623f7c09f1bf24dcae58ebab3c0cdd90ad966bc43a45b44867e12b);
    // assert(sha256((11, false, 65)) == 0xbedf2634151635012c3e04d9f3144a073f6cc4f5aee613afd5e7950dae7a7cdc);
    // assert(sha256(my_struct) == 0x7ace2a893688ac38b10aec92bae192f25a7db7887563f6a8fbe4e7b7c6307f25);
    // assert(sha256("FUEL") == 0x0e2f3c1696028c617b97909d5e87a76d369e535a6a15f409bd876c119af6a382);

    true
}
