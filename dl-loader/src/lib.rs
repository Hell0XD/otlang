use std::ffi::{CString, OsStr};

pub struct DynamicLibrary {
    handle: *mut u8,
}

impl Drop for DynamicLibrary {
    fn drop(&mut self) {
        dl::check_for_errors_in(|| dl::close(self.handle)).unwrap();
    }
}

impl DynamicLibrary {
    pub fn open(filename: &OsStr) -> Result<Self, String> {
        dl::open(filename).map(|handle| DynamicLibrary { handle })
    }

    pub unsafe fn symbol(&self, symbol: &str) -> Result<*mut u8, String> {
        let raw_string = CString::new(symbol).unwrap();
        dl::check_for_errors_in(|| {
            dl::symbol(
                self.handle as *mut libc::c_void,
                raw_string.as_ptr() as *const libc::c_char,
            )
        })
    }
}

#[cfg(target_family = "unix")]
mod dl {
    use std::ffi::{CStr, CString, OsStr};
    use std::os::unix::ffi::OsStrExt;
    use std::ptr;
    use std::str;

    // Source: https://linux.die.net/man/3/dlopen
    // Perform lazy binding. Only resolve symbols as the code that references them is executed.
    // If the symbol is never referenced, then it is never resolved.
    // (Lazy binding is only performed for function references; references to variables
    // are always immediately bound when the library is loaded.)
    const LOAD_SO_LAZILY: libc::c_int = 1;

    pub fn open(filename: &OsStr) -> Result<*mut u8, String> {
        check_for_errors_in(|| unsafe {
            let s = CString::new(filename.as_bytes().to_vec()).unwrap();
            dlopen(s.as_ptr() as *const libc::c_char, LOAD_SO_LAZILY) as *mut u8
        })
    }

    pub fn check_for_errors_in<T, F>(f: F) -> Result<T, String>
    where
        F: FnOnce() -> T,
    {
        unsafe {
            // !!!THIS IS NOT THREAD SAFE LIKE ON WINDOWS!!!
            let _old_error = dlerror();

            let result = f();

            let last_error = dlerror() as *const _;
            let ret = if last_error == ptr::null() {
                Ok(result)
            } else {
                let s = CStr::from_ptr(last_error).to_bytes();
                Err(str::from_utf8(s).unwrap().to_string())
            };

            ret
        }
    }

    pub unsafe fn symbol(handle: *mut libc::c_void, symbol: *const libc::c_char) -> *mut u8 {
        dlsym(handle, symbol) as *mut u8
    }

    pub fn close(handle: *mut u8) {
        unsafe {
            dlclose(handle as *mut libc::c_void);
        }
    }

    extern "C" {
        fn dlerror() -> *mut libc::c_char;
        fn dlopen(filename: *const libc::c_char, flag: libc::c_int) -> *mut libc::c_void;
        fn dlsym(handle: *mut libc::c_void, symbol: *const libc::c_char) -> *mut libc::c_void;
        fn dlclose(handle: *mut libc::c_void) -> libc::c_int;
    }
}

#[cfg(target_os = "windows")]
mod dl {
    use std::ffi::OsStr;
    use std::io::Error as IoError;
    use std::os::windows::prelude::*;
    use std::ptr;

    // disable "dll load failed" error dialog.
    // SEM_FAILCRITICALERRORS 0x01
    const NEW_ERROR_MODE: u32 = 0x01;

    pub fn open(filename: &OsStr) -> Result<*mut u8, String> {
        let prev_error_mode = unsafe { SetErrorMode(NEW_ERROR_MODE) };

        unsafe {
            SetLastError(0x0);
        }

        let result = {
            let filename_str: Vec<_> = filename.encode_wide().chain(Some(0).into_iter()).collect();
            let result = unsafe { LoadLibraryW(filename_str.as_ptr() as *const libc::c_void) };

            if result == ptr::null_mut() {
                Err(format!("{}", IoError::last_os_error()))
            } else {
                Ok(result as *mut u8)
            }
        };

        unsafe {
            SetErrorMode(prev_error_mode);
        }

        result
    }

    pub fn check_for_errors_in<T, F>(f: F) -> Result<T, String>
    where
        F: FnOnce() -> T,
    {
        unsafe {
            SetLastError(0);

            let result = f();

            let error = IoError::last_os_error(); // GetLastError Win32
            if 0 == error.raw_os_error().unwrap() {
                Ok(result)
            } else {
                Err(format!("{}", error))
            }
        }
    }

    pub unsafe fn symbol(handle: *mut libc::c_void, symbol: *const libc::c_char) -> *mut u8 {
        GetProcAddress(handle, symbol) as *mut u8
    }
    pub fn close(handle: *mut u8) {
        unsafe { FreeLibrary(handle as *mut libc::c_void) }
    }

    #[allow(non_snake_case)]
    extern "system" {
        fn SetErrorMode(uMode: libc::c_uint) -> libc::c_uint;
        fn SetLastError(error: libc::size_t);
        fn LoadLibraryW(name: *const libc::c_void) -> *mut libc::c_void;
        fn GetProcAddress(
            handle: *mut libc::c_void,
            name: *const libc::c_char,
        ) -> *mut libc::c_void;
        fn FreeLibrary(handle: *mut libc::c_void);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem;

    #[test]
    fn tt() {
        #[cfg(target_os = "windows")]
        let path = "./tmp.dll";
        #[cfg(target_family = "unix")]
        let path = "./libtmp.so";

        let libm = match DynamicLibrary::open(OsStr::new(path)) {
            Err(error) => panic!("Could not load self as module: {}", error),
            Ok(libm) => libm,
        };

        let sum: extern "C" fn(i32, i32) -> i32 = unsafe {
            match libm.symbol("sum") {
                Err(error) => panic!("Could not load function sum: {}", error),
                Ok(sum) => mem::transmute::<*mut u8, _>(sum),
            }
        };

        assert_eq!(sum(4, 6), 4 + 6);
    }
}
