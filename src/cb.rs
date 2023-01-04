/// Generic CircularBuffer where oldest element are overwritten
/// by the new ones.
///
/// Usage example:
/// ```
/// extern crate logss;
/// use logss::cb;
/// let mut cb:cb::CircularBuffer<i32> = cb::CircularBuffer::new(3);
/// cb.push(1);
/// cb.push(2);
/// cb.push(3);
/// cb.push(4);
/// assert_eq!(cb.buffer, vec![4, 2, 3]);
///
/// // Notice the order that clone returns
/// let cb2 = cb.clone();
/// assert_eq!(cb2.buffer, vec![2, 3, 4]);
/// ```
#[derive(Debug)]
pub struct CircularBuffer<T> {
    /// Actual buffer
    pub buffer: Vec<T>,
    /// index that keeps track of the writes
    write_index: usize,
}

impl<T> CircularBuffer<T>
where
    T: Clone,
{
    /// Constructs a new instance of [`CircularBuffer`].
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            write_index: 0,
        }
    }

    /// Returns the buffer capacity
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Returns the buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns ttue if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Push an element into the buffer
    pub fn push(&mut self, element: T) {
        let capacity = self.capacity();
        let len = self.len();

        if len < capacity {
            self.buffer.push(element);
        } else {
            self.buffer[self.write_index % capacity] = element;
        }

        self.write_index += 1;
    }

    /// Clones and returns a new instance of [`CircularBuffer`] in the write order
    pub fn ordered_clone(&self) -> CircularBuffer<T> {
        let capacity = self.capacity();
        let len = self.len();
        let mut cb = CircularBuffer::new(capacity);

        if len < capacity {
            cb.buffer = self.buffer.clone();
        } else {
            let index_position = self.write_index % capacity;
            cb.buffer.extend_from_slice(&self.buffer[index_position..]);
            cb.buffer.extend_from_slice(&self.buffer[..index_position]);
        }

        cb
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn circular_buffer() {
        let mut cb: CircularBuffer<i32> = CircularBuffer::new(3);
        assert_eq!(cb.capacity(), 3);
        assert_eq!(cb.len(), 0);

        cb.push(1);
        assert_eq!(cb.buffer, vec![1]);
        assert_eq!(cb.len(), 1);

        cb.push(2);
        cb.push(3);
        assert_eq!(cb.buffer, vec![1, 2, 3]);
        assert_eq!(cb.len(), 3);

        cb.push(4);
        assert_eq!(cb.buffer, vec![4, 2, 3]);
        assert_eq!(cb.len(), 3);
    }

    #[test]
    fn circular_buffer_clone() {
        let mut cb: CircularBuffer<i32> = CircularBuffer::new(3);
        cb.push(1);
        cb.push(2);
        cb.push(3);
        cb.push(4);

        let cb2 = cb.clone();
        assert_eq!(cb2.buffer, vec![2, 3, 4]);
    }
}
