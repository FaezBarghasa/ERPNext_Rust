/// Naming series: "ACC-SINV-.YYYY.-.#####" -> "ACC-SINV-2026-00001" (Stage 3.2).
pub struct NamingSeriesParser;
impl NamingSeriesParser {
    pub fn format(pattern: &str, year: u32, seq: u64) -> String {
        let mut out = pattern
            .replace(".YYYY.", &year.to_string())
            .replace("YYYY", &year.to_string());
        while out.contains("--") {
            out = out.replace("--", "-");
        }
        out = out.replace(".-.", "-").replace('.', "");
        // count trailing # run
        let hashes = pattern.chars().rev().take_while(|&c| c == '#').count();
        if hashes > 0 {
            let num = format!("{:0>width$}", seq, width = hashes);
            // replace last run of #
            let idx = out.rfind('#').map(|i| i + 1 - hashes).unwrap_or(out.len());
            out.replace_range(idx..idx + hashes, &num);
        }
        out.replace('#', &seq.to_string())
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
mod t {
    use super::*;
    #[test]
    fn series_formats() {
        assert_eq!(
            NamingSeriesParser::format("ACC-SINV-.YYYY.-.#####", 2026, 1),
            "ACC-SINV-2026-00001"
        );
    }
}
