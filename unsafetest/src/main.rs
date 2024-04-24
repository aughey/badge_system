use std::mem::MaybeUninit;

pub struct MyMemoryBuffer {
    data: MaybeUninit<[i32; 64]>,
    len: usize,
}
impl MyMemoryBuffer {
    pub fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            len: 0,
        }
    }
    pub fn push(&mut self, value: i32) -> Option<usize> {
        // SAFETY: Getting a mutable reference to the array in an unsafe block
        // The integrity of the initialization and length of the array is maintained within this method.
        let array = unsafe { &mut *self.data.as_mut_ptr() };
        if self.len >= array.len() {
            return None;
        } else {
            array[self.len] = value;
            self.len += 1;
        }
        Some(self.len)
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn as_slice(&self) -> &[i32] {
        // SAFETY: Get a reference to the array in an unsafe block.
        // len was maintained in push correctly, so the slice is valid.
        let array = unsafe { &*self.data.as_ptr() };
        array[..self.len].as_ref()
    }
}

fn main() {
    println!("Hello, world!");
}
