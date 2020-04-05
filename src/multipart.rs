// https://www.w3.org/TR/html401/interact/forms.html#h-17.13.4.2
use std::path::Path;

use tokio::fs::File;
use tokio::prelude::*;

pub async fn get_file_as_parts(
    key: &str,
    field: &str,
    file: &str,
) -> anyhow::Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let path = Path::new(file);
    let file_name = match path.file_name() {
        Some(file_name) => match file_name.to_str() {
            Some(name) => name,
            None => return Err(anyhow::anyhow!("Error No File found")),
        },
        None => return Err(anyhow::anyhow!("Error No File found")),
    };
    let guess = mime_guess::from_path(path);
    let t = match guess.first() {
        Some(m) => m,
        None => mime::TEXT_PLAIN,
    };
    let mut file = File::open(file).await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    let start=format!("--{}\r\nContent-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\nContent-Type: {}\r\n\r\n",key,field,file_name,t).bytes().collect::<Vec<u8>>();
    let end = format!("\r\n--{}--\r\n", key).bytes().collect::<Vec<u8>>();
    Ok((start, contents, end))
}
