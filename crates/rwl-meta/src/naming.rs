use compact_str::CompactString;
use std::collections::HashMap;

/// Naming series & autoname generator matching upstream Frappe autoname rules:
/// - `ACC-SINV-.YYYY.-.#####` -> `ACC-SINV-2026-00001`
/// - `format:PO-{supplier}-{YY}-{#####}`
/// - `field:naming_series`
/// - `hash` / `hash:8`
/// - `autoincrement`
pub struct NamingSeriesParser;

impl NamingSeriesParser {
    /// Formats a pattern with year, month, day, sequence, and optional document fields.
    pub fn format(pattern: &str, year: u32, seq: u64) -> String {
        Self::format_with_context(pattern, year, 1, 1, seq, &HashMap::new())
    }

    /// Comprehensive autoname formatter supporting date tokens (`YYYY`, `YY`, `MM`, `DD`),
    /// hash expressions, sequence hashes (`#####`), and `{field_name}` substitutions.
    pub fn format_with_context(
        pattern: &str,
        year: u32,
        month: u32,
        day: u32,
        seq: u64,
        fields: &HashMap<CompactString, CompactString>,
    ) -> String {
        let clean_pattern = pattern.strip_prefix("format:").unwrap_or(pattern);

        let yy = format!("{:02}", year % 100);
        let yyyy = year.to_string();
        let mm = format!("{:02}", month);
        let dd = format!("{:02}", day);

        let mut out = clean_pattern
            .replace(".YYYY.", &yyyy)
            .replace(".YY.", &yy)
            .replace(".MM.", &mm)
            .replace(".DD.", &dd)
            .replace("{YYYY}", &yyyy)
            .replace("{YY}", &yy)
            .replace("{MM}", &mm)
            .replace("{DD}", &dd)
            .replace("YYYY", &yyyy);

        // Replace custom {field} references
        for (field, val) in fields {
            let tag = format!("{{{field}}}");
            if out.contains(&tag) {
                out = out.replace(&tag, val.as_str());
            }
        }

        while out.contains("--") {
            out = out.replace("--", "-");
        }
        out = out.replace(".-.", "-").replace('.', "");

        // Clean sequence placeholders like {#####} or {####} first
        if out.contains("{#####}") {
            let num = format!("{:05}", seq);
            out = out.replace("{#####}", &num);
        } else if out.contains("{####}") {
            let num = format!("{:04}", seq);
            out = out.replace("{####}", &num);
        } else if let Some(start_hash) = out.find('#') {
            let hash_count = out[start_hash..].chars().take_while(|&c| c == '#').count();
            if hash_count > 0 {
                let num = format!("{:0>width$}", seq, width = hash_count);
                out.replace_range(start_hash..start_hash + hash_count, &num);
            }
        }

        // Clean any leftover '#' with raw sequence
        out.replace('#', &seq.to_string())
    }

    /// Evaluates Frappe `autoname` specification:
    /// - `hash` or `hash:N`
    /// - `field:field_name`
    /// - `format:...`
    /// - `autoincrement`
    /// - `prompt`
    /// - Series (e.g. `INV-.YYYY.-.#####`)
    pub fn parse_autoname(
        autoname_spec: &str,
        doc_fields: &HashMap<CompactString, CompactString>,
        year: u32,
        seq: u64,
    ) -> String {
        let spec = autoname_spec.trim();
        if spec == "hash" {
            Self::generate_hash(10)
        } else if let Some(len_str) = spec.strip_prefix("hash:") {
            let len = len_str.parse::<usize>().unwrap_or(10);
            Self::generate_hash(len)
        } else if let Some(field) = spec.strip_prefix("field:") {
            let field_key = CompactString::from(field.trim());
            doc_fields
                .get(&field_key)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("DOC-{seq}"))
        } else if spec == "autoincrement" {
            seq.to_string()
        } else if spec == "prompt" {
            doc_fields
                .get(&CompactString::from("name"))
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("DOC-{seq}"))
        } else {
            Self::format_with_context(spec, year, 1, 1, seq, doc_fields)
        }
    }

    /// Deterministic alphanumeric pseudo-random hash generator for `hash` autonames.
    pub fn generate_hash(len: usize) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(1_700_000_000);
        let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
        let mut seed = nanos;
        let mut out = String::with_capacity(len);
        for _ in 0..len {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let idx = (seed >> 32) as usize % chars.len();
            out.push(chars[idx] as char);
        }
        out
    }

    /// SurrealQL atomic counter statement for concurrent inserts.
    pub fn counter_stmt(table: &str) -> String {
        format!(
            "UPDATE counter:{} SET current_value += 1 RETURN current_value;",
            table
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_formats() {
        assert_eq!(
            NamingSeriesParser::format("ACC-SINV-.YYYY.-.#####", 2026, 1),
            "ACC-SINV-2026-00001"
        );
        assert_eq!(
            NamingSeriesParser::format("PO-.YY.-.####", 2026, 42),
            "PO-26-0042"
        );
    }

    #[test]
    fn test_format_with_fields() {
        let mut fields = HashMap::new();
        fields.insert("supplier".into(), "ACME".into());
        fields.insert("branch".into(), "NYC".into());

        let res = NamingSeriesParser::format_with_context(
            "format:PO-{supplier}-{branch}-{YY}-{#####}",
            2026,
            10,
            17,
            123,
            &fields,
        );
        assert_eq!(res, "PO-ACME-NYC-26-00123");
    }

    #[test]
    fn test_parse_autoname_options() {
        let mut fields = HashMap::new();
        fields.insert("item_code".into(), "WIDGET-01".into());

        assert_eq!(
            NamingSeriesParser::parse_autoname("field:item_code", &fields, 2026, 1),
            "WIDGET-01"
        );
        assert_eq!(
            NamingSeriesParser::parse_autoname("autoincrement", &fields, 2026, 999),
            "999"
        );
        let hash = NamingSeriesParser::parse_autoname("hash:10", &fields, 2026, 1);
        assert_eq!(hash.len(), 10);
    }
}
