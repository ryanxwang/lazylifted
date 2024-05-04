use std::{
    io::{Read, Write},
    path::Path,
};

use tempfile::NamedTempFile;

pub const RON_FILE_NAME: &str = "model.ron";
pub const PICKLE_FILE_NAME: &str = "model.pkl";

pub fn zip_files(target: &Path, files: Vec<(&str, &Path)>) {
    let file = std::fs::File::create(target).unwrap();
    let mut zip = zip::ZipWriter::new(file);

    for (file_name, file) in files {
        zip.start_file(
            file_name,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored)
                .unix_permissions(0o755),
        )
        .unwrap();
        let data = std::fs::read(file).unwrap();
        zip.write_all(&data).unwrap();
    }

    zip.finish().unwrap();
}

pub fn extract_from_zip(zip_path: &Path, file_name: &str) -> NamedTempFile {
    let file = std::fs::File::open(zip_path).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();

    let mut buffer = Vec::new();
    zip.by_name(file_name)
        .unwrap()
        .read_to_end(buffer.as_mut())
        .unwrap();

    let tempfile = NamedTempFile::new().unwrap();
    std::fs::write(tempfile.path(), buffer).unwrap();

    tempfile
}
