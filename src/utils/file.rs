use std::{
    fs::File,
    io::{self, Seek},
};

pub trait Empty {
    fn empty(&mut self) -> io::Result<()>;
}

impl Empty for File {
    fn empty(&mut self) -> io::Result<()> {
        self.seek(io::SeekFrom::Start(0))?;
        self.set_len(0)?;
        Ok(())
    }
}
