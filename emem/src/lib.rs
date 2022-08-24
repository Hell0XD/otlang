use std::fmt;
use std::ptr;

// round up to page size
fn round_up(from: usize, to: usize) -> usize {
    if from % to == 0 && from != 0 {
        from
    } else {
        from + to - (from % to)
    }
}

fn errno() -> i32 {
    // because Error was constructed by last_os_error there is no need for checking the result
    std::io::Error::last_os_error().raw_os_error().unwrap()
}

#[cfg(unix)]
fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

#[cfg(windows)]
fn page_size() -> usize {
    unsafe {
        let mut info = std::mem::zeroed();
        libc::GetSystemInfo(&mut info);
        return info.dwPageSize as usize;
    }
}

pub struct ExecutableMemory {
    data: *mut u8,
    len: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum MapError {
    /// # The following are POSIX-specific
    ///
    /// fd was not open for reading or, if using `MapWritable`, was not open for
    /// writing.
    ErrFdNotAvail,
    /// fd was not valid
    ErrInvalidFd,
    /// Either the address given by `MapAddr` or offset given by `MapOffset` was
    /// not a multiple of `ExecutableMemory::granularity` (unaligned to page size).
    ErrUnaligned,
    /// With `MapFd`, the fd does not support mapping.
    ErrNoMapSupport,
    /// If using `MapAddr`, the address + `min_len` was outside of the process's
    /// address space. If using `MapFd`, the target of the fd didn't have enough
    /// resources to fulfill the request.
    ErrNoMem,
    /// A zero-length map was requested. This is invalid according to
    /// [POSIX](http://pubs.opengroup.org/onlinepubs/9699919799/functions/mmap.html).
    /// Not all platforms obey this, but this wrapper does.
    ErrZeroLength,
    /// Unrecognized error. The inner value is the unrecognized errno.
    ErrUnknown(isize),
    /// Unrecognized error from `VirtualAlloc`. The inner value is the return
    /// value of GetLastError.
    ErrVirtualAlloc(i32),
}

impl fmt::Display for MapError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        let str = match *self {
            Self::ErrFdNotAvail => "fd not available for reading or writing",
            Self::ErrInvalidFd => "Invalid fd",
            Self::ErrUnaligned => {
                "Unaligned address, invalid flags, negative length or \
                 unaligned offset"
            }
            Self::ErrNoMapSupport => "File doesn't support mapping",
            Self::ErrNoMem => "Invalid address, or not enough available memory",
            Self::ErrZeroLength => "Zero-length mapping not allowed",
            Self::ErrUnknown(code) => return write!(out, "Unknown error = {}", code),
            Self::ErrVirtualAlloc(code) => return write!(out, "VirtualAlloc failure = {}", code),
        };
        write!(out, "{}", str)
    }
}

#[cfg(unix)]
impl ExecutableMemory {
    pub fn new(min_len: usize) -> Result<ExecutableMemory, MapError> {
        if min_len == 0 {
            return Err(MapError::ErrZeroLength);
        }
        let addr: *const u8 = ptr::null();
        let len = round_up(min_len, page_size());

        let r = unsafe {
            libc::mmap(
                addr as *mut libc::c_void,
                len as libc::size_t,
                libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            )
        };
        if r == libc::MAP_FAILED {
            Err(match errno() {
                libc::EACCES => MapError::ErrFdNotAvail,
                libc::EBADF => MapError::ErrInvalidFd,
                libc::EINVAL => MapError::ErrUnaligned,
                libc::ENODEV => MapError::ErrNoMapSupport,
                libc::ENOMEM => MapError::ErrNoMem,
                code => MapError::ErrUnknown(code as isize),
            })
        } else {
            Ok(ExecutableMemory {
                data: r as *mut u8,
                len,
            })
        }
    }
}

#[cfg(unix)]
impl Drop for ExecutableMemory {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.data as *mut libc::c_void, self.len as libc::size_t);
        }
    }
}

#[cfg(windows)]
impl ExecutableMemory {
    pub fn new(min_len: usize) -> Result<ExecutableMemory, MapError> {
        // To keep compatibility with unix
        if min_len == 0 {
            return Err(MapError::ErrZeroLength);
        }

        let len = round_up(min_len, page_size());

        let ptr = unsafe {
            libc::VirtualAlloc(
                ptr::null_mut(),
                len as libc::SIZE_T,
                libc::MEM_COMMIT | libc::MEM_RESERVE,
                libc::PAGE_EXECUTE_READWRITE,
            )
        };

        match ptr as usize {
            // check if its a null pointer
            0 => Err(MapError::ErrVirtualAlloc(errno())),
            _ => Ok(ExecutableMemory {
                data: ptr as *mut u8,
                len,
            }),
        }
    }
}

#[cfg(windows)]
impl Drop for ExecutableMemory {
    fn drop(&mut self) {
        unsafe {
            if libc::VirtualFree(self.data as *mut libc::c_void, 0, libc::MEM_RELEASE) == 0 {
                println!("VirtualFree failed: {}", errno());
            }
        }
    }
}

impl ExecutableMemory {
    /// Returns the pointer to the memory created or modified by this map.
    #[inline(always)]
    #[allow(unused)]
    pub fn data(&self) -> *mut u8 {
        self.data
    }

    /// Returns the number of bytes this map applies to.
    #[inline(always)]
    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use super::ExecutableMemory;

    #[test]
    fn it_works() {
        let mem = ExecutableMemory::new(256).unwrap();
        
        unsafe {
            *mem.data() = 5;
            assert_eq!(*mem.data(), 5);
        }

    }
}
