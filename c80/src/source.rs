//! Source files, byte-offset spans, and per-compilation IDs.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl SourceSpan {
    pub fn new(file: FileId, start: u32, end: u32) -> Self {
        Self { file, start, end }
    }

    pub fn point(file: FileId, offset: u32) -> Self {
        Self {
            file,
            start: offset,
            end: offset,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        debug_assert_eq!(self.file, other.file);
        Self {
            file: self.file,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn len(self) -> u32 {
        self.end.saturating_sub(self.start)
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub id: FileId,
    pub name: String,
    pub text: String,
    line_starts: Vec<u32>,
}

impl SourceFile {
    pub fn new(id: FileId, name: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let line_starts = line_starts(&text);
        Self {
            id,
            name: name.into(),
            text,
            line_starts,
        }
    }

    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        line_col(&self.line_starts, offset, self.text.len() as u32)
    }

    pub fn slice(&self, span: SourceSpan) -> &str {
        debug_assert_eq!(span.file, self.id);
        let start = (span.start as usize).min(self.text.len());
        let end = (span.end as usize).min(self.text.len());
        &self.text[start..end]
    }
}

#[derive(Debug, Clone)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn add(&mut self, name: impl Into<String>, text: impl Into<String>) -> FileId {
        let id = FileId(self.files.len() as u32);
        self.files.push(SourceFile::new(id, name, text));
        id
    }

    pub fn get(&self, id: FileId) -> &SourceFile {
        &self.files[id.0 as usize]
    }

    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    pub fn line_col(&self, span: SourceSpan) -> (u32, u32) {
        self.get(span.file).line_col(span.start)
    }
}

#[derive(Debug)]
pub struct IdGen {
    next: u32,
}

impl IdGen {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn next(&mut self) -> NodeId {
        let id = NodeId(self.next);
        self.next = self.next.saturating_add(1);
        id
    }
}

fn line_starts(text: &str) -> Vec<u32> {
    let mut starts = vec![0];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            starts.push((i + 1) as u32);
        }
    }
    starts
}

fn line_col(starts: &[u32], offset: u32, len: u32) -> (u32, u32) {
    let offset = offset.min(len);
    let idx = starts
        .partition_point(|&start| start <= offset)
        .saturating_sub(1);
    let line = idx as u32 + 1;
    let col = offset - starts[idx] + 1;
    (line, col)
}
