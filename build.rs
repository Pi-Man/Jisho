use std::{fs::File, io::{BufRead, BufReader, Write}, path::Path};




fn convert_line(line: &str) -> Option<String> {
    let mut state = 0;
    let mut iden = String::new();
    let mut val = String::new();
    for token in line.split_whitespace() {
        match state {
            0 => {
                if token == "#define" {
                    state = 1;
                }
                else {
                    return None
                }
            }
            1 => {
                iden = token.to_owned();
                state = 2;
            }
            2 => {
                val = token.to_owned();
                state = 3;
            }
            _ => return None
        }
    }
    if state == 3 {
        Some(format!("pub const {}: u32 = {};\n", iden, val))
    }
    else {
        Some("\n".to_owned())
    }
}

fn gen<P>(header: P, out: P) -> Result<(), std::io::Error>
where P: AsRef<Path>
{
    match File::open(header) {
        Ok(file_in) => {
            match File::create(out) {
                Ok(mut file_out) => {
                    for line in BufReader::new(file_in).lines() {
                        match line {
                            Ok(line) => {
                                let new_line = convert_line(&line);
                                if let Some(s) = new_line {
                                    if let Err(err) = file_out.write(s.as_bytes()) {
                                        return Err(err)
                                    }
                                }
                            }
                            Err(err) => {
                                return Err(err)
                            }
                        }
                    }
                    Ok(())
                }
                Err(err) => {
                    Err(err)
                }
            }
        }
        Err(err) => {
            Err(err)
        }
    }
}









fn main() {
    if let Err(err) = gen("src\\resources.h", "src\\resources.rs") {
        eprintln!("Error: {}", err);
    }
    println!("cargo:rustc-link-search=native=resources");
    println!("cargo:rustc-link-lib=static=jisho");
}