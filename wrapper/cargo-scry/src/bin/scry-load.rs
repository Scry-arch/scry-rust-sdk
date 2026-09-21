//! `scry-load`: puts a Scry program onto a board.
//!
//! `cargo scry run --board <board>` uses this as the cargo runner, so it is invoked as
//! `scry-load --board-file <profile> <program> [options]`, with the options being whatever the
//! user put after `--` on the cargo command line.
//!
//! The program is an ELF executable, a textual `.scry` assembly file (assembled here), or — only
//! under the explicit `--raw` flag — a pre-flattened binary; assembled and raw programs load at
//! address 0 with entry 0. The image is checked against the board's memory, sent to the board's
//! loader over the serial port, and then the program's output is shown until it ends, when the
//! returned operands are printed the way scryer prints them.

use cargo_scry::board::Board;
use cargo_scry::image::Image;
use cargo_scry::loader;
use cargo_scry::loader::Ack;
use cargo_scry::loader::Event;
use cargo_scry::loader::ReportParser;
use scry_asm::Assemble as _;
use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;
use std::time::Instant;

const USAGE: &str = "\
usage: scry-load --board-file <profile.toml> <program> [options]

The program is an ELF executable, or a .scry assembly text file
(assembled here, loaded at address 0 with entry 0).

options:
  --raw            treat the program as a pre-flattened binary instead
                   (loaded at address 0, entry 0)
  --check          only check that the program fits the board, send nothing
  --image <file>   write the flat image to <file> instead of sending it
  --port <name>    serial port of the board (default: autodetect)
  --no-wait        exit once the program is launched, don't show its output";

struct Options {
    board_file: PathBuf,
    program: PathBuf,
    raw: bool,
    check_only: bool,
    image_file: Option<PathBuf>,
    port: Option<String>,
    no_wait: bool,
}

fn main() {
    let options = parse_args().unwrap_or_else(|message| {
        eprintln!("error: {message}\n\n{USAGE}");
        exit(2);
    });
    match run(&options) {
        Ok(code) => exit(code),
        Err(message) => {
            eprintln!("error: {message}");
            exit(1);
        }
    }
}

fn parse_args() -> Result<Options, String> {
    let mut board_file = None;
    let mut program = None;
    let mut raw = false;
    let mut check_only = false;
    let mut image_file = None;
    let mut port = None;
    let mut no_wait = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value"));
        match arg.as_str() {
            "--board-file" => board_file = Some(PathBuf::from(value("--board-file")?)),
            "--image" => image_file = Some(PathBuf::from(value("--image")?)),
            "--port" => port = Some(value("--port")?),
            "--raw" => raw = true,
            "--check" => check_only = true,
            "--no-wait" => no_wait = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                exit(0);
            }
            other if other.starts_with('-') => return Err(format!("unknown option {other}")),
            _ if program.is_none() => program = Some(PathBuf::from(arg)),
            other => return Err(format!("unexpected argument {other}")),
        }
    }

    Ok(Options {
        board_file: board_file.ok_or("no board profile given (--board-file)")?,
        program: program.ok_or("no program file given")?,
        raw,
        check_only,
        image_file,
        port,
        no_wait,
    })
}

fn run(options: &Options) -> Result<i32, String> {
    let board = Board::load(&options.board_file)?;
    let data = std::fs::read(&options.program)
        .map_err(|error| format!("cannot read {}: {error}", options.program.display()))?;

    let image = load_image(&options.program, data, options.raw)?;
    image.check_against(&board)?;
    eprintln!("{}: {}", board.name, image.summary());

    if let Some(image_file) = &options.image_file {
        std::fs::write(image_file, &image.bytes)
            .map_err(|error| format!("cannot write {}: {error}", image_file.display()))?;
        eprintln!("wrote {}", image_file.display());
        return Ok(0);
    }
    if options.check_only {
        return Ok(0);
    }

    send(&board, &image, options)
}

/// Builds the image from the program file: a pre-flattened binary under
/// `--raw`, textual assembly for a `.scry` file, and an ELF otherwise —
/// never decided by the file's contents.
fn load_image(path: &Path, data: Vec<u8>, raw: bool) -> Result<Image, String> {
    if raw {
        return Image::flat(data);
    }
    let is_scry = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("scry"));
    if is_scry {
        let text = String::from_utf8(data)
            .map_err(|_| format!("{} is not UTF-8 assembly text", path.display()))?;
        let bytes = scry_asm::Raw::assemble(std::iter::once(text.as_str()))
            .map_err(|error| format!("cannot assemble {}: {error}", path.display()))?;
        return Image::flat(bytes);
    }
    let image = Image::from_elf(&data)
        .map_err(|error| format!("{error} (a pre-flattened binary needs --raw)"))?;
    image.verify_against(&data)?;
    Ok(image)
}

fn send(board: &Board, image: &Image, options: &Options) -> Result<i32, String> {
    let port_name = match &options.port {
        Some(port) => port.clone(),
        None => detect_port()?,
    };
    eprintln!("using {port_name}");

    let mut port = serialport::new(&port_name, board.loader.baud)
        .timeout(Duration::from_millis(50))
        .open()
        .map_err(|error| format!("cannot open {port_name}: {error}"))?;

    // Drop whatever a previous program left in the receive buffer.
    std::thread::sleep(Duration::from_millis(100));
    port.clear(serialport::ClearBuffer::Input)
        .map_err(|error| format!("cannot clear {port_name}: {error}"))?;

    port.write_all(&loader::download_frame(image))
        .and_then(|()| port.flush())
        .map_err(|error| format!("cannot send to {port_name}: {error}"))?;

    match read_byte(&mut *port, Duration::from_secs(5))?.map(Ack::from) {
        Some(Ack::Launched) => eprintln!("program launched"),
        Some(Ack::BadChecksum) => return Err("the loader rejected the checksum".to_owned()),
        Some(Ack::TimedOut) => {
            return Err("the loader timed out in the middle of the frame".to_owned());
        }
        Some(Ack::Unknown(byte)) => {
            return Err(format!("unexpected reply {byte:#04x} from the loader"));
        }
        None => {
            return Err("no reply from the loader; is it armed? (reset the board)".to_owned());
        }
    }
    if options.no_wait {
        return Ok(0);
    }

    // Whatever the user types goes to the program. Lines, since stdin is line buffered.
    let (keys_sender, keys) = std::sync::mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut buffer = [0; 256];
        let mut stdin = std::io::stdin();
        while let Ok(count) = stdin.read(&mut buffer) {
            if count == 0 || keys_sender.send(buffer[..count].to_vec()).is_err() {
                break;
            }
        }
    });

    let mut parser = ReportParser::default();
    let mut stdout = std::io::stdout();
    loop {
        while let Ok(typed) = keys.try_recv() {
            port.write_all(&typed)
                .map_err(|error| format!("cannot send to {port_name}: {error}"))?;
        }

        let Some(byte) = read_byte(&mut *port, Duration::from_millis(50))? else {
            continue;
        };
        match parser.feed(byte) {
            Event::Output(bytes) => {
                if !bytes.is_empty() {
                    let _ = stdout.write_all(&bytes);
                    let _ = stdout.flush();
                }
            }
            Event::Done(report) => {
                println!();
                print!("{}", report.to_scryer_format());
                return Ok(i32::from(report.trapped));
            }
        }
    }
}

/// Reads one byte, or `None` if nothing arrived within `patience`.
fn read_byte(
    port: &mut dyn serialport::SerialPort,
    patience: Duration,
) -> Result<Option<u8>, String> {
    let deadline = Instant::now() + patience;
    let mut byte = [0];
    loop {
        match port.read(&mut byte) {
            Ok(1) => return Ok(Some(byte[0])),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {}
            Err(error) => return Err(format!("cannot read from the serial port: {error}")),
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
    }
}

/// Finds the board's serial port: the only USB serial port, or the only FTDI one.
fn detect_port() -> Result<String, String> {
    let ports = serialport::available_ports()
        .map_err(|error| format!("cannot list serial ports: {error}; pass --port"))?;

    let usb = ports
        .iter()
        .filter_map(|port| match &port.port_type {
            serialport::SerialPortType::UsbPort(info) => Some((port.port_name.clone(), info)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let ftdi = usb
        .iter()
        .filter(|(_, info)| {
            info.manufacturer
                .as_deref()
                .is_some_and(|name| name.to_ascii_uppercase().contains("FTDI"))
        })
        .collect::<Vec<_>>();

    match (ftdi.as_slice(), usb.as_slice()) {
        ([(name, _)], _) | ([], [(name, _)]) => Ok(name.clone()),
        ([], []) => Err("no USB serial port found; pass --port".to_owned()),
        _ => Err(format!(
            "several serial ports could be the board ({}); pass --port",
            usb.iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}
