//! Host features for file path managment.
//!
//! There are cases where a plugin needs to store a complete file in it's state. For example, a sampler might want to store the recorded sample in a .wav file. However, chosing a valid path for this file is a delicate problem: First of all, different operating systems have different naming schemes for file paths. This means that the system has to be independent of naming schemes. Secondly, there might be multiple instances of the same plugin, other plugins, or even different hosts competing for a file path. Therefore, the system has to avoid collisions with other programs. Lastly, a path that was available when the state was saved might not be available when the state has to be restored. Therefore, the new absolute path to the file has to be retrievable.
//!
//! LV2 handles this problem by leaving it to the host implementors and specifying an interface for it. There are three distinct host features which are necessary to fulfill the tasks from above: [`MakePath`](struct.MakePath.html), which "makes" an absolute file path from a relative path, [`MapPath`](struct.MapPath), which maps an absolute path to/from an abstract string that can be stored as a property, and [`FreePath`](struct.FreePath.html), which frees the strings/paths created by the features above.
//!
//! Since all of these features need each other in order to be safe and sound, none of them can be used on their own. Instead, you use them to construct a [`PathManager`](struct.PathManager.html), which exposes all of their interfaces.
//!
//! The best way to understand this system is to have an example:
//!
//! ```
//! use lv2_core::prelude::*;
//! use lv2_state::*;
//! use lv2_state::path::*;
//! use lv2_atom::prelude::*;
//! use lv2_urid::*;
//! use urid::*;
//! use std::fs::File;
//! use std::path::Path;
//! use std::io::{Read, Write};
//!
//! // First, we need to write out some boilerplate code
//! // to define a proper plugin. There's no way around it. 😕
//!
//! /// The plugin we're outlining.
//! #[uri("urn:my-plugin")]
//! struct Sampler {
//!     // A vector of bytes, for simplicity's sake.
//!     // In a proper sampler, this would be a vector of floats.
//!     sample: Vec<u8>,
//!     urids: URIDs,
//! }
//!
//! /// The features we need.
//! #[derive(FeatureCollection)]
//! struct Features<'a> {
//!     makePath: MakePath<'a>,
//!     mapPath: MapPath<'a>,
//!     freePath: FreePath<'a>,
//!     uridMap: LV2Map<'a>,
//! }
//!
//! // A quick definition to identify the sample
//! // path in the state property store.
//! #[uri("urn:my-plugin:sample")]
//! struct Sample;
//!
//! /// Some URIDs we need.
//! #[derive(URIDCollection)]
//! struct URIDs {
//!     atom: AtomURIDCollection,
//!     sample: URID<Sample>,
//! }
//!
//! // Plugin implementation omitted...
//! # impl Plugin for Sampler {
//! #     type Ports = ();
//! #     type InitFeatures = Features<'static>;
//! #     type AudioFeatures = ();
//! #
//! #     fn new(_: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
//! #         Some(Self {
//! #             sample: Vec::new(),
//! #             urids: features.uridMap.populate_collection()?,
//! #         })
//! #     }
//! #
//! #     fn run(&mut self, _: &mut (), _: &mut ()) {}
//! # }
//!
//! impl State for Sampler {
//!     type StateFeatures = Features<'static>;
//!
//!     fn save(&self, mut store: StoreHandle, features: Features) -> Result<(), StateErr> {
//!         // Create a path manager, it manages all paths!
//!         let mut manager = PathManager::new(
//!             features.makePath,
//!             features.mapPath,
//!             features.freePath
//!         );
//!
//!         // Allocate a path to store the sample to.
//!         // The absolute path is the "real" path of the file we may write to
//!         // and the abstract path is the path we may store in a property.
//!         let (absolute_path, abstract_path) = manager
//!             .allocate_path(Path::new("sample.wav"))?;
//!
//!         // Store the sample. This isn't the correct way to save WAVs!
//!         let mut file = File::create(absolute_path).map_err(|_| StateErr::Unknown)?;
//!         file.write_all(self.sample.as_ref()).map_err(|_| StateErr::Unknown)?;
//!
//!         // Draft a new property to store the abstract path of the sample.
//!         {
//!             let mut path_writer = store.draft(self.urids.sample);
//!             let mut path_writer = path_writer
//!                 .init(self.urids.atom.string, ())
//!                 .map_err(|_| StateErr::Unknown)?;
//!             path_writer.append(&*abstract_path);
//!         }
//!
//!         // Commit everything!
//!         store.commit_all()
//!     }
//!
//!     fn restore(&mut self, store: RetrieveHandle, features: Features) -> Result<(), StateErr> {
//!         // Again, create a path a path manager.
//!         let mut manager = PathManager::new(
//!             features.makePath,
//!             features.mapPath,
//!             features.freePath
//!         );
//!
//!         // Retrieve the abstract path from the property store.
//!         let abstract_path = store
//!             .retrieve(self.urids.sample)?
//!             .read(self.urids.atom.string, ())
//!             .map_err(|_| StateErr::Unknown)?;
//!
//!         // Get the absolute path to the referenced file.
//!         let absolute_path = manager
//!             .deabstract_path(abstract_path)?;
//!
//!         // Open the file.
//!         let mut file = File::open(absolute_path)
//!             .map_err(|_| StateErr::Unknown)?;
//!
//!         // Write it to the sample.
//!         self.sample.clear();
//!         file.read_to_end(&mut self.sample)
//!             .map(|_| ())
//!             .map_err(|_| StateErr::Unknown)
//!     }
//! }
//! ```
use crate::StateErr;
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
    fn relative_to_absolute_path(&mut self, relative_path: &Path) -> Result<&'a Path, StateErr> {
        let relative_path: Vec<c_char> = relative_path
            .to_str()
            .ok_or(StateErr::PathNotUTF8)?
            .bytes()
            .chain(once(0))
            .map(|b| b as c_char)
            .collect();

        let absolute_path = unsafe { (self.function)(self.handle, relative_path.as_ptr()) };

        if absolute_path.is_null() {
            return Err(StateErr::HostError);
        }

        unsafe { CStr::from_ptr(absolute_path) }
            .to_str()
            .map(Path::new)
            .map_err(|_| StateErr::HostError)
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
    fn absolute_to_abstract_path(&mut self, path: &Path) -> Result<&'a str, StateErr> {
        let path: Vec<c_char> = path
            .to_str()
            .ok_or(StateErr::PathNotUTF8)?
            .bytes()
            .chain(once(0))
            .map(|b| b as c_char)
            .collect();

        let path = unsafe { (self.abstract_path)(self.handle, path.as_ptr()) };

        if path.is_null() {
            return Err(StateErr::HostError);
        }

        unsafe { CStr::from_ptr(path) }
            .to_str()
            .map_err(|_| StateErr::HostError)
    }

    fn abstract_to_absolute_path(&mut self, path: &str) -> Result<&'a Path, StateErr> {
        let path: Vec<c_char> = path.bytes().chain(once(0)).map(|b| b as c_char).collect();

        let path = unsafe { (self.absolute_path)(self.handle, path.as_ptr()) };

        if path.is_null() {
            return Err(StateErr::HostError);
        }

        unsafe { CStr::from_ptr(path) }
            .to_str()
            .map(Path::new)
            .map_err(|_| StateErr::HostError)
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
    ) -> Result<(ManagedPath<'a>, ManagedStr<'a>), StateErr> {
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

    pub fn deabstract_path(&mut self, path: &str) -> Result<ManagedPath<'a>, StateErr> {
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
