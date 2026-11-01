use proptest::prelude::*;
use rwl_net::compile_filters_to_surrealql;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_filters_never_allow_unescaped_quotes_or_breakout(
        field in "\\PC*",
        op in "\\PC*",
        val in "\\PC*"
    ) {
        let filter_json = serde_json::to_string(&vec![vec![field, op, val]]).unwrap();
        if let Some(sql) = compile_filters_to_surrealql(&filter_json) {
            // If the filter compiled:
            // 1. It must not contain raw unescaped quotes inside the string literal
            // 2. It must have matching double quotes at the boundary of value
            let quote_count = sql.chars().filter(|&c| c == '"').count();
            // Every valid clause must have an even number of double quotes (enclosing the escaped string)
            prop_assert!(
                quote_count % 2 == 0,
                "Odd number of quotes in compiled SQL: {}",
                sql
            );

            // 3. Must not end with comments or dangerous multi-statement delimiters
            prop_assert!(!sql.contains(";"));
            prop_assert!(!sql.contains("--"));
            prop_assert!(!sql.contains("/*"));
        }
    }

    #[test]
    fn prop_injection_payloads_safely_neutralized(
        injection in "(; DROP TABLE [a-z]+;?|' OR '1'='1|\" OR \"1\"=\"1|--|/\\*.*\\*/)"
    ) {
        // Field injection test: must be completely rejected
        let field_attack = serde_json::to_string(&vec![vec![format!("field_{injection}"), "=".into(), "val".into()]]).unwrap();
        let res = compile_filters_to_surrealql(&field_attack);
        prop_assert!(res.is_none(), "Field injection should have been rejected: {:?}", res);

        // Operator injection test: must be completely rejected
        let op_attack = serde_json::to_string(&vec![vec!["status".into(), format!("={injection}"), "val".into()]]).unwrap();
        let res_op = compile_filters_to_surrealql(&op_attack);
        prop_assert!(res_op.is_none(), "Operator injection should have been rejected: {:?}", res_op);

        // Value injection test: verify it is safely escaped inside double quotes and cannot break out
        let val_attack = serde_json::to_string(&vec![vec!["status".into(), "=".into(), injection]]).unwrap();
        if let Some(sql) = compile_filters_to_surrealql(&val_attack) {
            // Must start with "status = \"" and end with "\""
            prop_assert!(sql.starts_with("status = \""));
            prop_assert!(sql.ends_with("\""));
            // The unescaped quote count must be exactly 2 (the opening and closing quotes)
            let unescaped_quotes = sql
                .replace("\\\"", "")
                .chars()
                .filter(|&c| c == '"')
                .count();
            prop_assert_eq!(
                unescaped_quotes, 2,
                "Value injection broke out of quoted literal: {}",
                sql
            );
        }
    }
}
