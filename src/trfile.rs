use std::fs::File;

use tr::TR;

pub struct TRFile {
    src: csv::Reader<File>,
}

impl TRFile {
    pub fn reader(file: File) -> csv::Reader<File> {
        csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b' ')
            .from_reader(file)
    }
    pub fn open(file: File) -> TRFile {
        TRFile {
            src: Self::reader(file),
        }
    }

    pub fn iter<'a>(&'a mut self) -> impl Iterator<Item = Result<TR, csv::Error>> + 'a {
        self.src.deserialize()
    }
}
