use lv2_core::feature::Feature;
use lv2_core::prelude::*;
use lv2_sys as sys;
use std::ffi::*;
use std::fs::*;
use std::iter::once;
use std::os::raw::c_char;
use std::path::*;
use std::rc::Rc;
use std::sync::Mutex;
use urid::*;

pub enum HostManagedPathError {
    PathNotUTF8,
    HostError,
    OpenError(std::io::Error),
}

pub struct HostManagedPath {
    path: PathBuf,
    free_path: Rc<Mutex<FreePath>>,
}

impl HostManagedPath {
    pub fn open(&self) -> Result<HostManagedFile, std::io::Error> {
        Ok(HostManagedFile {
            path: self,
            file: File::open(self.path.as_path())?,
        })
    }

    pub fn create(&self) -> Result<HostManagedFile, std::io::Error> {
        Ok(HostManagedFile {
            path: self,
            file: File::create(self.path.as_path())?,
        })
    }

    pub fn with_options(&self, options: &OpenOptions) -> Result<HostManagedFile, std::io::Error> {
        Ok(HostManagedFile {
            path: self,
            file: options.open(self.path.as_path())?,
        })
    }
}

impl<'a, 'b> Drop for HostManagedPath {
    fn drop(&mut self) {
        self.free_path
            .lock()
            .unwrap()
            .free_path(self.path.as_path());
    }
}

pub struct HostManagedFile<'a> {
    path: &'a HostManagedPath,
    file: File,
}

impl<'a> HostManagedFile<'a> {
    pub fn path(&self) -> &'a HostManagedPath {
        self.path
    }
}

impl<'a> std::ops::Deref for HostManagedFile<'a> {
    type Target = File;

    fn deref(&self) -> &File {
        &self.file
    }
}

impl<'a> std::ops::DerefMut for HostManagedFile<'a> {
    fn deref_mut(&mut self) -> &mut File {
        &mut self.file
    }
}

pub struct MakePath {
    handle: sys::LV2_State_Make_Path_Handle,
    function: unsafe extern "C" fn(sys::LV2_State_Make_Path_Handle, *const c_char) -> *mut c_char,
}

unsafe impl UriBound for MakePath {
    const URI: &'static [u8] = sys::LV2_STATE__makePath;
}

unsafe impl Feature for MakePath {
    unsafe fn from_feature_ptr(feature: *const c_void, _: ThreadingClass) -> Option<Self> {
        (feature as *const sys::LV2_State_Make_Path)
            .as_ref()
            .and_then(|internal| {
                Some(Self {
                    handle: internal.handle,
                    function: internal.path?,
                })
            })
    }
}

impl MakePath {
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

pub struct MapPath {
    handle: sys::LV2_State_Map_Path_Handle,
    abstract_path: unsafe extern "C" fn(
        sys::LV2_State_Map_Path_Handle,
        absolute_path: *const c_char,
    ) -> *mut c_char,
    absolute_path: unsafe extern "C" fn(
        sys::LV2_State_Map_Path_Handle,
        abstract_path: *const c_char,
    ) -> *mut c_char,
}

unsafe impl UriBound for MapPath {
    const URI: &'static [u8] = sys::LV2_STATE__mapPath;
}

unsafe impl Feature for MapPath {
    unsafe fn from_feature_ptr(feature: *const c_void, _: ThreadingClass) -> Option<Self> {
        (feature as *const sys::LV2_State_Map_Path)
            .as_ref()
            .and_then(|internal| {
                Some(Self {
                    handle: internal.handle,
                    abstract_path: internal.abstract_path?,
                    absolute_path: internal.absolute_path?,
                })
            })
    }
}

impl MapPath {
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

pub struct FreePath {
    handle: sys::LV2_State_Free_Path_Handle,
    function: unsafe extern "C" fn(sys::LV2_State_Free_Path_Handle, *mut c_char),
}

unsafe impl UriBound for FreePath {
    const URI: &'static [u8] = sys::LV2_STATE__freePath;
}

unsafe impl Feature for FreePath {
    unsafe fn from_feature_ptr(feature: *const c_void, _: ThreadingClass) -> Option<Self> {
        (feature as *const sys::LV2_State_Free_Path)
            .as_ref()
            .and_then(|internal| {
                Some(Self {
                    handle: internal.handle,
                    function: internal.free_path?,
                })
            })
    }
}

impl FreePath {
    fn free_path(&mut self, path: &Path) {
        let mut path: Vec<c_char> = path
            .to_str()
            .unwrap()
            .bytes()
            .chain(std::iter::once(0))
            .map(|c| c as c_char)
            .collect();
        unsafe { (self.function)(self.handle, path.as_mut_ptr()) }
    }
}

pub struct PathManager {
    make_path: MakePath,
    map_path: MapPath,
    free_path: Rc<Mutex<FreePath>>,
}

impl PathManager {
    pub fn new(make_path: MakePath, map_path: MapPath, free_path: FreePath) -> Self {
        Self {
            make_path,
            map_path,
            free_path: Rc::new(Mutex::new(free_path)),
        }
    }

    pub fn allocate_path<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<HostManagedPath, HostManagedPathError> {
        Ok(HostManagedPath {
            path: self.make_path.make_path(path.as_ref())?,
            free_path: self.free_path.clone(),
        })
    }

    pub fn map_for_storage(
        &mut self,
        path: &HostManagedPath,
    ) -> Result<String, HostManagedPathError> {
        self.map_path.abstract_path(path.path.as_ref())
    }

    pub fn map_from_storage(
        &mut self,
        stored_path: &str,
    ) -> Result<HostManagedPath, HostManagedPathError> {
        let path = self.map_path.absolute_path(stored_path)?;
        Ok(HostManagedPath {
            path,
            free_path: self.free_path.clone(),
        })
    }
}
