pub struct RingBuffer<T> {
    buffer: Vec<T>,
    read_index: usize,
    write_index: usize,
    capacity: usize,
    size: usize,
}

impl<T: Copy + Default> RingBuffer<T> {
    pub fn new(length: usize) -> Self {
        RingBuffer {
            buffer: vec![T::default(); length],
            read_index: 0,
            write_index: 0,
            capacity: length,
            size: 0,
        }
    }

    pub fn reset(&mut self) {
        self.read_index = 0;
        self.write_index = 0;
        self.size = 0;
        self.buffer.fill(T::default());
    }

    pub fn put(&mut self, value: T) {
        self.buffer[self.write_index] = value;
        
    }

    pub fn peek(&self) -> T {
        self.buffer[self.read_index]
    }


    pub fn push(&mut self, value: T) {
        
        self.buffer[self.write_index] = value;

        
        if self.size == self.capacity {
            self.read_index = (self.read_index + 1) % self.capacity;
        } else {
            
            self.size += 1;
        }

        
        self.write_index = (self.write_index + 1) % self.capacity;
    }

    pub fn pop(&mut self) -> T {
        if self.size == 0 {
            
            T::default()
        } else {
            
            let value = self.buffer[self.read_index];
            self.read_index = (self.read_index + 1) % self.capacity;
            self.size -= 1; 
            value
        }
    }

    pub fn get(&self, offset: usize) -> T {
        
        let index = (self.read_index + offset) % self.capacity;
        self.buffer[index]
    }


    pub fn get_read_index(&self) -> usize {
        self.read_index
    }

    pub fn set_read_index(&mut self, index: usize) {
        self.read_index = index % self.capacity;
    }

    pub fn get_write_index(&self) -> usize {
        self.write_index
    }

    pub fn set_write_index(&mut self, index: usize) {
        self.write_index = index % self.capacity;
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::RingBuffer;

    #[test]
    fn test_initialization_and_capacity() {
        let buffer: RingBuffer<i32> = RingBuffer::new(5);
        assert_eq!(buffer.capacity(), 5);
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_push_and_size() {
        let mut buffer = RingBuffer::new(3);
        buffer.push(1);
        buffer.push(2);
        assert_eq!(buffer.len(), 2);
        buffer.push(3);
        assert_eq!(buffer.len(), 3);
        
        buffer.push(4);
        assert_eq!(buffer.len(), 3);
    }

    #[test]
    fn test_pop() {
        let mut buffer = RingBuffer::new(3);
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        assert_eq!(buffer.pop(), 1);
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.pop(), 2);
        assert_eq!(buffer.pop(), 3);
        
        assert_eq!(buffer.pop(), 0);
    }

    #[test]
    fn test_overwrite() {
        let mut buffer = RingBuffer::new(2);
        buffer.push(1);
        buffer.push(2);
        
        buffer.push(3);
        assert_eq!(buffer.pop(), 2);
        assert_eq!(buffer.pop(), 3);
    }

    #[test]
    fn test_wrap_around() {
        let mut buffer = RingBuffer::new(3);
        for i in 1..=5 {
            buffer.push(i);
        }
        
        assert_eq!(buffer.pop(), 3);
        assert_eq!(buffer.pop(), 4);
        assert_eq!(buffer.pop(), 5);
    }

    #[test]
    fn test_get_with_offset() {
        let mut buffer = RingBuffer::new(5);
        for i in 0..5 {
            buffer.push(i);
        }
        
        assert_eq!(buffer.get(0), 0); 
        assert_eq!(buffer.get(4), 4); 
        
        buffer.push(5); 
        assert_eq!(buffer.get(0), 1); 
        assert_eq!(buffer.get(4), 5); 
    }
}
