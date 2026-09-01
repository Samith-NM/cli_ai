use crossterm::style::Stylize;
use std::io::Write;

pub struct StreamRenderer {
    in_bold: bool,
    in_code: bool,
    at_line_start: bool,
    pending_star: bool,
}
impl StreamRenderer {
    pub fn new() -> Self {
        Self {
            in_bold: false,
            in_code: false,
            at_line_start: true,
            pending_star: false,
        }
    }


    pub fn push(&mut self, chunk: &str) {
        let mut out = std::io::stdout();

        for ch in chunk.chars() {
            if ch == '*' {
                if self.pending_star {
                       self.in_bold = !self.in_bold;
                    self.pending_star = false;
                } else {
                    self.pending_star = true;
                }
                continue;
            }else if self.pending_star{
                self.write_styled(&mut out ,"*");
                self.pending_star = false;
            }
        

            if ch == '`' {
                self.in_code = !self.in_code;
                continue;
            }

            if self.at_line_start && ch == '#' {
                self.write_styled(&mut out, "▎"); // heading marker
                self.at_line_start = false;
                continue;
            }
            if self.at_line_start && ch == '-' {
                self.write_styled(&mut out, "•");
                self.at_line_start = false;
                continue;
            }

            self.at_line_start = ch == '\n';
            self.write_styled(&mut out, &ch.to_string());
        }
        out.flush().ok();
            
    }

    fn write_styled(&self, out: &mut impl Write, s: &str) {
        let styled = if self.in_code {
            s.dark_grey().on_black()
        }else if self.in_bold {
            s.bold()
        }else {
            s.stylize()
        };
        write!(out, "{styled}").ok();
    }

   
    pub fn finish(&mut self) {
        if self.pending_star {
            self.write_styled(&mut std::io::stdout(), "*");
            self.pending_star = false;
        }
        println!();
    

        

    }
}