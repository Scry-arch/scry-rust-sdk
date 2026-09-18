//! Turning an ELF executable into the image a board's loader writes to memory.
//!
//! The image is one contiguous block: every loadable segment's file bytes at its address, gaps
//! between segments filled with zeros. Zero-initialised data (the part of a segment that has no
//! file bytes) is not part of the block; the loader is expected to clear the memory after it.

use crate::board::Board;
use object::Object as _;
use object::ObjectSegment as _;

#[derive(Debug, PartialEq, Eq)]
pub struct Image {
    /// The address of the first byte of `bytes`.
    pub load_address: u64,

    /// The address execution starts at.
    pub entry: u64,

    pub bytes: Vec<u8>,

    /// The end of the memory the program occupies, including zero-initialised data.
    pub memory_end: u64,
}

impl Image {
    /// The address just past the last byte of `bytes`.
    pub fn file_end(&self) -> u64 {
        self.load_address + self.bytes.len() as u64
    }

    pub fn from_elf(elf: &[u8]) -> Result<Image, String> {
        let file = object::File::parse(elf).map_err(|error| format!("not a valid ELF: {error}"))?;

        // (address, file bytes, size in memory)
        let mut segments = Vec::new();
        for segment in file.segments() {
            let data = segment
                .data()
                .map_err(|error| format!("cannot read a loadable segment: {error}"))?;
            if segment.size() == 0 {
                continue;
            }
            segments.push((segment.address(), data, segment.size()));
        }
        if segments.is_empty() {
            return Err("the ELF has no loadable segments".to_owned());
        }
        segments.sort_by_key(|&(address, ..)| address);

        for pair in segments.windows(2) {
            let (first_address, first_data, _) = pair[0];
            let (second_address, ..) = pair[1];
            if first_address + first_data.len() as u64 > second_address {
                return Err(format!(
                    "loadable segments overlap at {second_address:#x}; cannot build a flat image"
                ));
            }
        }

        let load_address = segments[0].0;
        let file_end = segments
            .iter()
            .map(|&(address, data, _)| address + data.len() as u64)
            .max()
            .unwrap_or(load_address);
        let memory_end = segments
            .iter()
            .map(|&(address, _, size)| address + size)
            .max()
            .unwrap_or(load_address);

        let mut bytes = vec![0; (file_end - load_address) as usize];
        for &(address, data, _) in &segments {
            let offset = (address - load_address) as usize;
            bytes[offset..offset + data.len()].copy_from_slice(data);
        }

        Ok(Image {
            load_address,
            entry: file.entry(),
            bytes,
            memory_end,
        })
    }

    /// Checks that the image can run on `board`.
    pub fn check_against(&self, board: &Board) -> Result<(), String> {
        let program = board.memory.program;
        if !program.contains_range(self.load_address, self.memory_end) {
            return Err(format!(
                "the program occupies {:#x}..{:#x}, which is outside the memory {} provides for \
                 programs, {:#x}..{:#x}",
                self.load_address, self.memory_end, board.name, program.start, program.end
            ));
        }

        if !(self.load_address..self.file_end()).contains(&self.entry) {
            return Err(format!(
                "the entry point {:#x} is outside the loaded image {:#x}..{:#x}",
                self.entry,
                self.load_address,
                self.file_end()
            ));
        }

        if let Some(max_payload) = board.loader.max_payload
            && self.bytes.len() as u64 > max_payload
        {
            return Err(format!(
                "the image is {} bytes, but the loader of {} accepts at most {max_payload}",
                self.bytes.len(),
                board.name
            ));
        }

        for (what, value) in [
            ("load address", self.load_address),
            ("entry point", self.entry),
            ("image size", self.bytes.len() as u64),
        ] {
            if u32::try_from(value).is_err() {
                return Err(format!("the {what} {value:#x} does not fit in 32 bits"));
            }
        }

        Ok(())
    }

    /// Checks that the image holds exactly the ELF's segment bytes at the right offsets. Guards
    /// the flattening against padding and ordering mistakes.
    pub fn verify_against(&self, elf: &[u8]) -> Result<(), String> {
        let file = object::File::parse(elf).map_err(|error| format!("not a valid ELF: {error}"))?;
        for segment in file.segments() {
            let data = segment
                .data()
                .map_err(|error| format!("cannot read a loadable segment: {error}"))?;
            if segment.size() == 0 {
                continue;
            }
            let offset = (segment.address() - self.load_address) as usize;
            if self.bytes.get(offset..offset + data.len()) != Some(data) {
                return Err(format!(
                    "the image does not match the ELF segment at {:#x}",
                    segment.address()
                ));
            }
        }
        Ok(())
    }

    pub fn summary(&self) -> String {
        format!(
            "load address {:#x}, entry {:#x}, {} bytes, memory up to {:#x}",
            self.load_address,
            self.entry,
            self.bytes.len(),
            self.memory_end
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal ELF32 executable by hand: a header, program headers, and segment bytes.
    /// `segments` are (address, file bytes, size in memory).
    fn elf32(entry: u32, segments: &[(u32, &[u8], u32)]) -> Vec<u8> {
        const EHDR: usize = 52;
        const PHDR: usize = 32;
        let headers = EHDR + PHDR * segments.len();

        let mut elf = Vec::new();
        elf.extend_from_slice(&[0x7f, b'E', b'L', b'F', 1, 1, 1, 0]);
        elf.extend_from_slice(&[0; 8]);
        elf.extend_from_slice(&2u16.to_le_bytes()); // ET_EXEC
        elf.extend_from_slice(&264u16.to_le_bytes()); // EM_SCRY
        elf.extend_from_slice(&1u32.to_le_bytes());
        elf.extend_from_slice(&entry.to_le_bytes());
        elf.extend_from_slice(&(EHDR as u32).to_le_bytes()); // e_phoff
        elf.extend_from_slice(&0u32.to_le_bytes()); // e_shoff
        elf.extend_from_slice(&0u32.to_le_bytes()); // e_flags
        elf.extend_from_slice(&(EHDR as u16).to_le_bytes());
        elf.extend_from_slice(&(PHDR as u16).to_le_bytes());
        elf.extend_from_slice(&(segments.len() as u16).to_le_bytes());
        elf.extend_from_slice(&40u16.to_le_bytes()); // e_shentsize
        elf.extend_from_slice(&0u16.to_le_bytes()); // e_shnum
        elf.extend_from_slice(&0u16.to_le_bytes()); // e_shstrndx
        assert_eq!(elf.len(), EHDR);

        let mut offset = headers;
        for &(address, data, memory_size) in segments {
            for field in [
                1, // PT_LOAD
                offset as u32,
                address,
                address,
                data.len() as u32,
                memory_size,
                7, // flags
                1, // align
            ] {
                elf.extend_from_slice(&field.to_le_bytes());
            }
            offset += data.len();
        }
        for &(_, data, _) in segments {
            elf.extend_from_slice(data);
        }
        elf
    }

    fn board(start: u64, end: u64, max_payload: Option<u64>) -> Board {
        let mut text = format!(
            "name = \"Test board\"\n[memory]\nprogram = {{ start = {start}, end = {end} }}\n"
        );
        if let Some(max_payload) = max_payload {
            text.push_str(&format!("[loader]\nmax-payload = {max_payload}\n"));
        }
        Board::parse(&text).unwrap()
    }

    #[test]
    fn flattens_segments_with_gaps_and_bss() {
        let elf = elf32(
            0x104,
            &[
                (0x100, &[1, 2, 3, 4, 5, 6], 6),
                // Two bytes of gap, then data followed by 8 bytes of zero-initialised memory.
                (0x108, &[9, 8], 10),
            ],
        );
        let image = Image::from_elf(&elf).unwrap();

        assert_eq!(image.load_address, 0x100);
        assert_eq!(image.entry, 0x104);
        assert_eq!(image.bytes, [1, 2, 3, 4, 5, 6, 0, 0, 9, 8]);
        assert_eq!(image.file_end(), 0x10a);
        assert_eq!(image.memory_end, 0x112);
        image.verify_against(&elf).unwrap();
    }

    #[test]
    fn segment_order_in_the_file_does_not_matter() {
        let elf = elf32(0x200, &[(0x208, &[7, 7], 2), (0x200, &[1, 2, 3, 4], 4)]);
        let image = Image::from_elf(&elf).unwrap();
        assert_eq!(image.load_address, 0x200);
        assert_eq!(image.bytes, [1, 2, 3, 4, 0, 0, 0, 0, 7, 7]);
        image.verify_against(&elf).unwrap();
    }

    #[test]
    fn rejects_overlapping_segments_and_empty_files() {
        let overlapping = elf32(0x100, &[(0x100, &[1, 2, 3, 4], 4), (0x102, &[5, 6], 2)]);
        let error = Image::from_elf(&overlapping).unwrap_err();
        assert!(error.contains("overlap"), "{error}");

        let error = Image::from_elf(&elf32(0, &[])).unwrap_err();
        assert!(error.contains("no loadable segments"), "{error}");
    }

    #[test]
    fn verification_catches_a_wrong_image() {
        let elf = elf32(0x100, &[(0x100, &[1, 2, 3, 4], 4)]);
        let mut image = Image::from_elf(&elf).unwrap();
        image.bytes[2] ^= 0xff;
        assert!(image.verify_against(&elf).is_err());
    }

    #[test]
    fn checks_the_program_region() {
        let elf = elf32(0x100, &[(0x100, &[1, 2, 3, 4], 0x40)]);
        let image = Image::from_elf(&elf).unwrap();

        image.check_against(&board(0x100, 0x140, None)).unwrap();

        // The zero-initialised tail counts: it needs memory too.
        let error = image.check_against(&board(0x100, 0x13f, None)).unwrap_err();
        assert!(error.contains("outside the memory"), "{error}");

        let error = image.check_against(&board(0x104, 0x200, None)).unwrap_err();
        assert!(error.contains("outside the memory"), "{error}");
    }

    #[test]
    fn checks_the_entry_point_and_the_payload_limit() {
        let outside = elf32(0x300, &[(0x100, &[1, 2, 3, 4], 4)]);
        let image = Image::from_elf(&outside).unwrap();
        let error = image.check_against(&board(0, 0x1000, None)).unwrap_err();
        assert!(error.contains("entry point"), "{error}");

        let elf = elf32(0x100, &[(0x100, &[1, 2, 3, 4], 4)]);
        let image = Image::from_elf(&elf).unwrap();
        image.check_against(&board(0, 0x1000, Some(4))).unwrap();
        let error = image.check_against(&board(0, 0x1000, Some(3))).unwrap_err();
        assert!(error.contains("accepts at most 3"), "{error}");
    }
}
