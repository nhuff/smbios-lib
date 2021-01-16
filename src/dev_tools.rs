//! Tools to speed library development.
//!
//! Place any functions here which are useful while developing the library.

use crate::read::*;
use crate::windows::*;
use std::fmt;
use std::fs;
use std::io;

// TODO:
// 1. Write a function to iterate through a folder full of windows DAT files
// 2. Write a function to iterate through a folder full of non-windows DAT files
// 3. Write a test function to output rust code for unit tests of each type by
//    finding candidates from a file collection.

fn load_windows_raw_files(folder: &str) -> Vec<RawSMBiosData> {
    let mut result = Vec::new();

    let entries = fs::read_dir(folder)
        .expect("valid files")
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .expect("msg");

    for elem in entries {
        let file_name = elem.to_str().expect("valid filename characters");

        // Temporary output to see what files we found
        // println!("{}", file_name);

        let raw_smbios_data = load_smbios_data_file(file_name);
        match raw_smbios_data {
            Ok(data) => result.push(data),
            Err(_) => {}
        }
    }

    result
}

struct PrintableArray<'a>(&'a [u8]);

impl<'a> fmt::Display for PrintableArray<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vec = self.0;

        write!(f, "[\n    ")?;
        for (count, v) in vec.iter().enumerate() {
            if count != 0 {
                write!(f, ",")?;
                if count % 16 == 0 {
                    write!(f, "\n    ")?;
                } else {
                    write!(f, " ")?;
                }
            }
            write!(f, "{:#04X}", v)?;
        }

        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_find_type() {
        for type_to_find in 0..44u8 {
            println!("#[cfg(test)]");
            println!("mod tests {{");
            println!("use super::*;");
            println!();
            println!("#[test]");
            println!("fn unit_test() {{");

            let results =
                load_windows_raw_files(r"C:\Users\Jeff\Desktop\BIOSRawFiles\WindowsHeader");
            for raw_data in results {
                let found_structure = raw_data
                    .smbios_table_data()
                    .into_iter()
                    .find(|x| x.header.struct_type() == type_to_find);

                if found_structure.is_some() {
                    let structure = found_structure.unwrap();
                    println!(
                        "let struct_type{} = vec!{};",
                        type_to_find,
                        PrintableArray(structure.data)
                    );
                    println!();

                    // println!("{:?}", structure.struct_type_name());

                    println!(
                        "let parts = SMBiosStructParts::new(struct_type{}.as_slice());",
                        type_to_find
                    );
                    println!("let test_struct = SMBiosBios_::new(&parts);");
                    println!();

                    // Get and parse fmt::Debug into field names and their values to test
                    let mut struct_string = Vec::new();
                    write!(&mut struct_string, "{:?}", structure.struct_type_name()).unwrap();
                    let str_string: String = struct_string.into_iter().map(|x| x as char).collect();
                    let strip_header = str_string.split("} }");
                    let non_header = strip_header.skip(1).next().unwrap();
                    let parts = non_header.split(": ");

                    let mut field_name = String::new();
                    let mut field_value = String::new();
                    for part in parts {
                        let mut field_and_value = part.split(", ");
                        field_value = field_and_value.next().unwrap().to_string();

                        if field_value.contains("\"") {
                            field_value = field_value.replace(")", ".to_string())");
                        }

                        println!("assert_eq!(test_struct.{}(), {});", field_name, field_value);

                        let next = field_and_value.next();
                        if next.is_some() {
                            field_name = next.unwrap().to_string();
                        }
                    }
                    println!("}}");
                    println!("}}");
                    println!();

                    break;
                }
            }
        }
    }
}
