use std::cmp;
use std::mem;

// Trait for allocator implementations
pub trait DecoderAllocator {
    fn allocate(&mut self) -> Vec<u8>;
    fn deallocate(&mut self);
    fn size(&self) -> usize;
    fn resize(&mut self, new_size: usize);
}

// Trait for decoder implementations
pub trait Decoder {
    fn get_buffer(&mut self) -> (&mut [u8], usize);
    fn decode(&mut self, data: &[u8], size: usize) -> Result<(usize, bool), std::io::Error>;
    fn resize_buffer(&mut self, new_size: usize);
}

// Base decoder struct
pub struct DecoderBase<T, A> 
where
    A: DecoderAllocator,
{
    next_step: Option<fn(&mut T, &[u8]) -> Result<(), std::io::Error>>,
    read_pos: usize,
    to_read: usize,
    allocator: A,
    buffer: Vec<u8>,
}

impl<T, A> DecoderBase<T, A> 
where
    A: DecoderAllocator,
{
    pub fn new(buf_size: usize, allocator: A) -> Self {
        let mut allocator = allocator;
        let buffer = allocator.allocate();
        
        DecoderBase {
            next_step: None,
            read_pos: 0,
            to_read: 0,
            allocator,
            buffer,
        }
    }

    // Protected method for derived implementations
    protected fn next_step(
        &mut self,
        read_pos: usize,
        to_read: usize,
        next: fn(&mut T, &[u8]) -> Result<(), std::io::Error>
    ) {
        self.read_pos = read_pos;
        self.to_read = to_read;
        self.next_step = Some(next);
    }

    pub fn get_allocator(&mut self) -> &mut A {
        &mut self.allocator
    }
}

impl<T, A> Decoder for DecoderBase<T, A>
where
    A: DecoderAllocator,
{
    fn get_buffer(&mut self) -> (&mut [u8], usize) {
        self.buffer = self.allocator.allocate();

        // Zero-copy optimization for large messages
        if self.to_read >= self.allocator.size() {
            return (&mut self.buffer[self.read_pos..], self.to_read);
        }

        (&mut self.buffer[..], self.allocator.size())
    }

    fn decode(&mut self, data: &[u8], size: usize) -> Result<(usize, bool), std::io::Error> {
        let mut bytes_used = 0;

        // Handle zero-copy case
        if data.as_ptr() == &self.buffer[self.read_pos] as *const u8 {
            debug_assert!(size <= self.to_read);
            self.read_pos += size;
            self.to_read -= size;
            bytes_used = size;

            while self.to_read == 0 {
                if let Some(next) = self.next_step {
                    next(data[bytes_used..])?;
                }
            }
            return Ok((bytes_used, false));
        }

        // Normal copy case
        while bytes_used < size {
            let to_copy = cmp::min(self.to_read, size - bytes_used);
            
            if self.read_pos as *const u8 != data[bytes_used..].as_ptr() {
                self.buffer[self.read_pos..self.read_pos + to_copy]
                    .copy_from_slice(&data[bytes_used..bytes_used + to_copy]);
            }

            self.read_pos += to_copy;
            self.to_read -= to_copy;
            bytes_used += to_copy;

            while self.to_read == 0 {
                if let Some(next) = self.next_step {
                    next(data[bytes_used..])?;
                }
            }
        }

        Ok((bytes_used, false))
    }

    fn resize_buffer(&mut self, new_size: usize) {
        self.allocator.resize(new_size);
    }
}
