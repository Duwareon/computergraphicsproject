use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;


fn find_contains(vector: &Vec<String>, search: &str) -> (usize, String) {
    let i = vector.into_iter().position(|r| r == search).unwrap();
    let j = &vector[i];
    return (i, j.to_string());
}

fn convert_to_binary_from_hex(hex: &str) -> String {
    hex.chars().map(to_binary).collect()
}

fn to_binary(c: char) -> &'static str {
    match c {
        '0' => "0000",
        '1' => "0001",
        '2' => "0010",
        '3' => "0011",
        '4' => "0100",
        '5' => "0101",
        '6' => "0110",
        '7' => "0111",
        '8' => "1000",
        '9' => "1001",
        'A' => "1010",
        'B' => "1011",
        'C' => "1100",
        'D' => "1101",
        'E' => "1110",
        'F' => "1111",
        _ => "",
    }
}

#[derive(Clone)]
pub struct Font {
    sourcefile: Vec<String>,
    // properties: FontProperties,
}

impl Font {
    pub fn new(sourcefile: Vec<String>) -> Self {
        Font {
            sourcefile: sourcefile,
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
    pub fn shape_char(self, glyph: char) -> Vec<String> {
        let file = self.sourcefile;

        // Find the requested character inside of the .BDF font file
        let search = find_contains(
            &file,
            &vec!["ENCODING", &(glyph as u32).to_string()].join(" "),
        );
        let linenum = search.0;

        // Get the needed bitmap information
        let mut i: u32 = 0;
        let mut bitarray: Vec<&str> = vec![];

        loop {
            i += 1;
            let mut currentline = linenum + i as usize;
            let line: &str = file[currentline].as_ref();
            let word: &str = line.split(" ").nth(0).unwrap();

            match word {
                /*"SWIDTH" => {
                    swidth = tokenize(line).try_into().unwrap();
                }

                "DWIDTH" => {
                    dwidth = tokenize(line).try_into().unwrap();
                }

                "BBX" => {
                    bbx = tokenize(line).try_into().unwrap();
                }*/
                "BITMAP" => {
                    loop {
                        i += 1;
                        currentline = linenum + i as usize;
                        let line: &str = file[currentline].as_ref();
                        let word: &str = line.split(" ").nth(0).unwrap();

                        match word {
                            "BITMAP" => continue,
                            "ENDCHAR" => break,
                            &_ => bitarray.append(&mut vec![line]),
                        }
                    }
                    break;
                }
                &_ => continue,
            }
        }

        let mut chararray: Vec<String> = vec![];

        for i in bitarray {
            chararray.append(&mut vec![convert_to_binary_from_hex(i)]);
        }

        return chararray;
    }
}

/*pub struct FontProperties {

}*/
