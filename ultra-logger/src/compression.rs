//! Compression utilities for log data

use crate::error::{LoggingError, Result};
use std::io::{Write, Read};

#[derive(Debug, Clone)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
    Lz4,
    Snappy,
}

impl CompressionType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "none" => Ok(CompressionType::None),
            "gzip" => Ok(CompressionType::Gzip),
            "zstd" => Ok(CompressionType::Zstd),
            "lz4" => Ok(CompressionType::Lz4),
            "snappy" => Ok(CompressionType::Snappy),
            _ => Err(LoggingError::CompressionError(
                format!("Unknown compression type: {}", s)
            )),
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            CompressionType::None => "none",
            CompressionType::Gzip => "gzip",
            CompressionType::Zstd => "zstd",
            CompressionType::Lz4 => "lz4",
            CompressionType::Snappy => "snappy",
        }
    }
}

pub trait Compressor: Send + Sync {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn compression_type(&self) -> CompressionType;
    fn estimated_compression_ratio(&self) -> f64;
}

pub fn create_compressor(compression_type: CompressionType) -> Result<Box<dyn Compressor>> {
    match compression_type {
        CompressionType::None => Ok(Box::new(NoCompressor)),
        CompressionType::Gzip => Ok(Box::new(GzipCompressor::new())),
        CompressionType::Zstd => Ok(Box::new(ZstdCompressor::new()?)),
        CompressionType::Lz4 => Ok(Box::new(Lz4Compressor::new())),
        CompressionType::Snappy => Ok(Box::new(SnappyCompressor::new())),
    }
}

// No Compression
pub struct NoCompressor;

impl Compressor for NoCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }
    
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }
    
    fn compression_type(&self) -> CompressionType {
        CompressionType::None
    }
    
    fn estimated_compression_ratio(&self) -> f64 {
        1.0 // No compression
    }
}

// GZIP Compression
pub struct GzipCompressor;

impl GzipCompressor {
    pub fn new() -> Self {
        Self
    }
}

impl Compressor for GzipCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(data)
            .map_err(LoggingError::IoError)?;
        encoder.finish()
            .map_err(LoggingError::IoError)
    }
    
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::GzDecoder;
        
        let mut decoder = GzDecoder::new(data);
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)
            .map_err(LoggingError::IoError)?;
        Ok(result)
    }
    
    fn compression_type(&self) -> CompressionType {
        CompressionType::Gzip
    }
    
    fn estimated_compression_ratio(&self) -> f64 {
        0.3 // Typical 70% compression for JSON logs
    }
}

// ZSTD Compression
pub struct ZstdCompressor {
    level: i32,
}

impl ZstdCompressor {
    pub fn new() -> Result<Self> {
        Ok(Self { level: 3 }) // Default compression level
    }
    
    pub fn with_level(level: i32) -> Result<Self> {
        Ok(Self { level })
    }
}

impl Compressor for ZstdCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        zstd::bulk::compress(data, self.level)
            .map_err(|e| LoggingError::CompressionError(e.to_string()))
    }
    
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        zstd::bulk::decompress(data, data.len() * 4) // Estimate decompressed size
            .map_err(|e| LoggingError::CompressionError(e.to_string()))
    }
    
    fn compression_type(&self) -> CompressionType {
        CompressionType::Zstd
    }
    
    fn estimated_compression_ratio(&self) -> f64 {
        0.25 // Zstd typically achieves better compression than gzip
    }
}

// LZ4 Compression (Fast)
pub struct Lz4Compressor;

impl Lz4Compressor {
    pub fn new() -> Self {
        Self
    }
}

impl Compressor for Lz4Compressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        lz4_flex::compress_prepend_size(data)
            .into_iter()
            .collect::<Vec<u8>>()
            .pipe(Ok)
    }
    
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        lz4_flex::decompress_size_prepended(data)
            .map_err(|e| LoggingError::CompressionError(e.to_string()))
    }
    
    fn compression_type(&self) -> CompressionType {
        CompressionType::Lz4
    }
    
    fn estimated_compression_ratio(&self) -> f64 {
        0.5 // LZ4 prioritizes speed over compression ratio
    }
}

// Snappy Compression (Google)
pub struct SnappyCompressor;

impl SnappyCompressor {
    pub fn new() -> Self {
        Self
    }
}

impl Compressor for SnappyCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = snap::write::FrameEncoder::new(Vec::new());
        encoder.write_all(data)
            .map_err(LoggingError::IoError)?;
        encoder.into_inner()
            .map_err(LoggingError::IoError)
    }
    
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = snap::read::FrameDecoder::new(data);
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)
            .map_err(LoggingError::IoError)?;
        Ok(result)
    }
    
    fn compression_type(&self) -> CompressionType {
        CompressionType::Snappy
    }
    
    fn estimated_compression_ratio(&self) -> f64 {
        0.4 // Good balance of speed and compression
    }
}

// Batch compression for multiple log entries
pub struct BatchCompressor {
    compressor: Box<dyn Compressor>,
    buffer: Vec<u8>,
    max_batch_size: usize,
}

impl BatchCompressor {
    pub fn new(
        compression_type: CompressionType, 
        max_batch_size: usize
    ) -> Result<Self> {
        Ok(Self {
            compressor: create_compressor(compression_type)?,
            buffer: Vec::with_capacity(max_batch_size),
            max_batch_size,
        })
    }
    
    pub fn add_entry(&mut self, data: &[u8]) -> Result<Option<Vec<u8>>> {
        if self.buffer.len() + data.len() > self.max_batch_size {
            let compressed = self.flush()?;
            return Ok(Some(compressed));
        }
        
        self.buffer.extend_from_slice(data);
        self.buffer.push(b'\n'); // Line separator
        Ok(None)
    }
    
    pub fn flush(&mut self) -> Result<Vec<u8>> {
        if self.buffer.is_empty() {
            return Ok(Vec::new());
        }
        
        let compressed = self.compressor.compress(&self.buffer)?;
        self.buffer.clear();
        Ok(compressed)
    }
    
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.max_batch_size
    }
    
    pub fn compression_ratio(&self) -> f64 {
        self.compressor.estimated_compression_ratio()
    }
}

// Extension trait for convenient pipe operations
trait Pipe<T> {
    fn pipe<R, F: FnOnce(T) -> R>(self, f: F) -> R;
}

impl<T> Pipe<T> for T {
    fn pipe<R, F: FnOnce(T) -> R>(self, f: F) -> R {
        f(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_no_compression() {
        let compressor = NoCompressor;
        let data = b"Hello, World!";
        
        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();
        
        assert_eq!(data, &compressed[..]);
        assert_eq!(data, &decompressed[..]);
    }
    
    #[test]
    fn test_gzip_compression() {
        let compressor = GzipCompressor::new();
        let data = b"Hello, World! This is a test message that should compress well.";
        
        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();
        
        assert!(compressed.len() < data.len()); // Should be smaller
        assert_eq!(data, &decompressed[..]);
    }
    
    #[test]
    fn test_batch_compressor() {
        let mut batch = BatchCompressor::new(CompressionType::Gzip, 1024).unwrap();
        
        let entry1 = b"First log entry";
        let entry2 = b"Second log entry";
        
        assert!(batch.add_entry(entry1).unwrap().is_none());
        assert!(batch.add_entry(entry2).unwrap().is_none());
        
        let compressed = batch.flush().unwrap();
        assert!(!compressed.is_empty());
    }
}
