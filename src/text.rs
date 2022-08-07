use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

// fn find_contains(vector: &Vec<String>, search: &str) -> (usize, String) {
//     let i = vector.into_iter().position(|r| r == search).unwrap();
//     let j = &vector[i];
//     return (i, j.to_string());
// }

// fn push_to_index(i: usize, val: u8, x: &mut Vec<u8>) {
// 	loop {
// 		if x.len() < i {
// 			x.push(0);
// 		}
// 		else {
// 			x.push(val);
// 			break;
// 		}
// 	}
// }

#[derive(Clone)]
pub struct Font {
    sourcefile: Vec<String>,
    // characters: Vec<Glyph>,
}

// TODO: Make this index all characters on startup, easiest way is probably to just make a vec that takes a codepoint and has that correlate to the linenumber for the BITMAP
impl Font {
    pub fn new(sourcefile: &str) -> Self {
        Font {
            sourcefile: Font::format(sourcefile),
        }
    }

    pub fn format(filelocation: &str) -> Vec<String> {
        let file = File::open(filelocation).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();
        let fontfile = contents.split("\n").collect::<Vec<&str>>();

        let mut fontfile_formatted = Vec::new();

        for i in fontfile {
            fontfile_formatted.append(&mut vec![i.to_string()]);
        }

        return fontfile_formatted;
    }

    // Returns a bitmap of the requested char,
    pub fn shape_char(self, glyph: char) -> Vec<[u16; 2]> {
        let mut found_encoding = false;
        let searchstr: &str = &("ENCODING ".to_owned() + &(glyph as u8).to_string());
        let mut bitmaploc: usize = 0;
        let mut bitmap: Vec<[u16; 2]> = vec![];

        // Find where the bitmap info is in the font file
        for (line, text) in self.sourcefile.clone().into_iter().enumerate() {
            match text.as_str() {
                y if y == searchstr => found_encoding = true,

                "BITMAP" => {
                    if found_encoding {
                        bitmaploc = line + 1;
                        break;
                    }
                }
                _ => continue,
            }
        }

        let mut y = 0;
        // Loop through the lines of the BITMAP
        loop {
            let linenum = bitmaploc + y;
            let line: &str = &self.sourcefile[linenum].to_owned();
            match line {
                "ENDCHAR" => break,
                line => {
                    let lineint = i64::from_str_radix(line, 16).unwrap();
                    let binstring = format!("{:08b}", lineint);
                    // Loop through the binary digits of the line
                    for (x, j) in binstring.chars().enumerate() {
                        if j == '1' {
                            bitmap.append(&mut vec![[x as u16, y as u16]]);
                        }
                    }
                }
            }
            y += 1;
        }
        return bitmap;
    }
}

// #[derive(Clone)]
// pub struct Glyph {
//     swidth: [u16; 2],
//     dwidth: [u16; 2],
//     bbx: [u16; 4],
//     bitmap: Vec<String>,
// }
