use std::borrow::Borrow;
use std::error::Error;
use std::ffi::{CStr, CString, NulError};
use std::fmt;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub enum UriError {
    CStrNulError(NulError)
}

impl Error for UriError {}

impl fmt::Display for UriError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Invalid URI String: ")?;
        match self {
            UriError::CStrNulError(e) => fmt::Display::fmt(e, f)
        }
    }
}

// TODO: Check Eq impl ?
#[derive(PartialEq, Eq)]
pub struct Uri {
    inner: CStr
}

impl Uri {
    #[inline]
    pub fn from_cstr(string: &CStr) -> Result<&Uri, UriError> {
        // TODO: check this actually is a valid url
        Ok(unsafe { Uri::from_cstr_unchecked(string) })
    }

    #[inline]
    pub unsafe fn from_cstr_unchecked(string: &CStr) -> &Uri {
        &*(string as *const CStr as *const Uri)
    }

    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Uri {
        Uri::from_cstr_unchecked(CStr::from_bytes_with_nul_unchecked(bytes))
    }

    #[inline]
    pub const fn as_cstr(&self) -> &CStr {
        &self.inner
    }
}

impl fmt::Debug for Uri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("<")?;
        fmt::Display::fmt(self.to_string_lossy().deref(), f)?;
        f.write_str(">")
    }
}

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.to_string_lossy().deref(), f)
    }
}

impl Deref for Uri {
    type Target = CStr;

    #[inline]
    fn deref(&self) -> &CStr {
        &self.inner
    }
}

impl ToOwned for Uri {
    type Owned = UriBuf;

    #[inline]
    fn to_owned(&self) -> UriBuf {
        unsafe { UriBuf::from_cstring_unchecked(self.inner.to_owned()) }
    }
}

impl<'a> Into<UriBuf> for &'a Uri {
    #[inline]
    fn into(self) -> UriBuf {
        self.to_owned()
    }
}

impl PartialEq<UriBuf> for Uri {
    fn eq(&self, other: &UriBuf) -> bool {
        other == self
    }
}

impl PartialEq<CStr> for Uri {
    fn eq(&self, other: &CStr) -> bool {
        &self.inner == other
    }
}

impl PartialEq<CString> for Uri {
    fn eq(&self, other: &CString) -> bool {
        &self.inner == other as &CStr
    }
}

pub trait AsUriRef {
    fn as_uri(&self) -> Result<&Uri, UriError>;
}

impl AsUriRef for Uri {
    fn as_uri(&self) -> Result<&Uri, UriError> {
        Ok(self)
    }
}

impl<'a> AsUriRef for &'a Uri {
    fn as_uri(&self) -> Result<&Uri, UriError> {
        Ok(*self)
    }
}

impl AsUriRef for UriBuf {
    fn as_uri(&self) -> Result<&Uri, UriError> {
        Ok(self.borrow())
    }
}

impl<T: AsUriRef> AsUriRef for Result<T, UriError> {
    fn as_uri(&self) -> Result<&Uri, UriError> {
        match self {
            Ok(uri) => uri.as_uri(),
            Err(e) => Err(e.clone())
        }
    }
}

impl AsUriRef for CStr {
    fn as_uri(&self) -> Result<&Uri, UriError> {
        Uri::from_cstr(self)
    }
}

impl AsUriRef for CString {
    fn as_uri(&self) -> Result<&Uri, UriError> {
        Uri::from_cstr(self)
    }
}

#[derive(PartialEq, Eq)]
pub struct UriBuf {
    inner: CString
}

impl UriBuf {
    #[inline]
    pub fn from_cstring(string: CString) -> Result<UriBuf, UriError> {
        // TODO: check this actually is a valid url
        Ok(unsafe { UriBuf::from_cstring_unchecked(string) })
    }

    #[inline]
    pub unsafe fn from_cstring_unchecked(string: CString) -> UriBuf {
        UriBuf { inner: string }
    }

    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: Vec<u8>) -> UriBuf {
        UriBuf::from_cstring_unchecked(CString::from_vec_unchecked(bytes))
    }
}

impl Borrow<Uri> for UriBuf {
    #[inline]
    fn borrow(&self) -> &Uri {
        unsafe { Uri::from_cstr_unchecked(&self.inner) }
    }
}

impl Deref for UriBuf {
    type Target = CString;

    #[inline]
    fn deref(&self) -> &CString {
        &self.inner
    }
}

impl DerefMut for UriBuf {
    #[inline]
    fn deref_mut(&mut self) -> &mut CString {
        &mut self.inner
    }
}

impl fmt::Debug for UriBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Uri as fmt::Debug>::fmt(UriBuf::borrow(self), f)
    }
}

impl fmt::Display for UriBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Uri as fmt::Display>::fmt(self.borrow(), f)
    }
}

impl PartialEq<Uri> for UriBuf {
    fn eq(&self, other: &Uri) -> bool {
        self.borrow() as &Uri == other
    }
}

impl Into<CString> for UriBuf {
    fn into(self) -> CString {
        self.inner
    }
}

pub trait IntoUri {
    fn into_uri(self) -> Result<UriBuf, UriError>;
    unsafe fn into_uri_unchecked(self) -> UriBuf;
}

impl<'a> IntoUri for &'a str {
    fn into_uri(self) -> Result<UriBuf, UriError> {
        let cstr = CString::new(self).map_err(UriError::CStrNulError)?;
        UriBuf::from_cstring(cstr)
    }

    unsafe fn into_uri_unchecked(self) -> UriBuf {
        UriBuf::from_cstring_unchecked(CString::from_vec_unchecked(self.into()))
    }
}

pub unsafe trait UriBound {
    const URI: &'static [u8];

    #[inline]
    fn uri() -> &'static Uri {
        unsafe { Uri::from_bytes_unchecked(Self::URI) }
    }
}
