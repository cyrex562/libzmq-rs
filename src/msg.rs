use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::ptr;
use std::mem;
use std::slice;

const CMD_TYPE_MASK: u8 = 0x1c;
const ZMQ_GROUP_MAX_LENGTH: usize = 255; // From zmq.h
const MSG_T_SIZE: usize = 64;
const MAX_VSM_SIZE: usize = MSG_T_SIZE - (mem::size_of::<*mut Metadata>() + 3 + 16 + mem::size_of::<u32>());

// Command names
const CANCEL_CMD_NAME: &[u8] = b"\x06CANCEL";
const SUB_CMD_NAME: &[u8] = b"\x09SUBSCRIBE";
const PING_CMD_NAME_SIZE: usize = 5;
const CANCEL_CMD_NAME_SIZE: usize = 7;
const SUB_CMD_NAME_SIZE: usize = 10;

#[derive(Debug)]
pub struct Metadata {
    ref_count: AtomicI32,
    // Add other metadata fields as needed
}

impl Metadata {
    fn new() -> Self {
        Metadata {
            ref_count: AtomicI32::new(1),
        }
    }

    fn add_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }

    fn drop_ref(&self) -> bool {
        self.ref_count.fetch_sub(1, Ordering::SeqCst) == 1
    }
}

type MsgFreeFn = unsafe extern "C" fn(*mut u8, *mut u8);

#[derive(Debug)]
struct Content {
    data: *mut u8,
    size: usize,
    ffn: Option<MsgFreeFn>,
    hint: *mut u8,
    ref_count: AtomicUsize,
}

impl Drop for Content {
    fn drop(&mut self) {
        if let Some(ffn) = self.ffn {
            unsafe { ffn(self.data as *mut u8, self.hint) };
        }
    }
}

#[derive(Debug)]
enum GroupStorage {
    Short([u8; 15]),
    Long(Box<LongGroup>),
}

#[derive(Debug)]
struct LongGroup {
    group: [u8; ZMQ_GROUP_MAX_LENGTH + 1],
    ref_count: AtomicUsize,
}

#[derive(Debug)]
pub struct Msg {
    metadata: Option<Box<Metadata>>,
    flags: u8,
    routing_id: u32,
    group: GroupStorage,
    content: MsgContent,
}

#[derive(Debug)]
enum MsgContent {
    Vsm {
        data: [u8; MAX_VSM_SIZE],
        size: u8,
    },
    Lmsg {
        content: Box<Content>,
    },
    Cmsg {
        data: *mut u8,
        size: usize,
    },
    Zclmsg {
        content: Box<Content>,
    },
    Delimiter,
    Join,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MsgFlags {
    More = 1,
    Command = 2,
    Ping = 4,
    Pong = 8,
    Subscribe = 12,
    Cancel = 16,
    CloseCmd = 20,
    Credential = 32,
    RoutingId = 64,
    Shared = 128,
}

impl Msg {
    pub fn new() -> Self {
        Msg {
            metadata: None,
            flags: 0,
            routing_id: 0,
            group: GroupStorage::Short([0; 15]),
            content: MsgContent::Vsm {
                data: [0; MAX_VSM_SIZE],
                size: 0,
            },
        }
    }

    pub fn with_size(size: usize) -> Result<Self, &'static str> {
        if size <= MAX_VSM_SIZE {
            let mut msg = Self::new();
            msg.content = MsgContent::Vsm {
                data: [0; MAX_VSM_SIZE],
                size: size as u8,
            };
            Ok(msg)
        } else {
            let content = Box::new(Content {
                data: unsafe { libc::malloc(size) as *mut u8 },
                size,
                ffn: None,
                hint: ptr::null_mut(),
                ref_count: AtomicUsize::new(1),
            });
            
            if content.data.is_null() {
                return Err("Memory allocation failed");
            }

            Ok(Msg {
                metadata: None,
                flags: 0,
                routing_id: 0,
                group: GroupStorage::Short([0; 15]),
                content: MsgContent::Lmsg { content },
            })
        }
    }

    pub fn with_data(data: &[u8]) -> Result<Self, &'static str> {
        let mut msg = Self::with_size(data.len())?;
        msg.copy_from_slice(data);
        Ok(msg)
    }

    pub fn data(&self) -> &[u8] {
        match &self.content {
            MsgContent::Vsm { data, size } => &data[..*size as usize],
            MsgContent::Lmsg { content } => unsafe {
                slice::from_raw_parts(content.data, content.size)
            },
            MsgContent::Cmsg { data, size } => unsafe {
                slice::from_raw_parts(*data, *size)
            },
            MsgContent::Zclmsg { content } => unsafe {
                slice::from_raw_parts(content.data, content.size)
            },
            _ => &[],
        }
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        match &mut self.content {
            MsgContent::Vsm { data, size } => &mut data[..*size as usize],
            MsgContent::Lmsg { content } => unsafe {
                slice::from_raw_parts_mut(content.data, content.size)
            },
            MsgContent::Cmsg { data, size } => unsafe {
                slice::from_raw_parts_mut(*data, *size)
            },
            MsgContent::Zclmsg { content } => unsafe {
                slice::from_raw_parts_mut(content.data, content.size)
            },
            _ => &mut [],
        }
    }

    pub fn size(&self) -> usize {
        match &self.content {
            MsgContent::Vsm { size, .. } => *size as usize,
            MsgContent::Lmsg { content } => content.size,
            MsgContent::Cmsg { size, .. } => *size,
            MsgContent::Zclmsg { content } => content.size,
            _ => 0,
        }
    }

    fn copy_from_slice(&mut self, data: &[u8]) {
        let dst = self.data_mut();
        dst.copy_from_slice(data);
    }

    pub fn set_flags(&mut self, flags: MsgFlags) {
        self.flags |= flags as u8;
    }

    pub fn reset_flags(&mut self, flags: MsgFlags) {
        self.flags &= !(flags as u8);
    }

    pub fn has_flag(&self, flag: MsgFlags) -> bool {
        (self.flags & flag as u8) == flag as u8
    }

    pub fn set_routing_id(&mut self, id: u32) -> Result<(), &'static str> {
        if id == 0 {
            return Err("Invalid routing id");
        }
        self.routing_id = id;
        Ok(())
    }

    pub fn get_routing_id(&self) -> u32 {
        self.routing_id
    }

    pub fn set_group(&mut self, group: &str) -> Result<(), &'static str> {
        let bytes = group.as_bytes();
        if bytes.len() > ZMQ_GROUP_MAX_LENGTH {
            return Err("Group name too long");
        }

        self.group = if bytes.len() > 14 {
            let mut long_group = Box::new(LongGroup {
                group: [0; ZMQ_GROUP_MAX_LENGTH + 1],
                ref_count: AtomicUsize::new(1),
            });
            long_group.group[..bytes.len()].copy_from_slice(bytes);
            GroupStorage::Long(long_group)
        } else {
            let mut short = [0; 15];
            short[..bytes.len()].copy_from_slice(bytes);
            GroupStorage::Short(short)
        };
        Ok(())
    }

    pub fn get_group(&self) -> &str {
        match &self.group {
            GroupStorage::Short(group) => {
                let len = group.iter().position(|&x| x == 0).unwrap_or(14);
                std::str::from_utf8(&group[..len]).unwrap_or("")
            }
            GroupStorage::Long(group) => {
                let len = group.group.iter().position(|&x| x == 0).unwrap_or(ZMQ_GROUP_MAX_LENGTH);
                std::str::from_utf8(&group.group[..len]).unwrap_or("")
            }
        }
    }
}

impl Drop for Msg {
    fn drop(&mut self) {
        // Handle cleanup of content
        match &mut self.content {
            MsgContent::Lmsg { content } => {
                if !self.has_flag(MsgFlags::Shared) || 
                   content.ref_count.fetch_sub(1, Ordering::SeqCst) == 1 {
                    unsafe { libc::free(content.data as *mut libc::c_void); }
                }
            }
            MsgContent::Zclmsg { content } => {
                if !self.has_flag(MsgFlags::Shared) || 
                   content.ref_count.fetch_sub(1, Ordering::SeqCst) == 1 {
                    if let Some(ffn) = content.ffn {
                        unsafe { ffn(content.data, content.hint); }
                    }
                }
            }
            _ => {}
        }

        // Handle cleanup of group
        if let GroupStorage::Long(group) = &self.group {
            if group.ref_count.fetch_sub(1, Ordering::SeqCst) == 1 {
                // Box will handle deallocation
            }
        }
    }
}

// Implementation of Clone would go here if needed
// Implementation of Debug would go here if needed
