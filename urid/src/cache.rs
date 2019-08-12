use crate::feature::*;
use crate::URID;
use core::UriBound;
use fnv::FnvHashMap;
use std::collections::{hash_map, HashMap};
use std::ffi::CStr;

pub struct CachedMap<'a> {
    map_feature: Map<'a>,
    cache: FnvHashMap<*const i8, URID>,
}

type Iter<'a> = std::iter::Map<
    hash_map::Iter<'a, *const i8, URID>,
    fn((&'a *const i8, &'a URID)) -> (&'static CStr, URID),
>;

impl<'a> CachedMap<'a> {
    pub fn from_feature(map_feature: Map<'a>) -> Self {
        Self {
            map_feature,
            cache: HashMap::default(),
        }
    }

    pub fn urid_of<T: UriBound>(&mut self) -> URID {
        let cache = &mut self.cache;
        let map_feature = &mut self.map_feature;
        let uri_address = T::URI.as_ptr() as *const i8;

        *(cache
            .entry(uri_address)
            .or_insert_with(|| map_feature.map(T::uri())))
    }

    pub fn try_urid_of<T: UriBound>(&self) -> Option<URID> {
        let uri_address = T::URI.as_ptr() as *const i8;
        self.cache.get(&uri_address).copied()
    }

    pub fn map_feature(&mut self) -> &mut Map<'a> {
        &mut self.map_feature
    }

    pub fn iter(&self) -> Iter {
        self.cache.iter().map(|(ptr, urid): _| {
            let ptr = *ptr;
            let uri: &'static CStr = unsafe { CStr::from_ptr(ptr) };
            (uri, *urid)
        })
    }
}

#[test]
fn test_cached_map() {
    use core::feature::IsLive;
    let mut test_bench = crate::test_bench::TestBench::new();
    let mut cached_map = CachedMap::from_feature(test_bench.make_map());

    // urid_of
    assert_eq!(0, cached_map.urid_of::<Map>());
    assert_eq!(1, cached_map.urid_of::<Unmap>());
    assert_eq!(0, cached_map.urid_of::<Map>());

    // try_urid_of
    assert_eq!(Some(0), cached_map.try_urid_of::<Map>());
    assert_eq!(Some(1), cached_map.try_urid_of::<Unmap>());
    assert_eq!(None, cached_map.try_urid_of::<IsLive>());

    // map_feature
    let feature = cached_map.map_feature();
    assert_eq!(0, feature.map(Map::uri()));
    assert_eq!(1, feature.map(Unmap::uri()));

    // iter
    let urids: Vec<(&CStr, URID)> = cached_map.iter().collect();
    assert_eq!(2, urids.len());
    assert!(urids.contains(&(Map::uri(), 0)));
    assert!(urids.contains(&(Unmap::uri(), 1)));
}
