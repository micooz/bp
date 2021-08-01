use std::fmt::Display;

#[derive(Debug)]
pub struct ToHex(pub Vec<u8>);

impl Display for ToHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_omitted = false;

        self.0.iter().enumerate().for_each(|(i, x)| {
            if i >= 32 {
                if !is_omitted {
                    write!(f, "... {} bytes omitted", self.0.len() - i).unwrap();
                }
                is_omitted = true;
                return;
            }
            write!(f, "{:02X?}", x).unwrap();

            if i < self.0.len() - 1 {
                f.write_str(" ").unwrap();
            }
        });

        Ok(())
    }
}

#[test]
fn test_to_hex_display() {
    assert_eq!(format!("{}", ToHex(vec![1, 10, 255])), "01 0A FF");
    assert_eq!(
        format!("{}", ToHex(vec![0; 33])),
        "00 ".repeat(32) + "... 1 bytes omitted"
    );
}
