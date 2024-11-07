use std::marker::PhantomData;

// Trait to replace the C++ ypipe_base
pub trait YPipeBase<T> {
    fn write(&mut self, value: T);
    fn unwrite(&mut self) -> Option<T>;
    fn flush(&self) -> bool;
    fn check_read(&mut self) -> bool;
    fn read(&mut self) -> Option<T>;
    fn probe<F>(&self, f: F) -> bool 
    where F: Fn(&T) -> bool;
}

// DBBuffer equivalent for Rust
struct DBuffer<T> {
    value: Option<T>,
}

impl<T> DBuffer<T> {
    fn new() -> Self {
        DBuffer { value: None }
    }

    fn write(&mut self, value: T) {
        self.value = Some(value);
    }

    fn read(&mut self) -> Option<T> {
        self.value.take()
    }

    fn check_read(&self) -> bool {
        self.value.is_some()
    }

    fn probe<F>(&self, f: F) -> bool 
    where F: Fn(&T) -> bool {
        self.value.as_ref().map_or(false, f)
    }
}

// Main YPipeConflate implementation
pub struct YPipeConflate<T> {
    dbuffer: DBuffer<T>,
    reader_awake: bool,
    _phantom: PhantomData<T>,
}

impl<T> YPipeConflate<T> {
    pub fn new() -> Self {
        YPipeConflate {
            dbuffer: DBuffer::new(),
            reader_awake: false,
            _phantom: PhantomData,
        }
    }
}

impl<T> YPipeBase<T> for YPipeConflate<T> {
    fn write(&mut self, value: T) {
        self.dbuffer.write(value);
    }

    fn unwrite(&mut self) -> Option<T> {
        None // Conflate pipe doesn't support unwrite
    }

    fn flush(&self) -> bool {
        self.reader_awake
    }

    fn check_read(&mut self) -> bool {
        let res = self.dbuffer.check_read();
        if !res {
            self.reader_awake = false;
        }
        res
    }

    fn read(&mut self) -> Option<T> {
        if !self.check_read() {
            return None;
        }
        self.dbuffer.read()
    }

    fn probe<F>(&self, f: F) -> bool 
    where F: Fn(&T) -> bool {
        self.dbuffer.probe(f)
    }
}

// Implement Clone restrictions
impl<T> YPipeConflate<T> {
    pub fn clone(&self) -> ! {
        panic!("YPipeConflate cannot be cloned")
    }
}
