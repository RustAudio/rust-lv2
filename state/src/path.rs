use lv2_core::feature::Feature;
use lv2_core::prelude::*;
use lv2_sys as sys;
use std::ffi::*;
use std::fs::*;
use std::iter::once;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::path::*;
use std::rc::Rc;
use std::sync::Mutex;
use urid::*;

#[derive(Debug)]
pub enum HostManagedPathError {
    PathNotUTF8,
    HostError,
    OpenError(std::io::Error),
    FeatureNotPresent,
}

pub struct HostManagedPath<'a> {
    path: PathBuf,
    free_path: FreePath<'a>,
}

impl<'a> HostManagedPath<'a> {
    pub fn open_file<'b>(&'b self) -> Result<HostManagedFile<'b, 'a>, std::io::Error> {
        Ok(HostManagedFile {
            path: self,
            file: File::open(self.path.as_path())?,
        })
    }

    pub fn create_file<'b>(&'b self) -> Result<HostManagedFile<'b, 'a>, std::io::Error> {
        Ok(HostManagedFile {
            path: self,
            file: File::create(self.path.as_path())?,
        })
    }

    pub fn open_with_options<'b>(
        &'b self,
        options: &OpenOptions,
    ) -> Result<HostManagedFile<'b, 'a>, std::io::Error> {
        Ok(HostManagedFile {
            path: self,
            file: options.open(self.path.as_path())?,
        })
    }
}

impl<'a> Drop for HostManagedPath<'a> {
    fn drop(&mut self) {
        self.free_path.free_path(self.path.as_path());
    }
}

pub struct HostManagedFile<'a, 'b> {
    path: &'a HostManagedPath<'b>,
    file: File,
}

impl<'a, 'b> HostManagedFile<'a, 'b> {
    pub fn path(&self) -> &'a HostManagedPath<'b> {
        self.path
    }
}

impl<'a, 'b> std::ops::Deref for HostManagedFile<'a, 'b> {
    type Target = File;

    fn deref(&self) -> &File {
        &self.file
    }
}

impl<'a, 'b> std::ops::DerefMut for HostManagedFile<'a, 'b> {
    fn deref_mut(&mut self) -> &mut File {
        &mut self.file
    }
}

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
    fn make_path(&mut self, path: &Path) -> Result<PathBuf, HostManagedPathError> {
        let path: Vec<c_char> = path
            .to_str()
            .ok_or(HostManagedPathError::PathNotUTF8)?
            .bytes()
            .chain(once(0))
            .map(|b| b as c_char)
            .collect();

        let path = unsafe { (self.function)(self.handle, path.as_ptr()) };

        if path.is_null() {
            return Err(HostManagedPathError::HostError);
        }

        unsafe { CStr::from_ptr(path) }
            .to_str()
            .map(PathBuf::from)
            .map_err(|_| HostManagedPathError::HostError)
    }
}

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
    fn abstract_path(&mut self, path: &Path) -> Result<String, HostManagedPathError> {
        let path: Vec<c_char> = path
            .to_str()
            .ok_or(HostManagedPathError::PathNotUTF8)?
            .bytes()
            .chain(once(0))
            .map(|b| b as c_char)
            .collect();

        let path = unsafe { (self.abstract_path)(self.handle, path.as_ptr()) };

        if path.is_null() {
            return Err(HostManagedPathError::HostError);
        }

        unsafe { CStr::from_ptr(path) }
            .to_str()
            .map(|path| path.to_owned())
            .map_err(|_| HostManagedPathError::HostError)
    }

    fn absolute_path(&mut self, path: &str) -> Result<PathBuf, HostManagedPathError> {
        let path: Vec<c_char> = path.bytes().chain(once(0)).map(|b| b as c_char).collect();

        let path = unsafe { (self.absolute_path)(self.handle, path.as_ptr()) };

        if path.is_null() {
            return Err(HostManagedPathError::HostError);
        }

        unsafe { CStr::from_ptr(path) }
            .to_str()
            .map(PathBuf::from)
            .map_err(|_| HostManagedPathError::HostError)
    }
}

struct FreePathImpl<'a> {
    handle: sys::LV2_State_Free_Path_Handle,
    free_path: unsafe extern "C" fn(sys::LV2_State_Free_Path_Handle, *mut c_char),
    lifetime: PhantomData<&'a mut c_void>,
}

#[derive(Clone)]
pub struct FreePath<'a> {
    internal: Rc<Mutex<FreePathImpl<'a>>>,
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
                    internal: Rc::new(Mutex::new(FreePathImpl {
                        handle: internal.handle,
                        free_path: internal.free_path?,
                        lifetime: PhantomData,
                    })),
                })
            })
    }
}

impl<'a> FreePath<'a> {
    fn free_path(&self, path: &Path) {
        let internal = self.internal.lock().unwrap();
        let mut path: Vec<c_char> = path
            .to_str()
            .unwrap()
            .bytes()
            .chain(std::iter::once(0))
            .map(|c| c as c_char)
            .collect();
        unsafe { (internal.free_path)(internal.handle, path.as_mut_ptr()) }
    }
}

pub struct PathManager<'a> {
    make_path: MakePath<'a>,
    map_path: Option<MapPath<'a>>,
    free_path: FreePath<'a>,
}

impl<'a> PathManager<'a> {
    pub fn new(make_path: MakePath<'a>, free_path: FreePath<'a>) -> Self {
        Self {
            make_path,
            free_path,
            map_path: None,
        }
    }

    pub fn with_map(
        make_path: MakePath<'a>,
        free_path: FreePath<'a>,
        map_path: MapPath<'a>,
    ) -> Self {
        Self {
            make_path,
            free_path,
            map_path: Some(map_path),
        }
    }

    pub fn add_map_feature(&mut self, map_path: MapPath<'a>) {
        self.map_path = Some(map_path);
    }

    pub fn allocate_path<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<HostManagedPath<'a>, HostManagedPathError> {
        Ok(HostManagedPath {
            path: self.make_path.make_path(path.as_ref())?,
            free_path: self.free_path.clone(),
        })
    }

    pub fn map_path(&mut self, path: &HostManagedPath) -> Result<String, HostManagedPathError> {
        let feature = self
            .map_path
            .as_mut()
            .ok_or(HostManagedPathError::FeatureNotPresent)?;
        feature.abstract_path(path.path.as_ref())
    }

    pub fn unmap_path(
        &mut self,
        mapped_path: &str,
    ) -> Result<HostManagedPath<'a>, HostManagedPathError> {
        let feature = self
            .map_path
            .as_mut()
            .ok_or(HostManagedPathError::FeatureNotPresent)?;
        let path = feature.absolute_path(mapped_path)?;
        Ok(HostManagedPath {
            path,
            free_path: self.free_path.clone(),
        })
    }
}
