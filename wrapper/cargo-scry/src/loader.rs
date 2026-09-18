//! The serial protocol of a Scry board's boot loader.
//!
//! Download frame, host to board. All integers are little-endian:
//!
//! | bytes | content |
//! |---|---|
//! | 8 | the magic `SCRYLD! `, with its trailing space |
//! | 4 | entry address: where execution starts |
//! | 4 | load address: where the first payload byte goes |
//! | 4 | payload length |
//! | n | payload |
//! | 1 | checksum: the sum of the payload bytes, modulo 256 |
//!
//! The loader writes the payload to memory, clears the rest of the program memory after it, and
//! answers with one byte: `K` launched, `E` bad checksum, `T` timed out mid-frame.
//!
//! This replaces the Scry-5 loader's original frame, which used the same magic but was followed
//! directly by a 16-bit length, carried neither address, and always started execution at address
//! 0. The two layouts are not compatible: a loader understands one or the other.
//!
//! Report frame, board to host, sent when the program ends: the magic `SCRYRT! `, a status byte
//! (0 = clean halt, anything else = trap), an operand count, then per returned operand a tag byte
//! and a 32-bit value. Bytes before the magic are the program's own output.

use crate::image::Image;

pub const DOWNLOAD_MAGIC: &[u8; 8] = b"SCRYLD! ";
pub const REPORT_MAGIC: &[u8; 8] = b"SCRYRT! ";

/// Builds the download frame for `image`. The image must have passed `Image::check_against`, which
/// guarantees that its addresses and size fit in 32 bits.
pub fn download_frame(image: &Image) -> Vec<u8> {
    let mut frame = Vec::with_capacity(image.bytes.len() + 21);
    frame.extend_from_slice(DOWNLOAD_MAGIC);
    frame.extend_from_slice(&(image.entry as u32).to_le_bytes());
    frame.extend_from_slice(&(image.load_address as u32).to_le_bytes());
    frame.extend_from_slice(&(image.bytes.len() as u32).to_le_bytes());
    frame.extend_from_slice(&image.bytes);
    frame.push(checksum(&image.bytes));
    frame
}

pub fn checksum(payload: &[u8]) -> u8 {
    payload
        .iter()
        .fold(0u8, |sum, byte| sum.wrapping_add(*byte))
}

/// The loader's answer to a download frame.
#[derive(Debug, PartialEq, Eq)]
pub enum Ack {
    Launched,
    BadChecksum,
    TimedOut,
    Unknown(u8),
}

impl From<u8> for Ack {
    fn from(byte: u8) -> Self {
        match byte {
            b'K' => Ack::Launched,
            b'E' => Ack::BadChecksum,
            b'T' => Ack::TimedOut,
            other => Ack::Unknown(other),
        }
    }
}

/// How a program ended.
#[derive(Debug, PartialEq, Eq)]
pub struct Report {
    pub trapped: bool,
    /// (tag, value) per returned operand.
    pub operands: Vec<(u8, u32)>,
}

impl Report {
    /// Formats the report the way scryer prints a finished program, so the two can be compared
    /// and checked by the same scripts.
    pub fn to_scryer_format(&self) -> String {
        if self.trapped {
            return "----------  Error  ----------\nErr(\"The program trapped\")\n".to_owned();
        }
        let mut text = "----------  Returned Operands  ----------\n".to_owned();
        for &(tag, value) in &self.operands {
            text.push_str(&format_operand(tag, value));
            text.push_str(", ");
        }
        text.push('\n');
        text
    }
}

/// Formats one returned operand. The tag's low two bits give the width as `8 << n` bits, 0x20
/// marks a signed value, 0x40 not-a-number and 0x80 not-a-result.
pub fn format_operand(tag: u8, value: u32) -> String {
    if tag & 0x80 != 0 {
        return "NaR".to_owned();
    }
    if tag & 0x40 != 0 {
        return "NaN".to_owned();
    }
    let width = 8u32 << (tag & 3);
    let mask = if width >= 32 {
        u64::from(u32::MAX)
    } else {
        (1u64 << width) - 1
    };
    let value = u64::from(value) & mask;
    if tag & 0x20 != 0 {
        let signed = if value >= 1u64 << (width - 1) {
            value as i64 - (1i64 << width)
        } else {
            value as i64
        };
        format!("{signed}i{width}")
    } else {
        format!("{value}u{width}")
    }
}

/// Splits the byte stream coming from a running program into the program's own output and the
/// loader's final report.
#[derive(Default)]
pub struct ReportParser {
    /// How much of the report magic has been matched.
    matched: usize,
    /// The report's bytes after the magic, once the magic has been seen.
    report: Option<Vec<u8>>,
}

/// What one received byte turned out to be.
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// Bytes of program output, to be shown to the user. May be empty.
    Output(Vec<u8>),
    /// The report is complete.
    Done(Report),
}

impl ReportParser {
    pub fn feed(&mut self, byte: u8) -> Event {
        if let Some(report) = &mut self.report {
            report.push(byte);
            // Status and count, then five bytes per operand.
            let needed = 2 + report.get(1).map_or(0, |&count| 5 * usize::from(count));
            if report.len() >= 2 && report.len() >= needed {
                let operands = report[2..]
                    .chunks_exact(5)
                    .map(|chunk| {
                        let value = u32::from_le_bytes([chunk[1], chunk[2], chunk[3], chunk[4]]);
                        (chunk[0], value)
                    })
                    .collect();
                return Event::Done(Report {
                    trapped: report[0] != 0,
                    operands,
                });
            }
            return Event::Output(Vec::new());
        }

        if byte == REPORT_MAGIC[self.matched] {
            self.matched += 1;
            if self.matched == REPORT_MAGIC.len() {
                self.matched = 0;
                self.report = Some(Vec::new());
            }
            return Event::Output(Vec::new());
        }

        // Not the magic after all: what was held back is program output.
        let mut output = REPORT_MAGIC[..self.matched].to_vec();
        self.matched = 0;
        if byte == REPORT_MAGIC[0] {
            self.matched = 1;
        } else {
            output.push(byte);
        }
        Event::Output(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_frame_layout() {
        let image = Image {
            load_address: 0x100,
            entry: 0x0123_4567,
            bytes: vec![0xff, 0x02, 0x10],
            memory_end: 0x200,
        };
        let frame = download_frame(&image);

        assert_eq!(&frame[..8], b"SCRYLD! ");
        assert_eq!(frame[8..12], [0x67, 0x45, 0x23, 0x01]);
        assert_eq!(frame[12..16], [0x00, 0x01, 0x00, 0x00]);
        assert_eq!(frame[16..20], [3, 0, 0, 0]);
        assert_eq!(frame[20..23], [0xff, 0x02, 0x10]);
        // 0xff + 0x02 + 0x10 = 0x111, modulo 256.
        assert_eq!(frame[23], 0x11);
        assert_eq!(frame.len(), 24);
    }

    #[test]
    fn acknowledgements() {
        assert_eq!(Ack::from(b'K'), Ack::Launched);
        assert_eq!(Ack::from(b'E'), Ack::BadChecksum);
        assert_eq!(Ack::from(b'T'), Ack::TimedOut);
        assert_eq!(Ack::from(b'?'), Ack::Unknown(b'?'));
    }

    #[test]
    fn operand_formatting() {
        assert_eq!(format_operand(0x00, 42), "42u8");
        assert_eq!(format_operand(0x02, 80), "80u32");
        assert_eq!(format_operand(0x02, u32::MAX), "4294967295u32");
        // Only the low `width` bits count.
        assert_eq!(format_operand(0x00, 0x1_2a), "42u8");
        assert_eq!(format_operand(0x20, 0xfc), "-4i8");
        assert_eq!(format_operand(0x21, 0x7fff), "32767i16");
        assert_eq!(format_operand(0x22, u32::MAX), "-1i32");
        assert_eq!(format_operand(0x80, 0), "NaR");
        assert_eq!(format_operand(0x40, 0), "NaN");
    }

    fn run(parser: &mut ReportParser, bytes: &[u8]) -> (Vec<u8>, Option<Report>) {
        let mut output = Vec::new();
        for &byte in bytes {
            match parser.feed(byte) {
                Event::Output(bytes) => output.extend(bytes),
                Event::Done(report) => return (output, Some(report)),
            }
        }
        (output, None)
    }

    #[test]
    fn separates_program_output_from_the_report() {
        let mut stream = b"Hello, world!\n".to_vec();
        stream.extend_from_slice(b"SCRYRT! ");
        stream.extend_from_slice(&[0, 2]);
        stream.extend_from_slice(&[0x00, 42, 0, 0, 0]);
        stream.extend_from_slice(&[0x22, 0xff, 0xff, 0xff, 0xff]);

        let (output, report) = run(&mut ReportParser::default(), &stream);
        assert_eq!(output, b"Hello, world!\n");
        let report = report.unwrap();
        assert!(!report.trapped);
        assert_eq!(report.operands, [(0x00, 42), (0x22, u32::MAX)]);
        assert_eq!(
            report.to_scryer_format(),
            "----------  Returned Operands  ----------\n42u8, -1i32, \n"
        );
    }

    #[test]
    fn a_partial_magic_in_the_output_is_not_lost() {
        let mut stream = b"SCRY is not SCRYRT yet SS".to_vec();
        stream.extend_from_slice(b"SCRYRT! ");
        stream.extend_from_slice(&[1, 0]);

        let (output, report) = run(&mut ReportParser::default(), &stream);
        assert_eq!(output, b"SCRY is not SCRYRT yet SS");
        let report = report.unwrap();
        assert!(report.trapped);
        assert!(report.operands.is_empty());
        assert!(report.to_scryer_format().contains("trapped"));
    }

    #[test]
    fn an_unfinished_report_is_not_reported() {
        let mut stream = b"SCRYRT! ".to_vec();
        stream.extend_from_slice(&[0, 1, 0x00, 42]);
        let (_, report) = run(&mut ReportParser::default(), &stream);
        assert!(report.is_none());
    }
}
