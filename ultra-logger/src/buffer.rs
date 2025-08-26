//! Lock-free ring buffer implementation for ultra-low latency logging

use crate::error::{LoggingError, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use crossbeam_utils::CachePadded;
use parking_lot::RwLock;

/// Lock-free ring buffer optimized for single-producer, single-consumer
pub struct RingBuffer<T> {
    buffer: Vec<RwLock<Option<T>>>,
    capacity: usize,
    write_pos: CachePadded<AtomicUsize>,
    read_pos: CachePadded<AtomicUsize>,
    mask: usize,
}

impl<T> RingBuffer<T> {
    /// Create a new ring buffer with specified capacity (must be power of 2)
    pub fn new(capacity: usize) -> Result<Self> {
        if !capacity.is_power_of_two() {
            return Err(LoggingError::BufferError(
                "Capacity must be power of 2".to_string()
            ));
        }
        
        if capacity == 0 {
            return Err(LoggingError::BufferError(
                "Capacity must be greater than 0".to_string()
            ));
        }
        
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(RwLock::new(None));
        }
        
        Ok(Self {
            buffer,
            capacity,
            write_pos: CachePadded::new(AtomicUsize::new(0)),
            read_pos: CachePadded::new(AtomicUsize::new(0)),
            mask: capacity - 1,
        })
    }
    
    /// Try to write an item to the buffer (non-blocking)
    #[inline]
    pub fn try_write(&self, item: T) -> Result<()> {
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        let read_pos = self.read_pos.load(Ordering::Acquire);
        
        // Check if buffer is full
        if write_pos.wrapping_sub(read_pos) >= self.capacity {
            return Err(LoggingError::BufferError("Buffer full".to_string()));
        }
        
        let index = write_pos & self.mask;
        
        // Try to acquire write lock
        if let Some(mut slot) = self.buffer[index].try_write() {
            if slot.is_none() {
                *slot = Some(item);
                self.write_pos.store(write_pos.wrapping_add(1), Ordering::Release);
                return Ok(());
            }
        }
        
        Err(LoggingError::BufferError("Slot occupied".to_string()))
    }
    
    /// Try to read an item from the buffer (non-blocking)
    #[inline]
    pub fn try_read(&self) -> Option<T> {
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        let write_pos = self.write_pos.load(Ordering::Acquire);
        
        // Check if buffer is empty
        if read_pos == write_pos {
            return None;
        }
        
        let index = read_pos & self.mask;
        
        // Try to acquire read lock
        if let Some(mut slot) = self.buffer[index].try_write() {
            if let Some(item) = slot.take() {
                self.read_pos.store(read_pos.wrapping_add(1), Ordering::Release);
                return Some(item);
            }
        }
        
        None
    }
    
    /// Get the number of items currently in the buffer
    pub fn len(&self) -> usize {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Acquire);
        write_pos.wrapping_sub(read_pos)
    }
    
    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Check if the buffer is full
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity
    }
    
    /// Get the capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Clear all items from the buffer
    pub fn clear(&self) {
        while self.try_read().is_some() {}
    }
    
    /// Get buffer utilization as a percentage
    pub fn utilization(&self) -> f64 {
        (self.len() as f64 / self.capacity as f64) * 100.0
    }
}

unsafe impl<T: Send> Send for RingBuffer<T> {}
unsafe impl<T: Send> Sync for RingBuffer<T> {}

/// Multi-producer, single-consumer ring buffer for high-throughput scenarios
pub struct MpscRingBuffer<T> {
    buffer: Vec<RwLock<Option<T>>>,
    capacity: usize,
    write_pos: CachePadded<AtomicUsize>,
    read_pos: CachePadded<AtomicUsize>,
    mask: usize,
}

impl<T> MpscRingBuffer<T> {
    pub fn new(capacity: usize) -> Result<Self> {
        if !capacity.is_power_of_two() {
            return Err(LoggingError::BufferError(
                "Capacity must be power of 2".to_string()
            ));
        }
        
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(RwLock::new(None));
        }
        
        Ok(Self {
            buffer,
            capacity,
            write_pos: CachePadded::new(AtomicUsize::new(0)),
            read_pos: CachePadded::new(AtomicUsize::new(0)),
            mask: capacity - 1,
        })
    }
    
    /// Try to write an item (lock-free for multiple producers)
    pub fn try_write(&self, item: T) -> Result<()> {
        loop {
            let write_pos = self.write_pos.load(Ordering::Relaxed);
            let read_pos = self.read_pos.load(Ordering::Acquire);
            
            // Check if buffer is full
            if write_pos.wrapping_sub(read_pos) >= self.capacity {
                return Err(LoggingError::BufferError("Buffer full".to_string()));
            }
            
            // Try to claim the next write position
            let next_write_pos = write_pos.wrapping_add(1);
            if self.write_pos.compare_exchange_weak(
                write_pos,
                next_write_pos,
                Ordering::Relaxed,
                Ordering::Relaxed
            ).is_ok() {
                let index = write_pos & self.mask;
                
                // Write the item
                loop {
                    if let Some(mut slot) = self.buffer[index].try_write() {
                        if slot.is_none() {
                            *slot = Some(item);
                            return Ok(());
                        }
                    }
                    std::hint::spin_loop();
                }
            }
            
            std::hint::spin_loop();
        }
    }
    
    /// Try to read an item (single consumer)
    pub fn try_read(&self) -> Option<T> {
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        let write_pos = self.write_pos.load(Ordering::Acquire);
        
        if read_pos == write_pos {
            return None;
        }
        
        let index = read_pos & self.mask;
        
        if let Some(mut slot) = self.buffer[index].try_write() {
            if let Some(item) = slot.take() {
                self.read_pos.store(read_pos.wrapping_add(1), Ordering::Release);
                return Some(item);
            }
        }
        
        None
    }
    
    pub fn len(&self) -> usize {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Acquire);
        write_pos.wrapping_sub(read_pos)
    }
    
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

unsafe impl<T: Send> Send for MpscRingBuffer<T> {}
unsafe impl<T: Send> Sync for MpscRingBuffer<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    use std::time::Instant;

    #[test]
    fn test_ring_buffer_basic_operations() {
        let buffer = RingBuffer::new(1024).unwrap();
        
        // Test write and read
        buffer.try_write(42).unwrap();
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.try_read(), Some(42));
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_ring_buffer_wraparound() {
        let buffer = RingBuffer::new(4).unwrap();
        
        // Fill buffer
        for i in 0..4 {
            buffer.try_write(i).unwrap();
        }
        assert!(buffer.is_full());
        
        // Should fail to write when full
        assert!(buffer.try_write(4).is_err());
        
        // Read and write to test wraparound
        assert_eq!(buffer.try_read(), Some(0));
        buffer.try_write(4).unwrap();
        
        // Verify order
        assert_eq!(buffer.try_read(), Some(1));
        assert_eq!(buffer.try_read(), Some(2));
        assert_eq!(buffer.try_read(), Some(3));
        assert_eq!(buffer.try_read(), Some(4));
    }

    #[test]
    fn test_mpsc_ring_buffer_concurrent() {
        let buffer = Arc::new(MpscRingBuffer::new(1024).unwrap());
        let buffer_clone = buffer.clone();
        
        // Spawn producer threads
        let producers: Vec<_> = (0..4).map(|producer_id| {
            let buffer = buffer.clone();
            thread::spawn(move || {
                for i in 0..1000 {
                    let value = producer_id * 1000 + i;
                    while buffer.try_write(value).is_err() {
                        thread::yield_now();
                    }
                }
            })
        }).collect();
        
        // Consumer thread
        let consumer = thread::spawn(move || {
            let mut received = Vec::new();
            let target_count = 4 * 1000; // 4 producers * 1000 items each
            
            while received.len() < target_count {
                if let Some(value) = buffer_clone.try_read() {
                    received.push(value);
                } else {
                    thread::yield_now();
                }
            }
            
            received
        });
        
        // Wait for producers
        for producer in producers {
            producer.join().unwrap();
        }
        
        // Wait for consumer
        let received = consumer.join().unwrap();
        assert_eq!(received.len(), 4000);
    }

    #[test]
    fn test_performance() {
        let buffer = RingBuffer::new(1024 * 1024).unwrap(); // 1M entries
        let count = 100_000;
        
        let start = Instant::now();
        
        // Write performance test
        for i in 0..count {
            buffer.try_write(i).unwrap();
        }
        
        let write_duration = start.elapsed();
        
        let start = Instant::now();
        
        // Read performance test
        for _ in 0..count {
            buffer.try_read().unwrap();
        }
        
        let read_duration = start.elapsed();
        
        println!("Write throughput: {:.0} ops/sec", 
                 count as f64 / write_duration.as_secs_f64());
        println!("Read throughput: {:.0} ops/sec", 
                 count as f64 / read_duration.as_secs_f64());
        
        // Should achieve >1M ops/sec
        assert!(count as f64 / write_duration.as_secs_f64() > 1_000_000.0);
        assert!(count as f64 / read_duration.as_secs_f64() > 1_000_000.0);
    }
}
