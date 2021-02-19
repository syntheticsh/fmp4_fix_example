use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};

use anyhow::{anyhow, Result};
use byteorder::{BigEndian, ReadBytesExt};

fn main() -> Result<()> {
    let mut reader = BufReader::new(
        std::fs::OpenOptions::new().read(true).open(
            std::env::args()
                .nth(1)
                .ok_or_else(|| anyhow!("no input file"))?,
        )?,
    );

    let mut content = Vec::new();
    let _content_size = reader.read_to_end(&mut content)?;

    let (content, box_ftyp) = read_box(content)?; // read ftyp
    let (content, box_moov) = read_box(content)?; // read moov
    let (content, box_moof) = read_box(content)?; // read moof

    let mut fix = Cursor::new(box_moof);
    fix.seek(SeekFrom::Current(16))?; //skip to mfhd content

    let version: u8 = fix.read_u8()?;
    let flags: u32 = fix.read_u24::<BigEndian>()?;
    let mut sequence: u32 = fix.read_u32::<BigEndian>()?;

    println!(
        "version: {} flags: {} sequence {}",
        version, flags, sequence
    );

    sequence = 10;

    fix.seek(SeekFrom::Current(-4))?;
    fix.write_all(&sequence.to_be_bytes())?;

    let box_moof: Vec<u8> = fix.into_inner();

    let mut writer = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .open("test.mp4")?,
    );

    writer.write_all(&box_ftyp)?;
    writer.write_all(&box_moov)?;
    writer.write_all(&box_moof)?;
    writer.write_all(&content)?;

    Ok(())
}

fn read_box(content: Vec<u8>) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut reader = Cursor::new(content);

    let mut buf = [0u8; 8]; // 8 bytes for box header.
    reader.read_exact(&mut buf)?;
    // Get size.
    let mut s: [u8; 4] = Default::default();
    s.copy_from_slice(&buf[0..4]);
    let mut size = u32::from_be_bytes(s) as u64;
    if size == 1 {
        reader.read_exact(&mut buf)?;
        let mut ls: [u8; 8] = Default::default();
        ls.copy_from_slice(&buf);
        size = u64::from_be_bytes(ls);
    }

    // Get box type string.
    let mut t: [u8; 4] = Default::default();
    t.copy_from_slice(&buf[4..8]);
    let typ = u32::from_be_bytes(t);

    eprintln!("size: {} type: {:x?}", size, typ);

    let mut content = reader.into_inner();
    let mut box_: Vec<u8> = content.drain(0..size as usize).collect();

    Ok((content, box_))
}
