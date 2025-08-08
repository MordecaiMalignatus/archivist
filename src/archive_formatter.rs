use serde_json::ser::CompactFormatter;
use serde_json::ser::Formatter;

use std::io;

pub struct ArchiveFormatter {
    inside_archive: bool,
    inside_card: bool,
    indent: usize,
}

impl ArchiveFormatter {
    pub fn new() -> Self {
        ArchiveFormatter {
            inside_archive: false,
            inside_card: false,
            indent: 0,
        }
    }
}

impl Formatter for ArchiveFormatter {
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let c: String = if !self.inside_archive {
            // Start of the file
            self.inside_archive = true;
            self.indent += 2;
            "{\n".to_string()
        } else if !self.inside_card {
            // New card starting
            self.inside_card = true;
            (" ".repeat(self.indent) + "{").to_string()
        } else {
            // Sub-object inside a card that we may not expect, but handle anyway
            "{".to_string()
        };

        writer.write(c.as_bytes()).map(|_f| Ok(()))?
    }

    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let c = if self.inside_card {
            // End of card, since it contains no subobjecst
            self.inside_card = false;
            "}"
        } else if self.inside_archive {
            self.inside_archive = false;
            self.indent -= 2;
            &("\n".to_owned() + &" ".repeat(self.indent) + "}")
        } else {
            "}"
        };

        writer.write(c.as_bytes()).map(|_f| Ok(()))?
    }

    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let c = if !self.inside_card {
            let indent = " ".repeat(self.indent);
            if first {
                indent
            } else {
                ",\n".to_owned() + &indent
            }
        } else {
            if first { "" } else { "," }.to_string()
        };

        writer.write(c.as_bytes()).map(|_f| Ok(()))?
    }

    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let c = ": ";
        writer.write(c.as_bytes()).map(|_f| Ok(()))?
    }

    fn end_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        // let c: &[u8] = if self.inside_card { b"" } else { b"\n" };
        let c = b"";
        writer.write(c).map(|_f| Ok(()))?
    }

    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let c: &[u8] = match self.inside_card {
            true => {
                if first {
                    b""
                } else {
                    b","
                }
            }
            false => {
                if first {
                    b"\n"
                } else {
                    b",\n"
                }
            }
        };
        writer.write(c).map(|_f| Ok(()))?
    }

    fn end_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        //let c: &[u8] = if !self.inside_card { b"\n" } else { b"" };
        let c = b"";
        writer.write(c).map(|_f| Ok(()))?
    }

    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if !self.inside_card {
            self.indent += 2
        }
        let c = "[";
        writer.write(c.as_bytes()).map(|_f| Ok(()))?
    }

    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let c = if self.inside_card {
            "]"
        } else {
            self.indent -= 2;
            &("\n".to_owned() + &" ".repeat(self.indent) + "]")
        };
        writer.write(c.as_bytes()).map(|_f| Ok(()))?
    }

    // ======================================================================
    // all these below are taken from CompactFormatter and passed through.
    fn write_null<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_null(writer)
    }

    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_bool(writer, value)
    }

    fn write_byte_array<W>(&mut self, writer: &mut W, value: &[u8]) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_byte_array(writer, value)
    }

    fn write_char_escape<W>(
        &mut self,
        writer: &mut W,
        char_escape: serde_json::ser::CharEscape,
    ) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_char_escape(writer, char_escape)
    }

    fn write_number_str<W>(&mut self, writer: &mut W, value: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_number_str(writer, value)
    }

    fn write_raw_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_raw_fragment(writer, fragment)
    }

    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_string_fragment(writer, fragment)
    }

    fn write_f32<W>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_f32(writer, value)
    }

    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_f64(writer, value)
    }

    fn write_i8<W>(&mut self, writer: &mut W, value: i8) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_i8(writer, value)
    }

    fn write_i16<W>(&mut self, writer: &mut W, value: i16) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_i16(writer, value)
    }

    fn write_i32<W>(&mut self, writer: &mut W, value: i32) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_i32(writer, value)
    }

    fn write_i64<W>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_i64(writer, value)
    }

    fn write_i128<W>(&mut self, writer: &mut W, value: i128) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_i128(writer, value)
    }

    fn write_u8<W>(&mut self, writer: &mut W, value: u8) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_u8(writer, value)
    }

    fn write_u16<W>(&mut self, writer: &mut W, value: u16) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_u16(writer, value)
    }

    fn write_u32<W>(&mut self, writer: &mut W, value: u32) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_u32(writer, value)
    }

    fn write_u64<W>(&mut self, writer: &mut W, value: u64) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_u64(writer, value)
    }

    fn write_u128<W>(&mut self, writer: &mut W, value: u128) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        CompactFormatter.write_u128(writer, value)
    }
}
#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use crate::Card;
    use crate::serialize_with_formatter;

    #[test]
    fn test_formatter_single_list() {
        let mut data = HashMap::new();
        data.insert(
            "test".to_string(),
            vec![Card {
                name: "test_card".to_string(),
                set_name: "The Test Set".to_string(),
                oracle_id: String::new(),
                count: 1,
                colors: vec!["W".to_string()],
                rarity: String::new(),
                uri: String::new(),
                set: "TEST".to_string(),
                collector_number: "40".to_string(),
                foil: false,
            }],
        );

        let file_content = serialize_with_formatter(&mut data).expect("formatter should work fine");

        let wanted_result = r#"{
  "test": [
    {"name": "test_card","collector_number": "40","set_name": "The Test Set","oracle_id": "","count": 1,"colors": ["W"],"rarity": "","uri": "","set": "TEST","foil": false}
  ]
}"#.to_string();

        let s: String =
            String::from_utf8(file_content).expect("serde_json should produce valid UTF-8");
        assert_eq!(s, wanted_result)
    }

    // This test is commented out as it fails frequetly, HashMaps do not
    // preserve order so this test flakes about 50% of time, while remaining
    // useful to debug formatting problems when needed.

//     #[test]
//     fn test_multiple_sets() {
//         let mut data = HashMap::new();
//         data.insert(
//             "test".to_string(),
//             vec![Card {
//                 name: "test_card".to_string(),
//                 set_name: "The Test Set".to_string(),
//                 oracle_id: String::new(),
//                 count: 1,
//                 colors: vec!["W".to_string()],
//                 rarity: String::new(),
//                 uri: String::new(),
//                 set: "TEST".to_string(),
//                 collector_number: "41".to_string(),
//                 foil: false,
//             }],
//         );
//         data.insert(
//             "second_test".to_string(),
//             vec![Card {
//                 name: "second test card".to_string(),
//                 set_name: "The Second Test Set".to_string(),
//                 oracle_id: String::new(),
//                 count: 1,
//                 colors: vec!["B".to_string()],
//                 rarity: "".to_string(),
//                 uri: "".to_string(),
//                 set: "SECOND TEST".to_string(),
//                 collector_number: "42".to_string(),
//                 foil: false,
//             }],
//         );
//         let file_content = serialize_with_formatter(&mut data).expect("formatter should work fine");

//         let wanted_result = r#"{
//   "test": [
//     {"name": "test_card","collector_number": "41","set_name": "The Test Set","oracle_id": "","count": 1,"colors": ["W"],"rarity": "","uri": "","set": "TEST","foil": false}
//   ],
//   "second_test": [
//     {"name": "second test card","collector_number": "42","set_name": "The Second Test Set","oracle_id": "","count": 1,"colors": ["B"],"rarity": "","uri": "","set": "SECOND TEST","foil": false}
//   ]
// }"#.to_string();

//         let s: String =
//             String::from_utf8(file_content).expect("serde_json should produce valid UTF-8");
//         assert_eq!(s, wanted_result)
//     }

    #[test]
    fn test_multiple_entries_in_one_set() {
        let mut data = HashMap::new();
        data.insert(
            "test".to_string(),
            vec![
                Card {
                    name: "test_card".to_string(),
                    set_name: "The Test Set".to_string(),
                    oracle_id: String::new(),
                    count: 1,
                    colors: vec!["W".to_string()],
                    rarity: String::new(),
                    uri: String::new(),
                    set: "TEST".to_string(),
                    collector_number: "41".to_string(),
                    foil: false,
                },
                Card {
                    name: "second test card".to_string(),
                    set_name: "The Test Set".to_string(),
                    oracle_id: String::new(),
                    count: 1,
                    colors: vec!["B".to_string()],
                    rarity: "".to_string(),
                    uri: "".to_string(),
                    set: "TEST".to_string(),
                    collector_number: "42".to_string(),
                    foil: false,
                },
            ],
        );
        let file_content = serialize_with_formatter(&mut data).expect("formatter should work fine");

        let wanted_result = r#"{
  "test": [
    {"name": "test_card","collector_number": "41","set_name": "The Test Set","oracle_id": "","count": 1,"colors": ["W"],"rarity": "","uri": "","set": "TEST","foil": false},
    {"name": "second test card","collector_number": "42","set_name": "The Test Set","oracle_id": "","count": 1,"colors": ["B"],"rarity": "","uri": "","set": "TEST","foil": false}
  ]
}"#.to_string();

        let s: String =
            String::from_utf8(file_content).expect("serde_json should produce valid UTF-8");
        assert_eq!(s, wanted_result)
    }
}
