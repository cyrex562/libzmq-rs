use std::cmp::Ordering;
use std::ptr;
use std::slice;

pub struct ReferenceTag;

pub struct Blob {
    data: *mut u8,
    size: usize,
    owned: bool,
}

impl Blob {
    // Creates an empty Blob.
    pub fn new() -> Self {
        Blob {
            data: ptr::null_mut(),
            size: 0,
            owned: true,
        }
    }

    // Creates a Blob of a given size, with uninitialized content.
    pub fn with_size(size: usize) -> Self {
        let data = if size > 0 {
            unsafe { libc::malloc(size) as *mut u8 }
        } else {
            ptr::null_mut()
        };
        Blob {
            data,
            size,
            owned: true,
        }
    }

    // Creates a Blob of a given size, and initializes content by copying from another buffer.
    pub fn from_data(data: &[u8]) -> Self {
        let size = data.len();
        let blob_data = if size > 0 {
            unsafe { libc::malloc(size) as *mut u8 }
        } else {
            ptr::null_mut()
        };
        if size > 0 && !blob_data.is_null() {
            unsafe {
                ptr::copy_nonoverlapping(data.as_ptr(), blob_data, size);
            }
        }
        Blob {
            data: blob_data,
            size,
            owned: true,
        }
    }

    // Creates a Blob for temporary use that only references a pre-allocated block of data.
    pub fn from_reference(data: &mut [u8], _tag: ReferenceTag) -> Self {
        Blob {
            data: data.as_mut_ptr(),
            size: data.len(),
            owned: false,
        }
    }

    // Returns the size of the Blob.
    pub fn size(&self) -> usize {
        self.size
    }

    // Returns a pointer to the data of the Blob.
    pub fn data(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data, self.size) }
    }

    // Returns a mutable pointer to the data of the Blob.
    pub fn data_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data, self.size) }
    }

    // Defines an order relationship on Blob.
    pub fn cmp(&self, other: &Blob) -> Ordering {
        let min_size = self.size.min(other.size);
        let cmp_res = unsafe { libc::memcmp(self.data as *const _, other.data as *const _, min_size) };
        if cmp_res < 0 {
            Ordering::Less
        } else if cmp_res > 0 {
            Ordering::Greater
        } else {
            self.size.cmp(&other.size)
        }
    }

    // Sets a Blob to a deep copy of another Blob.
    pub fn set_deep_copy(&mut self, other: &Blob) {
        self.clear();
        self.data = if other.size > 0 {
            unsafe { libc::malloc(other.size) as *mut u8 }
        } else {
            ptr::null_mut()
        };
        self.size = other.size;
        self.owned = true;
        if self.size > 0 && !self.data.is_null() {
            unsafe {
                ptr::copy_nonoverlapping(other.data, self.data, self.size);
            }
        }
    }

    // Sets a Blob to a copy of a given buffer.
    pub fn set(&mut self, data: &[u8]) {
        self.clear();
        self.data = if data.len() > 0 {
            unsafe { libc::malloc(data.len()) as *mut u8 }
        } else {
            ptr::null_mut()
        };
        self.size = data.len();
        self.owned = true;
        if self.size > 0 && !self.data.is_null() {
            unsafe {
                ptr::copy_nonoverlapping(data.as_ptr(), self.data, self.size);
            }
        }
    }

    // Empties a Blob.
    pub fn clear(&mut self) {
        if self.owned && !self.data.is_null() {
            unsafe {
                libc::free(self.data as *mut _);
            }
        }
        self.data = ptr::null_mut();
        self.size = 0;
    }
}

impl Drop for Blob {
    fn drop(&mut self) {
        if self.owned && !self.data.is_null() {
            unsafe {
                libc::free(self.data as *mut _);
            }
        }
    }
}

impl Clone for Blob {
    fn clone(&self) -> Self {
        let mut new_blob = Blob::new();
        new_blob.set_deep_copy(self);
        new_blob
    }
}

impl PartialEq for Blob {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && unsafe { libc::memcmp(self.data as *const _, other.data as *const _, self.size) } == 0
    }
}

impl Eq for Blob {}

impl PartialOrd for Blob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Blob {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}
