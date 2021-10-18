use std::fmt::Display;

pub const MAX_DISPLAY_BYTES: usize = 16;

#[derive(Debug)]
pub struct ToHex(pub Vec<u8>);

impl ToHex {
    pub fn print_n(&self, f: &mut dyn std::fmt::Write, n: usize) {
        let mut is_omitted = false;

        self.0.iter().enumerate().for_each(|(i, x)| {
            if i >= n {
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
    }
}

impl Display for ToHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.print_n(f, MAX_DISPLAY_BYTES);
        Ok(())
    }
}
