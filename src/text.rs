use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

// fn find_contains(vector: &Vec<String>, search: &str) -> (usize, String) {
//     let i = vector.into_iter().position(|r| r == search).unwrap();
//     let j = &vector[i];
//     return (i, j.to_string());
// }

#[derive(Clone)]
pub struct Font {
    sourcefile: Vec<String>,
    cache: Vec<Vec<[u16; 2]>>,
}

impl Font {
    pub fn new(sourcefile: &str) -> Self {
        Font {
            sourcefile: Font::format_file(sourcefile),
            cache: vec!(),
        }
    }

    pub fn cacheglyph(&mut self, glyph: char) {
        let index = glyph.clone() as usize;
        loop {
            // println!("{}, {:?}", self.cache.len(), index);
            
            if self.cache.len() < index+1 {
                self.cache.push(vec![[0u16; 2]]);
            }
            else {
                self.cache[index] = vec![[1u16; 2]];
                self.cache[index] = self.clone().shape_char(glyph);
                break;
            }
        }
    }

    pub fn format_file(filelocation: &str) -> Vec<String> {
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
    pub fn shape_char(&mut self, glyph: char) -> Vec<[u16; 2]> {
        let mut found_encoding = false;
        let searchstr: &str = &("ENCODING ".to_owned() + &(glyph as u8).to_string());
        let mut bitmaploc: usize = 0;
        let mut bitmap: Vec<[u16; 2]> = vec![];

        let mut cachehasglyph = self.cache.len() > glyph as usize;

        if cachehasglyph {
            if self.cache[glyph as usize] == vec![[0, 0]] {
                cachehasglyph = false
            }
        }

        if !cachehasglyph {
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
            self.cacheglyph(glyph)
        }
        else {
            bitmap = self.cache[glyph as usize].clone();
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
