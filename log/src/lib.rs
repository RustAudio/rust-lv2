use lv2_core::feature::*;

pub struct LogHandler<'a,P> {
    internal:&'a lv2_sys::LV2_Log_Log,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
