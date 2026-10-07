#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, ch) in text.chars().enumerate() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }

    pub fn line_col(&self, char_offset: usize) -> (usize, usize) {
        let line = match self.line_starts.binary_search(&char_offset) {
            Ok(line) => line,
            Err(line) => line.saturating_sub(1),
        };
        let col = char_offset.saturating_sub(self.line_starts[line]);
        (line, col)
    }

    pub fn offset(&self, line: usize, col: usize) -> Option<usize> {
        self.line_starts.get(line).map(|&start| start + col)
    }
}
