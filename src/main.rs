extern crate byteorder;
extern crate fastnbt;
extern crate flate2;
extern crate serde;

use std::fs::{File, read_dir};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::Read;
use std::time::SystemTime;
use std::thread;
use std::path::Path;

use flate2::read::ZlibDecoder;
use byteorder::{ReadBytesExt, BigEndian};
use serde::Deserialize;
use fastnbt::de;
use fastnbt::error::Result;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ChunkDat<'a> {
    #[serde(borrow)]
    level: LevelDat<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct LevelDat<'a> {
    #[serde(borrow)]
    tile_entities: Vec<TileEntitiy<'a>>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct TileEntitiy<'a> {
    #[serde(rename = "x")]
    x: i32,
    #[serde(rename = "y")]
    y: i32,
    #[serde(rename = "z")]
    z: i32,
    text1: Option<&'a str>,
    text2: Option<&'a str>,
    text3: Option<&'a str>,
    text4: Option<&'a str>,
}

// format sign text
fn format_sign_text(t: &str) -> &str {
    let d: Vec<_> = t.split('"').collect();
    match t.contains("extra") {
        true => d[5],
        false => d[3]
    }
}

// main function
fn main() {
    let start = SystemTime::now();

    let delimiter = "--------------------\n";
    let mut elif = File::create("out.txt").unwrap();
    elif.write_all(delimiter.as_bytes()).unwrap();

    let mut threads1 = vec![];
    let path = Path::new("world/region");
    for entry in read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.extension() == None || path.extension().unwrap() != "mca" {
            continue;
        }
        let mut yo = elif.try_clone().unwrap();

        threads1.push(thread::spawn(move || {
            let mut file = File::open(&path).unwrap();
            
            for i in 0..1024 {
                file.seek(SeekFrom::Start(i * 4)).unwrap();
                let offset = 4096 * file.read_u24::<BigEndian>().unwrap();
                if offset <= 0 {
                    continue;
                }
                let size = 4096 * (file.read_u8().unwrap() as u32);

                let mut data = vec![];
                file.seek(SeekFrom::Start((offset + 5) as u64)).unwrap();
                let reference = Read::by_ref(&mut file).take((size - 5) as u64);
                ZlibDecoder::new(reference).read_to_end(&mut data).unwrap();

                let aaa: Result<ChunkDat> = de::from_bytes(data.as_slice());
                match aaa {
                    Ok(chunk) => {
                        for item in chunk.level.tile_entities.iter() {
                            if item.text1 != None {
                                let text1 = format_sign_text(item.text1.unwrap());
                                let text2 = format_sign_text(item.text2.unwrap());
                                let text3 = format_sign_text(item.text3.unwrap());
                                let text4 = format_sign_text(item.text4.unwrap());
                                if text1 == "" && text2 == "" && text3 == "" && text4 == "" {
                                    continue;
                                }
                                yo.write_all(format!("{} {} {}\n§ {}\n§ {}\n§ {}\n§ {}\n{}", item.x, item.y, item.z, text1, text2, text3, text4, delimiter).as_bytes()).unwrap();
                            }
                        }
                    },
                    Err(_) => {},
                };
            }

            println!("{} done", path.to_str().unwrap());
        }));

    }

    for child in threads1 {
        let _ = child.join();
    }

    println!("done, {}ms", start.elapsed().unwrap().as_millis());
    
}
