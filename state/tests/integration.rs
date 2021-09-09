use lv2_core::plugin::Foobaring;
use lv2_core::prelude::*;

struct Stateful {}

impl Clone for Stateful {
    fn clone(&self) -> Self {
        // match_extensions!();
        todo!()
    }
}

impl Foobaring for Stateful {
    fn foo() {
        match_extensions!();
    }
}

lv2_descriptors! {
    Stateful
}

#[test]
pub fn foo() {}
