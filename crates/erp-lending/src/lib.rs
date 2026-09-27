pub mod amortization;

pub use amortization::{
    AmortizationEngine, AmortizationPeriod, LendingError, LoanGlEngine,
};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn test_loan_amortization_schedule_zero_termination() {
        let principal = dec!(10000.00);
        let annual_rate = dec!(12.0); // 12% annual rate = 1% per month
        let periods = 12;
        let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let schedule = AmortizationEngine::generate_schedule(
            principal,
            annual_rate,
            periods,
            start_date,
        )
        .expect("Schedule generation failed");

        assert_eq!(schedule.len(), 12);

        // Sum of all principal portions must equal the original principal exactly
        let total_principal_paid: Decimal = schedule.iter().map(|p| p.principal_portion).sum();
        assert_eq!(total_principal_paid, principal);

        // The final period's remaining balance must be exactly 0.00
        let last_period = schedule.last().unwrap();
        assert_eq!(last_period.remaining_balance, dec!(0.00));
    }

    #[test]
    fn test_loan_gl_repayment_journal_balance() {
        let date = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let entries = LoanGlEngine::create_repayment_gl_entries(
            "1110 - Bank Account",
            "1250 - Loan Principal Receivable",
            "1260 - Interest Receivable",
            dec!(800.00),
            dec!(100.00),
            date,
            "LN-REP-001",
            "Acme Corp",
        );

        assert_eq!(entries.len(), 3);
        let jv = LoanGlEngine::repayment_to_journal_entry(&entries, date, "Acme Corp", "Month 1 Loan Repayment");
        assert!(jv.validate_balance().is_ok());
    }
}
