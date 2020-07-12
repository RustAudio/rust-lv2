use lv2_core::feature::Feature;
use lv2_core::prelude::*;
use lv2_sys as sys;
use std::ffi::*;
use std::fs::*;
use std::iter::once;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::path::*;
use std::sync::Mutex;
use urid::*;

pub enum MakePathError {
    PathNotUTF8,
    HostError,
    OpenError(std::io::Error),
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
    pub fn make_path(&mut self, path: &Path) -> Result<PathBuf, MakePathError> {
        let path: Vec<c_char> = path
            .to_str()
            .ok_or(MakePathError::PathNotUTF8)?
            .bytes()
            .chain(once(0))
            .map(|b| b as c_char)
            .collect();

        let path = unsafe { (self.function)(self.handle, path.as_ptr()) };

        if path.is_null() {
            return Err(MakePathError::HostError);
        }

        unsafe { CStr::from_ptr(path) }
            .to_str()
            .map(PathBuf::from)
            .map_err(|_| MakePathError::HostError)
    }
}

pub struct FreePath<'a> {
    handle: sys::LV2_State_Free_Path_Handle,
    function: unsafe extern "C" fn(sys::LV2_State_Free_Path_Handle, *mut c_char),
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
                    function: internal.free_path?,
                    lifetime: PhantomData,
                })
            })
    }
}

impl<'a> FreePath<'a> {
    pub fn free_path(&mut self, path: &Path) {
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

pub struct TempFileGenerator<'a> {
    make_path: MakePath<'a>,
    free_path: Mutex<FreePath<'a>>,
}

impl<'a> TempFileGenerator<'a> {
    pub fn new(make_path: MakePath<'a>, free_path: FreePath<'a>) -> Self {
        TempFileGenerator {
            make_path,
            free_path: Mutex::new(free_path),
        }
    }

    pub fn make_temp_file<'b>(
        &'b mut self,
        path: &Path,
        options: OpenOptions,
    ) -> Result<TempFile<'b, 'a>, MakePathError> {
        let path = self.make_path.make_path(path)?;

        let file = options.open(&path).map_err(MakePathError::OpenError)?;

        Ok(TempFile {
            file,
            path,
            generator: self,
        })
    }
}

pub struct TempFile<'a, 'b> {
    file: File,
    path: PathBuf,
    generator: &'a TempFileGenerator<'b>,
}

impl<'a, 'b> std::ops::Deref for TempFile<'a, 'b> {
    type Target = File;

    fn deref(&self) -> &File {
        &self.file
    }
}

impl<'a, 'b> std::ops::DerefMut for TempFile<'a, 'b> {
    fn deref_mut(&mut self) -> &mut File {
        &mut self.file
    }
}

impl<'a, 'b> std::ops::Drop for TempFile<'a, 'b> {
    fn drop(&mut self) {
        self.generator
            .free_path
            .lock()
            .unwrap()
            .free_path(self.path.as_path());
    }
}
