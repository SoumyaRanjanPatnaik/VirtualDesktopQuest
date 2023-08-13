//! # SharedMemMap
//!
//! SharedMemMap is a memory safe wrapper around the operations shm_open, memmap and shm_close
//! functions from the libc ffi

use derive_builder::Builder;
use libc::{
    c_char, c_void, ftruncate64, mmap64, munmap, shm_open, MAP_PRIVATE, O_CREAT, O_EXCL, O_RDONLY,
    O_RDWR, O_WRONLY, PROT_EXEC, PROT_READ, PROT_WRITE,
};
use std::{
    ffi::CString,
    ops::{Deref, DerefMut},
    os::fd::RawFd,
    path::PathBuf,
    ptr::null_mut,
};

/// # ShmFactory
/// Factory for creating [SharedMemMap](SharedMemMap) objects
#[allow(dead_code)]
#[derive(Clone, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct ShmFactory {
    #[builder(setter(into))]
    name: String,
    size: usize,
    #[builder(default = "None")]
    scope: Option<String>,
    #[builder(default = "false")]
    read: bool,
    #[builder(default = "false")]
    write: bool,
    #[builder(default = "true")]
    create: bool,
    #[builder(default = "true")]
    rw: bool,
    #[builder(default = "false")]
    exec: bool,
    #[builder(default = "false")]
    check_exists: bool,
}

impl ShmFactoryBuilder {
    fn validate(&self) -> Result<(), String> {
        if self.name == None {
            Err(String::from("'name' is a required field"))
        } else if self.size == None {
            Err(String::from("'size' is a required field"))
        } else {
            Ok(())
        }
    }
}

#[allow(dead_code)]
impl ShmFactory {
    /// Create a new [SharedMemMap](SharedMemMap) object with a static lifetime
    pub fn create_shm_map<'a>(self) -> Option<SharedMemMap<'a>> {
        let runtime_dir = std::env::vars().find(|var| var.0 == "XDG_RUNTIME_DIR")?.1;
        let mut rd_path = PathBuf::from(&runtime_dir);
        if let Some(scope_val) = self.scope {
            rd_path = rd_path.join(scope_val);
        }
        rd_path = rd_path.join(self.name);
        let shm_path: String = rd_path.to_string_lossy().to_string();
        let shm_path_cstring = CString::new(shm_path).ok()?;
        let shm_path_raw = shm_path_cstring.to_bytes_with_nul().as_ptr() as *const c_char;
        let mut shm_flag: i32 = 0;
        let mut mode = 0;

        if self.create {
            shm_flag = shm_flag | O_CREAT;
        }
        if self.write {
            shm_flag = shm_flag | O_WRONLY;
            mode += 200;
        }
        if self.read {
            shm_flag = shm_flag | O_RDONLY;
            mode += 400;
        }
        if self.rw {
            shm_flag = shm_flag | O_RDWR;
            mode = 600;
        }
        if self.exec {
            mode += 100;
        }
        if self.check_exists {
            shm_flag = shm_flag | O_EXCL;
        }

        let shm_fd_raw: RawFd = unsafe { shm_open(shm_path_raw, shm_flag, mode) };
        let _is_error = unsafe {
            let ret = ftruncate64(shm_fd_raw, self.size.try_into().ok()?);
            ret == -1
        };
        let mut map_flags = 0;
        if self.rw {
            map_flags = map_flags | PROT_READ;
            map_flags = map_flags | PROT_WRITE;
        } else if self.read {
            map_flags = map_flags | PROT_READ;
        } else if self.write {
            map_flags = map_flags | PROT_WRITE;
        }
        if self.exec {
            map_flags = map_flags | PROT_EXEC;
        }
        let map_addr = unsafe {
            mmap64(
                null_mut(),
                self.size as usize,
                map_flags,
                MAP_PRIVATE,
                shm_fd_raw,
                0,
            )
        };
        let mem_slice = unsafe {
            std::slice::from_raw_parts_mut(map_addr as *mut u8, self.size.try_into().ok()?)
        };
        Some(SharedMemMap {
            fd: shm_fd_raw,
            shm: mem_slice,
            addr: map_addr,
            size: self.size as usize,
        })
    }
}

/// # SharedMemMap
/// Represents a type-safe abstraction around shared memory map. Obtained by applying memmap on the [RawFd]
/// obtained from [shm_open]. Use [ShmFactory] for creating a new instance of [SharedMemMap]
///
/// Example:
/// ```
/// let shm_factory = ShmFactoryBuilder::default()
/// .name("/wl_scrcpy_vd")
/// .size(1920 * 1080)
/// .rw(true)
/// .create(true)
/// .build()
/// .unwrap();
/// let mut shm = shm_factory.create_shm_map().ok_or("Failed to create shm")?;
///
/// ```
#[allow(dead_code)]
pub struct SharedMemMap<'a> {
    shm: &'a mut [u8],
    addr: *mut c_void,
    fd: RawFd,
    size: usize,
}

#[allow(dead_code)]
impl SharedMemMap<'_> {
    pub fn len(&self) -> usize {
        self.size
    }
    pub unsafe fn fd(&self) -> RawFd {
        self.fd
    }
}

impl Deref for SharedMemMap<'_> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        return self.shm;
    }
}

impl DerefMut for SharedMemMap<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.shm
    }
}

impl Drop for SharedMemMap<'_> {
    fn drop(&mut self) {
        unsafe { munmap(self.addr, self.size) };
    }
}
