#![windows_subsystem = "console"]

extern crate hassle_rs;
extern crate structopt;

use hassle_rs::{DxcValidator, Dxil};
use std::fs::File;
use std::io::{BufWriter, Error, ErrorKind, Read, Result, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "dxil-val")]
struct Settings {
    /// Force re-validation if already signed
    #[structopt(short = "f", long = "force")]
    force: bool,

    /// Input file
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    input: PathBuf,

    /// Input file
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,
}

#[repr(C)]
pub struct MinimalHeader {
    four_cc: u32,
    hash_digest: [u32; 4],
}

const HEADER_SIZE: usize = std::mem::size_of::<MinimalHeader>();

pub fn get_digest(buffer: &[u8]) -> Result<[u32; 4]> {
    assert_eq!(HEADER_SIZE, 20);
    if buffer.len() < HEADER_SIZE {
        Err(Error::from(ErrorKind::InvalidData))
    } else {
        let buffer_ptr: *const u8 = buffer.as_ptr();
        let header_ptr: *const MinimalHeader = buffer_ptr as *const _;
        let header_ref: &MinimalHeader = unsafe { &*header_ptr };
        let digest: [u32; 4] = [
            header_ref.hash_digest[0],
            header_ref.hash_digest[1],
            header_ref.hash_digest[2],
            header_ref.hash_digest[3],
        ];
        Ok(digest)
    }
}

pub fn has_digest(buffer: &[u8]) -> Result<bool> {
    let hash_digest = get_digest(buffer)?;
    let mut has_digest = false;
    has_digest |= hash_digest[0] != 0x0;
    has_digest |= hash_digest[1] != 0x0;
    has_digest |= hash_digest[2] != 0x0;
    has_digest |= hash_digest[3] != 0x0;
    Ok(has_digest)
}

pub fn zero_digest(buffer: &mut [u8]) -> Result<()> {
    assert_eq!(HEADER_SIZE, 20);
    if buffer.len() < HEADER_SIZE {
        Err(Error::from(ErrorKind::InvalidData))
    } else {
        let buffer_ptr: *mut u8 = buffer.as_mut_ptr();
        let header_ptr: *mut MinimalHeader = buffer_ptr as *mut _;
        let header_mut: &mut MinimalHeader = unsafe { &mut *header_ptr };
        header_mut.hash_digest[0] = 0x0;
        header_mut.hash_digest[1] = 0x0;
        header_mut.hash_digest[2] = 0x0;
        header_mut.hash_digest[3] = 0x0;
        Ok(())
    }
}

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let meta = file.metadata()?;
    let size = meta.len() as usize;
    let mut data = vec![0; size];
    file.read_exact(&mut data)?;
    Ok(data)
}

pub fn validate(
    validator: &mut DxcValidator,
    input_path: &Path,
    output_path: Option<&Path>,
    force: bool,
) -> Result<()> {
    let mut input_data = read_file(input_path)?;

    println!("Signing DXIL file: {:?}", &input_path);

    // Check if DXIL is already signed
    if has_digest(&input_data)? {
        if force {
            // Clear the existing digest
            println!("  DXIL is already signed - clearing existing digest.");
            zero_digest(&mut input_data)?;
        } else {
            println!(
                "  DXIL is already signed - digest: {:?}",
                get_digest(&input_data)?
            );
            return Ok(());
        }
    }

    match validator.validate_slice(&input_data) {
        Ok((output_data, errors)) => {
            if errors.is_empty() {
                // Make sure the DXIL is now signed
                if has_digest(&output_data)? {
                    println!(
                        "  DXIL is now signed - digest: {:?}",
                        get_digest(&output_data)?
                    );
                } else {
                    eprintln!("  Validation failed: data is not signed.");
                    return Err(Error::from(ErrorKind::Other));
                }
                if let Some(output_path) = output_path {
                    println!("  Saving result: {:?}", &output_path);
                    let output_file = File::create(output_path)?;
                    let mut output_writer = BufWriter::new(output_file);
                    output_writer.write_all(&output_data)?;
                }
                Ok(())
            } else {
                eprintln!("  Validation failed: {:?}", errors);
                Err(Error::from(ErrorKind::InvalidData))
            }
        }
        Err(err) => {
            eprintln!("  Error validating DXIL: {:?}", err);
            Err(Error::from(ErrorKind::Other))
        }
    }
}

fn main() {
    let settings = Settings::from_args();
    println!("{:?}", settings);

    let dxil = Dxil::new();
    let validator = dxil.create_validator();

    let requests: Vec<(PathBuf, PathBuf)> = vec![(settings.input, settings.output)];

    match validator {
        Ok(mut validator) => {
            let version = validator.version().unwrap_or_else(|_| (0, 0));
            println!("Validation version: {}.{}", version.0, version.1);

            // Process each signing request
            for (ref input_path, ref output_path) in requests {
                if let Err(err) = validate(
                    &mut validator,
                    &input_path,
                    Some(&output_path),
                    settings.force,
                ) {
                    eprintln!("Validation failed: {:?}", err);
                    exit(1);
                }
            }

            println!("Validation complete");
            exit(0);
        }
        Err(err) => {
            eprintln!("Error creating DXIL validator: {:?}", err);
            exit(2);
        }
    }
}
