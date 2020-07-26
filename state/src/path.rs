//! Miscellaneous host features for path handling.
//!
//! This module contains three important host features: `MakePath`, `MapPath`, and `FreePath`.
//!
//! [`MakePath`](struct.MakePath.html) extends a relative path to an abstract path contained in a unique namespace for the plugin and the plugin instance. However, files stored under this path are not guaranteed to persist after the plugin instance has been saved and restored.
//!
//! In order to save a file together with the plugin state, it's absolute path has to be mapped to an abstract one using [`MapPath`](struct.MapPath.html). This tells the host to store this file along with the state and provides something the plugin can store as a property. When the state is restored, the `MapPath` feature has to be used again to retrieve the absolute path to the restored file.
//!
//! [`FreePath`](struct.FreePath.html) is used to tell the host that a certain file or folder isn't used anymore and that it should be freed by the host.
use lv2_core::feature::Feature;
use lv2_core::prelude::*;
use lv2_sys as sys;
use std::ffi::*;
use std::iter::once;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::path::*;
use urid::*;

/// An error that may occur when handling paths.
#[derive(Debug)]
pub enum PathError {
    /// The path to convert is not relative.
    PathNotRelative,
    /// The path to convert is not absolute.
    PathNotAbsolute,
    /// The path to convert is not encoded in UTF-8.
    PathNotUTF8,
    /// The host does not comply to the specification.
    HostError,
}

/// A host feature to make absolute paths.
pub struct MakePath<'a> {
    handle: sys::LV2_State_Make_Path_Handle,
    function: unsafe extern "C" fn(sys::LV2_State_Make_Path_Handle, *const c_char) -> *mut c_char,
    lifetime: PhantomData<&'a mut c_void>,
}

unsafe impl<'a> UriBound for MakePath<'a> {
    const URI: &'static [u8] = sys::LV2_STATE__makePath;
}

unsafe impl<'a> Feature for MakePath<'a> {
    unsafe fn from_feature_ptr(feature: *const c_void, _: ThreadingClass) -> Option<Self> {
        (feature as *const sys::LV2_State_Make_Path)
            .as_ref()
            .and_then(|internal| {
                Some(Self {
                    handle: internal.handle,
                    function: internal.path?,
                    lifetime: PhantomData,
                })
            })
    }
}

impl<'a> MakePath<'a> {
    /// Convert a relative path to an absolute path.
    ///
    /// The returned absolute path will be within a namespace unique to the plugin instance and the relative path will be the suffix of the absolute path. The leading directory will already be prepared for the creation of files and directories under the the returned path.
    ///
    /// This method may fail if the relative path isn't relative, it it's not encoded in UTF-8, or if the host does not return a valid absolute path.
    ///
    /// # Persistance
    ///
    /// The returned path will not be valid across plugin instances, even across save/restore calls. To save files along with the plugin state, use the [`MapPath`](struct.MapPath.html) feature to map the absolute path to an abstract one that can be saved.
    ///
    /// If a path isn't needed anymore, you should free it with the [`FreePath`](struct.FreePath.html) feature.
    pub fn relative_to_absolute_path(
        &mut self,
        relative_path: &Path,
    ) -> Result<PathBuf, PathError> {
        if !relative_path.is_relative() {
            return Err(PathError::PathNotRelative);
        }

        let relative_path: Vec<c_char> = relative_path
            .to_str()
            .ok_or(PathError::PathNotUTF8)?
            .bytes()
            .chain(once(0))
            .map(|b| b as c_char)
            .collect();

        let absolute_path = unsafe { (self.function)(self.handle, relative_path.as_ptr()) };

        if absolute_path.is_null() {
            return Err(PathError::HostError);
        }

        unsafe { CStr::from_ptr(absolute_path) }
            .to_str()
            .map(PathBuf::from)
            .map_err(|_| PathError::HostError)
    }
}

/// A host feature to save and restore files.
pub struct MapPath<'a> {
    handle: sys::LV2_State_Map_Path_Handle,
    abstract_path: unsafe extern "C" fn(
        sys::LV2_State_Map_Path_Handle,
        absolute_path: *const c_char,
    ) -> *mut c_char,
    absolute_path: unsafe extern "C" fn(
        sys::LV2_State_Map_Path_Handle,
        abstract_path: *const c_char,
    ) -> *mut c_char,
    lifetime: PhantomData<&'a mut c_void>,
}

unsafe impl<'a> UriBound for MapPath<'a> {
    const URI: &'static [u8] = sys::LV2_STATE__mapPath;
}

unsafe impl<'a> Feature for MapPath<'a> {
    unsafe fn from_feature_ptr(feature: *const c_void, _: ThreadingClass) -> Option<Self> {
        (feature as *const sys::LV2_State_Map_Path)
            .as_ref()
            .and_then(|internal| {
                Some(Self {
                    handle: internal.handle,
                    abstract_path: internal.abstract_path?,
                    absolute_path: internal.absolute_path?,
                    lifetime: PhantomData,
                })
            })
    }
}

impl<'a> MapPath<'a> {
    /// Mark a file for storage and retrieve an abstract path to it.
    ///
    /// Calling this method in the context of [`save`](../trait.State.html#tymethod.save) tells the host that the file or directory with the given path should be stored along the state of the plugin. The host will then create an abstract path that can be saved as a string property and converted back to an absolute path when restoring the state.
    ///
    /// However, the returned abstract path may or may not a be a valid path and therefore is only returned as a string.
    ///
    /// The method may fail if the given path isn't absolute, if it's not encoded in UTF-8, or if the host does not return a valid string.
    pub fn absolute_to_abstract_path(&mut self, path: &Path) -> Result<String, PathError> {
        if !path.is_absolute() {
            return Err(PathError::PathNotAbsolute);
        }

        let path: Vec<c_char> = path
            .to_str()
            .ok_or(PathError::PathNotUTF8)?
            .bytes()
            .chain(once(0))
            .map(|b| b as c_char)
            .collect();

        let path = unsafe { (self.abstract_path)(self.handle, path.as_ptr()) };

        if path.is_null() {
            return Err(PathError::HostError);
        }

        unsafe { CStr::from_ptr(path) }
            .to_str()
            .map(|path| path.to_owned())
            .map_err(|_| PathError::HostError)
    }

    /// Retrieve the absolute path to a stored file.
    ///
    /// Calling this method in the context of [`restore`](../trait.State.html#tymethod.restore) will retrieve the absolute path to a file that was stored using the [`absolute_to_abstract_path`](#method.absolute_to_abstract_path) method. All guarantees of [`MakePath::relative_to_absolute_path`](struct.MakePath.html#method.relative_to_absolute_path) apply here too.
    ///
    /// The method only fails if the host does not return a valid path.
    pub fn abstract_to_absolute_path(&mut self, path: &str) -> Result<PathBuf, PathError> {
        let path: Vec<c_char> = path.bytes().chain(once(0)).map(|b| b as c_char).collect();

        let path = unsafe { (self.absolute_path)(self.handle, path.as_ptr()) };

        if path.is_null() {
            return Err(PathError::HostError);
        }

        unsafe { CStr::from_ptr(path) }
            .to_str()
            .map(PathBuf::from)
            .map_err(|_| PathError::HostError)
    }
}

/// A host feature to a previously allocated path.
pub struct FreePath<'a> {
    handle: sys::LV2_State_Free_Path_Handle,
    free_path: unsafe extern "C" fn(sys::LV2_State_Free_Path_Handle, *mut c_char),
    lifetime: PhantomData<&'a mut c_void>,
}

unsafe impl<'a> UriBound for FreePath<'a> {
    const URI: &'static [u8] = sys::LV2_STATE__freePath;
}

unsafe impl<'a> Feature for FreePath<'a> {
    unsafe fn from_feature_ptr(feature: *const c_void, _: ThreadingClass) -> Option<Self> {
        (feature as *const sys::LV2_State_Free_Path)
            .as_ref()
            .and_then(|internal| {
                Some(Self {
                    handle: internal.handle,
                    free_path: internal.free_path?,
                    lifetime: PhantomData,
                })
            })
    }
}

impl<'a> FreePath<'a> {
    /// Tell the host that the files under the absolute path aren't used anymore.
    ///
    /// This method may be called with paths returned by [`MakePath::relative_to_absolute_path`](struct.MakePath.html#method.relative_to_absolute_path) and [`MapPath::abstract_to_absolute_path`](struct.MapPath.html#method.abstract_to_absolute_path)
    pub fn free_absolute_path(&self, path: &Path) {
        let mut path: Vec<c_char> = path
            .to_str()
            .unwrap()
            .bytes()
            .chain(std::iter::once(0))
            .map(|c| c as c_char)
            .collect();
        unsafe { (self.free_path)(self.handle, path.as_mut_ptr()) }
    }
}
