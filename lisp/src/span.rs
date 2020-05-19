use std::ops::Range;

#[derive(Debug, Copy, Clone)]
pub struct Span {
    start_index: usize,
    end_index: usize,
}

impl Span {
    pub fn single_byte(index: usize) -> Span {
        Span {
            start_index: index,
            end_index: index+1,
        }
    }

    pub fn start_index(self) -> usize {
        self.start_index
    }

    pub fn end_index(self) -> usize {
        self.end_index
    }

    pub fn index_range(self) -> Range<usize> {
        self.start_index()..self.end_index()
    }

    pub fn line_range(self, s: &str) -> Range<usize> {
        get_line_index(self.start_index(), s)
            ..get_line_index(self.end_index(), s)
    }

    pub fn single_line(self, s: &str) -> bool {
        self.slice(s).bytes().all(|b| b != b'\n')
    }

    pub fn contains(self, i: usize) -> bool {
        self.index_range().contains(&i)
    }

    pub fn slice(self, s: &str) -> &str {
        &s[self.index_range()]
    }
}

impl From<pest::Span<'_>> for Span {
    fn from(span: pest::Span) -> Span {
        Span { start_index: span.start(), end_index: span.end() }
    }
}

fn get_line_index(byte_index: usize, string: &str) -> usize {
    let mut newlines_so_far = 0;

    for (i, b) in string.bytes().enumerate() {
        if i == byte_index { break }

        if b == b'\n' {
            newlines_so_far += 1;
        }
    }

    newlines_so_far+1
}

