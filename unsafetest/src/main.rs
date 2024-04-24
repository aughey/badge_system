use std::mem::MaybeUninit;

struct MyMemoryBuffer {
    data: MaybeUninit<[i32; 64]>,
    len: usize,
}
impl MyMemoryBuffer {
    fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            len: 0,
        }
    }
    fn push(&mut self, value: i32) {
        unsafe {
            self.data.as_mut_ptr().add(self.len).write(value);
            self.len += 1;
        }
    }
    fn len(&self) -> usize {
        self.len
    }
    fn as_slice(&self) -> &[i32] {
        let array = unsafe { &*self.data.as_ptr() };
        array[..self.len].as_ref()
    }
}

fn main() {
    println!("Hello, world!");
}
