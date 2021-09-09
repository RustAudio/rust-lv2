use lv2_core::plugin::Foobaring;
use lv2_core::prelude::*;

struct Stateful;

// Uncomment this, and it works

impl Clone for Stateful {
    fn clone(&self) -> Self {
        //match_extensions!();
        todo!()
    }
}

impl Foobaring for Stateful {
    fn foo(&self) {
        match_extensions!();
    }
}

#[no_mangle]
fn foo2() -> *const SysFoo {
    todo!()
}
