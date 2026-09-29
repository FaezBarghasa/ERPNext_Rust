//! Fuzzy Header Matching & Multilingual Column Normalizer (`erp_importer::fuzzy_matcher`).

use compact_str::CompactString;
use std::collections::HashMap;

/// Standard ERP DocType target field schema definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetFieldSchema {
    pub field_name: CompactString,
    pub is_required: bool,
    pub aliases: Vec<CompactString>,
}

/// Fuzzy Column Classifier.
pub struct FuzzyMatcher;

impl FuzzyMatcher {
    /// Computes Levenshtein edit distance between two strings.
    #[must_use]
    pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let v1: Vec<char> = s1.chars().collect();
        let v2: Vec<char> = s2.chars().collect();
        let l1 = v1.len();
        let l2 = v2.len();

        let mut matrix = vec![vec![0usize; l2 + 1]; l1 + 1];

        for (i, row) in matrix.iter_mut().enumerate().take(l1 + 1) {
            row[0] = i;
        }
        for (j, cell) in matrix[0].iter_mut().enumerate().take(l2 + 1) {
            *cell = j;
        }

        for i in 1..=l1 {
            for j in 1..=l2 {
                let cost = if v1[i - 1] == v2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[l1][l2]
    }

    /// Matches raw spreadsheet header names against standard ERP DocType field schema.
    #[must_use]
    pub fn auto_map_headers(
        raw_headers: &[&str],
        schema: &[TargetFieldSchema],
    ) -> HashMap<usize, CompactString> {
        let mut mapping = HashMap::new();

        for (col_idx, &raw_header) in raw_headers.iter().enumerate() {
            let normalized_raw = raw_header.trim().to_lowercase();

            let mut best_field: Option<CompactString> = None;
            let mut min_distance = usize::MAX;

            for target in schema {
                // Exact field name match
                if target.field_name.to_lowercase() == normalized_raw {
                    best_field = Some(target.field_name.clone());
                    break;
                }

                // Match against multilingual aliases
                for alias in &target.aliases {
                    let normalized_alias = alias.trim().to_lowercase();
                    if normalized_alias == normalized_raw {
                        best_field = Some(target.field_name.clone());
                        min_distance = 0;
                        break;
                    }

                    let dist = Self::levenshtein_distance(&normalized_raw, &normalized_alias);
                    if dist < min_distance && dist <= 2 {
                        min_distance = dist;
                        best_field = Some(target.field_name.clone());
                    }
                }
            }

            if let Some(matched) = best_field {
                mapping.insert(col_idx, matched);
            }
        }

        mapping
    }

    /// Built-in schema for standard `Item` DocType with English, Farsi, Japanese, and Chinese aliases.
    #[must_use]
    pub fn default_item_schema() -> Vec<TargetFieldSchema> {
        vec![
            TargetFieldSchema {
                field_name: "item_code".into(),
                is_required: true,
                aliases: vec![
                    "sku".into(),
                    "product_id".into(),
                    "کد کالا".into(),
                    "شناسه کالا".into(),
                    "شماره قلم".into(),
                    "商品コード".into(),
                    "品番".into(),
                    "物料编号".into(),
                    "商品编码".into(),
                ],
            },
            TargetFieldSchema {
                field_name: "item_name".into(),
                is_required: true,
                aliases: vec![
                    "title".into(),
                    "product_name".into(),
                    "نام کالا".into(),
                    "شرح کالا".into(),
                    "品名".into(),
                    "商品名".into(),
                    "物料名称".into(),
                ],
            },
            TargetFieldSchema {
                field_name: "standard_rate".into(),
                is_required: false,
                aliases: vec![
                    "price".into(),
                    "rate".into(),
                    "unit_price".into(),
                    "قیمت".into(),
                    "نرخ واحد".into(),
                    "قیمت واحد".into(),
                    "単価".into(),
                    "価格".into(),
                    "单价".into(),
                ],
            },
            TargetFieldSchema {
                field_name: "stock_uom".into(),
                is_required: true,
                aliases: vec![
                    "uom".into(),
                    "unit".into(),
                    "واحد سنجش".into(),
                    "واحد".into(),
                    "単位".into(),
                    "计量单位".into(),
                ],
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(
            FuzzyMatcher::levenshtein_distance("item_code", "item_code"),
            0
        );
        assert_eq!(
            FuzzyMatcher::levenshtein_distance("item_code", "item_codx"),
            1
        );
        assert_eq!(FuzzyMatcher::levenshtein_distance("کالا", "کالاها"), 2);
    }

    #[test]
    fn test_multilingual_header_mapping() {
        let schema = FuzzyMatcher::default_item_schema();

        // 1. Farsi Headers
        let farsi_headers = ["کد کالا", "نام کالا", "قیمت واحد", "واحد سنجش"];
        let farsi_map = FuzzyMatcher::auto_map_headers(&farsi_headers, &schema);
        assert_eq!(farsi_map.get(&0).unwrap(), "item_code");
        assert_eq!(farsi_map.get(&1).unwrap(), "item_name");
        assert_eq!(farsi_map.get(&2).unwrap(), "standard_rate");
        assert_eq!(farsi_map.get(&3).unwrap(), "stock_uom");

        // 2. Japanese Headers
        let jp_headers = ["商品コード", "品名", "単価"];
        let jp_map = FuzzyMatcher::auto_map_headers(&jp_headers, &schema);
        assert_eq!(jp_map.get(&0).unwrap(), "item_code");
        assert_eq!(jp_map.get(&1).unwrap(), "item_name");
        assert_eq!(jp_map.get(&2).unwrap(), "standard_rate");

        // 3. Chinese Headers
        let cn_headers = ["物料编号", "物料名称", "单价"];
        let cn_map = FuzzyMatcher::auto_map_headers(&cn_headers, &schema);
        assert_eq!(cn_map.get(&0).unwrap(), "item_code");
        assert_eq!(cn_map.get(&1).unwrap(), "item_name");
        assert_eq!(cn_map.get(&2).unwrap(), "standard_rate");
    }
}
