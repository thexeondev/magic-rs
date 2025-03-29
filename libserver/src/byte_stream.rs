use crate::{import, malloc};

pub struct ByteStream(pub *const u8);

impl ByteStream {
    pub fn new(initial_capacity: usize) -> Self {
        import!(byte_stream_ctor(ptr: *const u8, initial_capacity: i32) -> () = 0x2688E6);
        let instance = malloc(40);
        byte_stream_ctor(instance, initial_capacity as i32);
        Self(instance)
    }

    pub fn get_byte_array(&self) -> &[u8] {
        unsafe {
            let byte_array_ptr = *(self.0.wrapping_add(28) as *const *const u8);
            let length = self.get_length();

            std::slice::from_raw_parts(byte_array_ptr, length as usize)
        }
    }

    pub fn get_length(&self) -> i32 {
        unsafe {
            let offset = *(self.0.wrapping_add(16) as *const i32);
            let length = *(self.0.wrapping_add(20) as *const i32);
            std::cmp::max(offset, length)
        }
    }

    pub fn reset_offset(&mut self) {
        unsafe { *(self.0.wrapping_add(16) as *mut i32) = 0 }
    }
}

impl<T> From<T> for ByteStream
where
    T: AsRef<[u8]>,
{
    fn from(value: T) -> Self {
        let value = value.as_ref();
        let stream = ByteStream::new(value.len());
        unsafe {
            std::slice::from_raw_parts_mut(
                *(stream.0.wrapping_add(28) as *const *mut u8),
                value.len(),
            )
            .copy_from_slice(value);
            *(stream.0.wrapping_add(20) as *mut i32) = value.len() as i32;
        }

        stream
    }
}
