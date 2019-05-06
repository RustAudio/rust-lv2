use crate::feature::descriptor::FeatureDescriptor;
use crate::feature::Feature;
use std::marker::PhantomData;

pub struct FeatureList<'a> {
    inner: *const *const sys::LV2_Feature,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> FeatureList<'a> {
    pub unsafe fn from_raw(inner: *const *const sys::LV2_Feature) -> FeatureList<'a> {
        Self {
            inner: inner as _,
            _lifetime: PhantomData,
        }
    }

    pub fn find<F: Feature>(&self) -> Option<&'a F> {
        self.into_iter()
            .filter_map(FeatureDescriptor::into_feature_ref::<F>)
            .next()
    }
}

impl<'a, 'b> IntoIterator for &'b FeatureList<'a> {
    type Item = FeatureDescriptor<'a>;
    type IntoIter = FeatureIter<'a>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        FeatureIter {
            ptr: self.inner,
            _lifetime: PhantomData,
        }
    }
}

pub struct FeatureIter<'a> {
    ptr: *const *const sys::LV2_Feature,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Iterator for FeatureIter<'a> {
    type Item = FeatureDescriptor<'a>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.ptr.is_null() {
            return None;
        }
        if unsafe { *self.ptr }.is_null() {
            return None;
        }

        let feature = unsafe { FeatureDescriptor::from_raw(*self.ptr as _) };
        self.ptr = unsafe { self.ptr.offset(1) };
        Some(feature)
    }
}
