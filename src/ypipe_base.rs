pub trait YPipeBase<T> {
    fn write(&mut self, value: &T, incomplete: bool);
    fn unwrite(&mut self) -> Option<T>;
    fn flush(&mut self) -> bool;
    fn check_read(&self) -> bool;
    fn read(&mut self) -> Option<T>;
    fn probe<F>(&self, f: F) -> bool 
    where
        F: Fn(&T) -> bool;
}
