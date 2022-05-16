contract;

abi ImpurityTest {
    #[storage(read, write)]
    fn impure_func() -> bool;

// DISABLED until we can work out ABI methods.
//    #[storage(read)]
//    fn only_read() -> bool;
//
//    #[storage(write)]
//    fn only_write() -> bool;
//} {
//    #[storage(read)]
//    fn do_a_read() -> bool {
//        Self::only_read()
//    }
//
//    #[storage(write)]
//    fn do_a_write() -> bool {
//        only_write()
//    }
}

impl ImpurityTest for Contract {
    #[storage(read, write)]
    fn impure_func() -> bool {
        let a = can_also_read_and_write();
        true
    }

// DISABLED, see above.
//    #[storage(read)]
//    fn only_read() -> bool {
//        true
//    }
//
//    #[storage(write)]
//    fn only_write() -> bool {
//        true
//    }
}

#[storage(read)]
fn can_read_only() -> u64 {
    100
}

#[storage(read)]
fn can_also_read_only() -> u64 {
    can_read_only()
}

#[storage(write)]
fn can_write_only() -> u64 {
    101
}

#[storage(write)]
fn can_also_write_only() -> u64 {
    can_write_only()
}

#[storage(read, write)]
fn can_read_and_write() -> u64 {
    let a = can_also_read_only();
    let b = can_also_write_only();
    102
}

#[storage(read)]
#[storage(write)]
fn can_also_read_and_write() -> u64 {
    can_read_and_write()
}
