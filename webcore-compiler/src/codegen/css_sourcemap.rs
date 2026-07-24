//! Multi-source Source Map v3 for the generated stylesheet (#54).
//!
//! Unlike the JS source map (a single `.webc` source), the stylesheet is
//! assembled from every styled component, so its map carries one `.webc` source
//! per contributing file. Each entry maps a generated-CSS line to the line of
//! the originating rule in its `.webc`, letting DevTools jump from a scoped rule
//! back to the source. Emitted in dev only (prod CSS is minified — no map).

use crate::codegen::js::sourcemap::encode_vlq;

/// One `outputLine → (source, sourceLine)` correspondence (column 0 on both
/// sides — mapping is line-granular).
struct Segment {
    out_line: u32,
    source: usize,
    src_line: u32,
}

/// Builder for a multi-source CSS source map v3.
pub(crate) struct CssSourceMap {
    file: String,
    sources: Vec<String>,
    sources_content: Vec<String>,
    segments: Vec<Segment>,
}

impl CssSourceMap {
    pub(crate) fn new(file: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            sources: Vec::new(),
            sources_content: Vec::new(),
            segments: Vec::new(),
        }
    }

    /// Register a source file (deduplicated by name) and return its index.
    pub(crate) fn add_source(&mut self, name: &str, content: &str) -> usize {
        if let Some(i) = self.sources.iter().position(|s| s == name) {
            return i;
        }
        self.sources.push(name.to_string());
        self.sources_content.push(content.to_string());
        self.sources.len() - 1
    }

    /// Map a 0-indexed generated line to a 0-indexed source line.
    pub(crate) fn add(&mut self, out_line: u32, source: usize, src_line: u32) {
        self.segments.push(Segment {
            out_line,
            source,
            src_line,
        });
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Encode the source map as a v3 JSON string.
    pub(crate) fn build(&self) -> String {
        let max_line = self.segments.iter().map(|s| s.out_line).max().unwrap_or(0);
        let mut lines: Vec<Vec<&Segment>> = vec![Vec::new(); (max_line + 1) as usize];
        for s in &self.segments {
            lines[s.out_line as usize].push(s);
        }

        // Source index and source line persist across segments/lines; output
        // column resets to 0 at the start of each line (v3 spec).
        let mut prev_source: i32 = 0;
        let mut prev_src_line: i32 = 0;
        let mut vlq_lines: Vec<String> = Vec::with_capacity(lines.len());

        for line_segs in &lines {
            if line_segs.is_empty() {
                vlq_lines.push(String::new());
                continue;
            }
            let mut prev_out_col: i32 = 0;
            let mut segs: Vec<String> = Vec::with_capacity(line_segs.len());
            for s in line_segs {
                // Column 0 on both sides; only line/source deltas carry info.
                let seg = format!(
                    "{}{}{}{}",
                    encode_vlq(0 - prev_out_col),
                    encode_vlq(s.source as i32 - prev_source),
                    encode_vlq(s.src_line as i32 - prev_src_line),
                    encode_vlq(0), // source column delta (always 0 → 0)
                );
                segs.push(seg);
                prev_out_col = 0;
                prev_source = s.source as i32;
                prev_src_line = s.src_line as i32;
            }
            vlq_lines.push(segs.join(","));
        }

        let mappings = vlq_lines.join(";");
        let sources = json_array(&self.sources);
        let contents = json_array(&self.sources_content);
        let file = json_escape(&self.file);
        format!(
            r#"{{"version":3,"file":"{file}","sources":{sources},"sourcesContent":{contents},"names":[],"mappings":"{mappings}"}}"#
        )
    }
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn json_array(items: &[String]) -> String {
    let inner: Vec<String> = items
        .iter()
        .map(|s| format!("\"{}\"", json_escape(s)))
        .collect();
    format!("[{}]", inner.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_map_has_no_segments() {
        let m = CssSourceMap::new("theme.css");
        assert!(m.is_empty());
    }

    #[test]
    fn dedups_sources_and_emits_v3() {
        let mut m = CssSourceMap::new("theme.css");
        let a = m.add_source("Btn.webc", "component Btn {}");
        let b = m.add_source("Btn.webc", "component Btn {}");
        assert_eq!(a, b, "same source name must dedup to one index");
        let c = m.add_source("Card.webc", "component Card {}");
        assert_ne!(a, c);
        m.add(5, a, 3);
        m.add(6, c, 9);
        let json = m.build();
        assert!(json.contains(r#""version":3"#));
        assert!(json.contains(r#""file":"theme.css""#));
        assert!(json.contains("Btn.webc") && json.contains("Card.webc"));
        assert!(json.contains(r#""mappings":"#));
        // Two sources, two contents.
        assert_eq!(json.matches(".webc").count(), 2);
    }
}
