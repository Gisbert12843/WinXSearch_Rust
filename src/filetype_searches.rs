use anyhow::{anyhow, Result};
use pdf_extract::extract_text;
use std::sync::{Arc, Mutex};

pub fn search_pdf_file(
    file: &walkdir::DirEntry,
    vec_search_value: Arc<Mutex<Vec<String>>>,
) -> Result<bool>
{
    match extract_text(&file.path())
    {
        Ok(text) =>
        {
            let mut found = false;
            let vec_search_value = vec_search_value.lock().unwrap();

            for search_value in vec_search_value.iter()
            {
                if text.contains(search_value)
                {
                    found = true;
                    break;
                }
            }
            Ok(found)
        }
        Err(e) =>
        {
            // Convert the error to a string and wrap it in `anyhow!`
            Err(anyhow!("Failed to extract text: {:?}", e))
        }
    }
}
