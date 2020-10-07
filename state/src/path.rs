//! Host features for file path managment.
//!
//! There are cases where a plugin needs to store a complete file in it's state. For example, a sampler might want to store the recorded sample in a .wav file. However, chosing a valid path for this file is a delicate problem: First of all, different operating systems have different naming schemes for file paths. This means that the system has to be independent of naming schemes. Secondly, there might be multiple instances of the same plugin, other plugins, or even different hosts competing for a file path. Therefore, the system has to avoid collisions with other programs. Lastly, a path that was available when the state was saved might not be available when the state has to be restored. Therefore, the new absolute path to the file has to be retrievable.
//!
//! LV2 handles this problem by leaving it to the host implementors and specifying an interface for it. There are three distinct host features which are necessary to fulfill the tasks from above: [`MakePath`](struct.MakePath.html), which "makes" an absolute file path from a relative path, [`MapPath`](struct.MapPath), which maps an absolute path to/from an abstract string that can be stored as a property, and [`FreePath`](struct.FreePath.html), which frees the strings/paths created by the features above.
//!
//! Since these features are strongly tied, they can only be used via a [`PathManager`](struct.PathManager.html). The best way to understand this system is to have an example:
//!
//! ```
//! use lv2_core::prelude::*;
//! use lv2_state::*;
//! use lv2_state::path::*;
//! use lv2_atom::prelude::*;
//! use urid::*;
//! use std::fs::File;
//! use std::path::Path;
//! use std::io::Write;
//!
//! #[derive(FeatureCollection)]
//! struct Features<'a> {
//!     makePath: MakePath<'a>,
//!     mapPath: MapPath<'a>,
//!     freePath: FreePath<'a>,
//! }
//!
//! #[uri("urn:my-plugin")]
//! struct Sampler {
//!     sample: Vec<u8>, // A vector of bytes, for simplicity's sake.
//! }
//!
//! // Plugin implementation omitted...
//! # impl Plugin for Sampler {
//! #     type Ports = ();
//! #     type InitFeatures = ();
//! #     type AudioFeatures = ();
//! #  
//! #     fn new(_: &PluginInfo, _: &mut ()) -> Option<Self> {
//! #         Some(Self {
//! #             sample: Vec::new(),
//! #         })
//! #     }
//! #  
//! #     fn run(&mut self, _: &mut (), _: &mut ()) {}
//! # }
//!
//! impl State for Sampler {
//!     type StateFeatures = Features<'static>;
//!
//!     fn save(&self, store: StoreHandle, features: Features) -> Result<(), StateErr> {
//!         let mut manager = PathManager::new(
//!             features.makePath,
//!             features.mapPath,
//!             features.freePath
//!         );
//!
//!         let (absolute_path, abstract_path) = manager
//!             .allocate_path(Path::new("sample.wav"))
//!             .map_err(|_| StateErr::Unknown)?;
//! 
//!         let mut file = File::create(abs_path).map_err(|_| StateErr::Unknown)?;
//!         file.write_all(self.sample.as_ref()).map_err(|_| StateErr::Unknown)?;
//! 
//!         let mut path_writer = store.draft::<String>(URID::new(42).unwrap());
//!         path_writer.init()
//!         
//!         Ok(())
//!     }
//!
//!     fn restore(&mut self, store: RetrieveHandle, features: Features) -> Result<(), StateErr> {
//!         Ok(())
//!     }
//! }
//! ```
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
use std::rc::Rc;
use std::sync::Mutex;
use urid::*;

/// An error that may occur when handling paths.
#[derive(Debug)]
pub enum PathError {
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
    fn relative_to_absolute_path(&mut self, relative_path: &Path) -> Result<&'a Path, PathError> {
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
            .map(Path::new)
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
    fn absolute_to_abstract_path(&mut self, path: &Path) -> Result<&'a str, PathError> {
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
            .map_err(|_| PathError::HostError)
    }

    fn abstract_to_absolute_path(&mut self, path: &str) -> Result<&'a Path, PathError> {
        let path: Vec<c_char> = path.bytes().chain(once(0)).map(|b| b as c_char).collect();

        let path = unsafe { (self.absolute_path)(self.handle, path.as_ptr()) };

        if path.is_null() {
            return Err(PathError::HostError);
        }

        unsafe { CStr::from_ptr(path) }
            .to_str()
            .map(Path::new)
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
    fn free_path(&self, path: &str) {
        unsafe { (self.free_path)(self.handle, path.as_ptr() as *mut c_char) }
    }
}

pub struct ManagedPath<'a> {
    path: &'a Path,
    free_path: Rc<Mutex<FreePath<'a>>>,
}

impl<'a> std::ops::Deref for ManagedPath<'a> {
    type Target = Path;

    fn deref(&self) -> &Path {
        self.path
    }
}

impl<'a> AsRef<Path> for ManagedPath<'a> {
    fn as_ref(&self) -> &Path {
        self.path
    }
}

impl<'a> Drop for ManagedPath<'a> {
    fn drop(&mut self) {
        self.free_path
            .lock()
            .unwrap()
            .free_path(self.path.to_str().unwrap())
    }
}

pub struct ManagedStr<'a> {
    str: &'a str,
    free_path: Rc<Mutex<FreePath<'a>>>,
}

impl<'a> std::ops::Deref for ManagedStr<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        self.str
    }
}

impl<'a> Drop for ManagedStr<'a> {
    fn drop(&mut self) {
        self.free_path.lock().unwrap().free_path(self.str)
    }
}

impl<'a> AsRef<str> for ManagedStr<'a> {
    fn as_ref(&self) -> &str {
        self.str
    }
}

pub struct PathManager<'a> {
    make: MakePath<'a>,
    map: MapPath<'a>,
    free: Rc<Mutex<FreePath<'a>>>,
}

impl<'a> PathManager<'a> {
    pub fn new(make: MakePath<'a>, map: MapPath<'a>, free: FreePath<'a>) -> Self {
        Self {
            make,
            map,
            free: Rc::new(Mutex::new(free)),
        }
    }

    pub fn allocate_path(
        &mut self,
        relative_path: &Path,
    ) -> Result<(ManagedPath<'a>, ManagedStr<'a>), PathError> {
        let absolute_path = self
            .make
            .relative_to_absolute_path(relative_path)
            .map(|path| ManagedPath {
                path,
                free_path: self.free.clone(),
            })?;

        let abstract_path = self
            .map
            .absolute_to_abstract_path(absolute_path.as_ref())
            .map(|str| ManagedStr {
                str,
                free_path: self.free.clone(),
            })?;

        Ok((absolute_path, abstract_path))
    }

    pub fn deabstract_path(&mut self, path: &str) -> Result<ManagedPath<'a>, PathError> {
        self.map
            .abstract_to_absolute_path(path)
            .map(|path| ManagedPath {
                path,
                free_path: self.free.clone(),
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::path::*;

    unsafe extern "C" fn make_path_impl(
        temp_dir: sys::LV2_State_Make_Path_Handle,
        relative_path: *const c_char,
    ) -> *mut c_char {
        let relative_path = match CStr::from_ptr(relative_path).to_str() {
            Ok(path) => path,
            _ => return std::ptr::null_mut(),
        };

        let temp_dir = temp_dir as *const mktemp::Temp;
        let mut absolute_path = (*temp_dir).as_path().to_path_buf();
        absolute_path.push(relative_path);

        CString::new(absolute_path.to_str().unwrap())
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    }

    unsafe extern "C" fn abstract_path_impl(
        temp_dir: sys::LV2_State_Map_Path_Handle,
        absolute_path: *const c_char,
    ) -> *mut c_char {
        let absolute_path = match CStr::from_ptr(absolute_path).to_str() {
            Ok(path) => Path::new(path),
            _ => return std::ptr::null_mut(),
        };

        let temp_dir = temp_dir as *const mktemp::Temp;
        let temp_dir = (*temp_dir).as_path();
        let abstract_path = absolute_path.strip_prefix(temp_dir).unwrap();

        CString::new(abstract_path.to_str().unwrap())
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    }

    unsafe extern "C" fn free_path_impl(
        free_counter: sys::LV2_State_Free_Path_Handle,
        path: *mut c_char,
    ) {
        *(free_counter as *mut u32).as_mut().unwrap() += 1;
        CString::from_raw(path);
    }

    #[test]
    fn test_path() {
        let temp_dir = mktemp::Temp::new_dir().unwrap();

        let make_path_feature = sys::LV2_State_Make_Path {
            handle: &temp_dir as *const _ as *mut c_void,
            path: Some(make_path_impl),
        };
        let make_path = unsafe {
            MakePath::from_feature_ptr(
                &make_path_feature as *const _ as *const c_void,
                ThreadingClass::Other,
            )
        }
        .unwrap();

        let map_path_feature = sys::LV2_State_Map_Path {
            handle: &temp_dir as *const _ as *mut c_void,
            abstract_path: Some(abstract_path_impl),
            absolute_path: Some(make_path_impl),
        };
        let map_path = unsafe {
            MapPath::from_feature_ptr(
                &map_path_feature as *const _ as *const c_void,
                ThreadingClass::Other,
            )
        }
        .unwrap();

        let mut free_counter: u32 = 0;
        let free_path_feature = sys::LV2_State_Free_Path {
            handle: &mut free_counter as *mut _ as *mut c_void,
            free_path: Some(free_path_impl),
        };
        let free_path = unsafe {
            FreePath::from_feature_ptr(
                &free_path_feature as *const _ as *const c_void,
                ThreadingClass::Other,
            )
        }
        .unwrap();

        let mut manager = PathManager::new(make_path, map_path, free_path);
        let relative_path = Path::new("sample.wav");
        let ref_absolute_path: PathBuf = [temp_dir.as_path(), relative_path].iter().collect();

        {
            let (absolute_path, abstract_path) = manager.allocate_path(relative_path).unwrap();
            assert_eq!(ref_absolute_path, &*absolute_path);
            assert_eq!(relative_path.to_str().unwrap(), &*abstract_path);

            let absolute_path = manager.deabstract_path(&abstract_path).unwrap();
            assert_eq!(ref_absolute_path, &*absolute_path);
        }

        assert_eq!(free_counter, 3);
    }
}
