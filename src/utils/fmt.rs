use std::fmt::Display;

#[derive(Debug)]
pub struct ToHex(pub Vec<u8>);

impl Display for ToHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().enumerate().for_each(|(i, x)| {
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
}
