use anyhow::Result;
use pdf_extract::extract_text;
use std::path::Path;
use tracing;

/// Extract text content from a PDF file
pub fn extract_text_from_pdf<P: AsRef<Path>>(file_path: P) -> Result<String> {
    let path = file_path.as_ref();
    tracing::info!("Extracting text from PDF: {:?}", path);
    
    let text = extract_text(path)?;
    
    if text.trim().is_empty() {
        tracing::warn!("PDF file appears to be empty or contains no extractable text");
        return Ok(String::new());
    }
    
    tracing::info!("Successfully extracted {} characters from PDF", text.len());
    Ok(text)
}

/// Split text into chunks for embedding processing
pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();
    
    if words.is_empty() {
        return chunks;
    }
    
    let mut start = 0;
    
    while start < words.len() {
        let end = (start + chunk_size).min(words.len());
        let chunk_words = &words[start..end];
        let chunk_text = chunk_words.join(" ");
        
        if !chunk_text.trim().is_empty() {
            chunks.push(chunk_text);
        }
        
        // Move start position with overlap
        if end >= words.len() {
            break;
        }
        start = end.saturating_sub(overlap);
    }
    
    tracing::info!("Split text into {} chunks", chunks.len());
    chunks
}

/// Process PDF file and return chunked text
pub fn process_pdf_file<P: AsRef<Path>>(
    file_path: P, 
    chunk_size: usize, 
    overlap: usize
) -> Result<Vec<String>> {
    let text = extract_text_from_pdf(file_path)?;
    let chunks = chunk_text(&text, chunk_size, overlap);
    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text() {
        let text = "This is a test sentence with multiple words to test chunking functionality.";
        let chunks = chunk_text(text, 5, 2);
        
        assert!(!chunks.is_empty());
        assert!(chunks.len() > 1);
        
        // First chunk should have 5 words
        let first_chunk_words: Vec<&str> = chunks[0].split_whitespace().collect();
        assert_eq!(first_chunk_words.len(), 5);
    }

    #[test]
    fn test_empty_text() {
        let chunks = chunk_text("", 10, 2);
        assert!(chunks.is_empty());
    }
}
