use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    os::unix::fs::FileExt,
};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    NotFound,
}

pub struct Database {
    path: String,
    file: File,
    index: HashMap<String, u64>,
}

impl Database {
    pub fn open(file_path: &str) -> Result<Self, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&file_path)
            .map_err(|e| Error::Io(e))?;

        let mut db = Database {
            path: file_path.to_string(),
            file,
            index: HashMap::new(),
        };
        db.load_index()?;

        Ok(db)
    }

    fn load_index(&mut self) -> Result<(), Error> {
        let mut offset = 0;
        let mut key = String::new();
        let mut header_bytes = [0u8; Header::SIZE];
        loop {
            match self.file.read_at(&mut header_bytes, offset) {
                Ok(0) => break,
                Ok(_) => {
                    let header = Header::from_bytes(header_bytes);
                    let key_size = header.key_size as usize;
                    let value_size = header.value_size as usize;
                    let entry_size = Header::SIZE + key_size + value_size;
                    let mut entry_bytes = vec![0u8; entry_size];
                    self.file.read_at(&mut entry_bytes, offset);
                    let entry = Entry::from_bytes(entry_bytes);
                    self.index.insert(entry.key, offset);
                    offset += entry_size as u64;
                }
                Err(e) => return Err(Error::Io(e)),
            }
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<String, std::io::Error> {
        match self.index.get(key) {
            Some(offset) => {
                // Read header from file
                let mut header_bytes = [0u8; Header::SIZE];
                self.file.read_at(&mut header_bytes, *offset)?;
                let header = Header::from_bytes(header_bytes);

                // Read value bytes
                let mut value_bytes = vec![0u8; header.value_size as usize];
                self.file.read_at(
                    &mut value_bytes,
                    offset + header.key_size as u64 + Header::SIZE as u64,
                )?;
                String::from_utf8(value_bytes).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Failed to convert value bytes to string",
                    )
                })
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Key not found",
            )),
        }
    }

    pub fn put(&mut self, key: &str, value: &str) -> Result<(), Error> {
        // Create entry
        let header = Header {
            checksum: 0,
            timestamp: 0,
            is_deleted: false,
            key_size: key.len() as u32,
            value_size: value.len() as u32,
        };
        let entry = Entry {
            header,
            key: key.to_string(),
            value: value.to_string(),
        };
        let entry_bytes = entry.to_bytes();

        // Write entry to file
        let offset = self.file.metadata().map_err(|e| Error::Io(e))?.len();
        self.file
            .write_at(&entry_bytes, offset)
            .map_err(|e| Error::Io(e))?;

        // Update index
        self.index.insert(key.to_string(), offset);

        Ok(())
    }

    pub fn delete(&mut self, key: &str) -> Result<(), Error> {
        match self.index.get(key) {
            Some(offset) => {
                // Read header from file
                let mut header_bytes = [0u8; Header::SIZE];
                self.file
                    .read_at(&mut header_bytes, *offset)
                    .map_err(|e| Error::Io(e))?;
                let mut header = Header::from_bytes(header_bytes);

                // Update header
                header.is_deleted = true;
                let header_bytes = header.to_bytes();

                // Write header to file
                self.file
                    .write_at(&header_bytes, *offset)
                    .map_err(|e| Error::Io(e))?;

                self.index.remove(key);

                Ok(())
            }
            None => Err(Error::NotFound),
        }
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.file.sync_all().map_err(|e| Error::Io(e))?;
        Ok(())
    }
}

struct Header {
    checksum: u32,  // CRC32 of key and value
    timestamp: u32, // Unix timestamp
    is_deleted: bool,
    key_size: u32,
    value_size: u32,
}

impl Header {
    const SIZE: usize = std::mem::size_of::<Header>();

    fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];

        // Convert fields to bytes (little-endian)
        bytes[0..4].copy_from_slice(&self.checksum.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.timestamp.to_le_bytes());
        bytes[8] = self.is_deleted as u8;
        bytes[9..13].copy_from_slice(&self.key_size.to_le_bytes());
        bytes[13..17].copy_from_slice(&self.value_size.to_le_bytes());

        bytes
    }

    fn from_bytes(bytes: [u8; Self::SIZE]) -> Self {
        Header {
            checksum: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            timestamp: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            is_deleted: bytes[8] != 0,
            key_size: u32::from_le_bytes(bytes[9..13].try_into().unwrap()),
            value_size: u32::from_le_bytes(bytes[13..17].try_into().unwrap()),
        }
    }
}

struct Entry {
    header: Header,
    key: String,
    value: String,
}

impl Entry {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.extend_from_slice(self.key.as_bytes());
        bytes.extend_from_slice(self.value.as_bytes());
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        let header = Header::from_bytes(bytes[0..Header::SIZE].try_into().unwrap());
        let key = String::from_utf8(
            bytes[Header::SIZE..Header::SIZE + header.key_size as usize].to_vec(),
        )
        .unwrap();
        let value =
            String::from_utf8(bytes[Header::SIZE + header.key_size as usize..].to_vec()).unwrap();
        Entry { header, key, value }
    }
}
