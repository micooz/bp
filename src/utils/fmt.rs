use std::fmt::Display;

pub struct ToHex(pub Vec<u8>);

impl Display for ToHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{").unwrap();

        self.0.iter().enumerate().for_each(|(i, x)| {
            write!(f, "{:#04X?}", x).unwrap();

            if i < self.0.len() - 1 {
                f.write_str(" ").unwrap();
            }
        });

        f.write_str("}").unwrap();

        Ok(())
    }
}
