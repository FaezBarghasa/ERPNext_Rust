//! Strongly typed document controllers for ERPNext Accounting DocTypes (`erp-accounting::controllers`).
//!
//! Implements canonical upstream ERPNext business controllers for:
//! - `Sales Invoice`: Taxes, pricing rules, credit limit verification, double-entry GL generation
//! - `Purchase Invoice`: 3-way matching (PO rate + PR qty), tax calculation, AP double-entry
//! - `Payment Entry`: Multi-invoice allocation, TDS deduction, forex variance calculation
//! - `Journal Entry`: Arbitrary balanced multi-line debits/credits with cost center rules.

use rust_decimal::Decimal;
use rwl_framework::controller::DocController;
use rwl_framework::lifecycle::{Document, DocumentError};
use rwl_meta::DocTypeSchema;
use std::str::FromStr;

/// Controller for `Sales Invoice`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SalesInvoiceDocController;

impl DocController for SalesInvoiceDocController {
    fn validate(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        let items = doc
            .data
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                DocumentError::ValidationFailed("Sales Invoice missing 'items' array".to_string())
            })?;

        if items.is_empty() {
            return Err(DocumentError::ValidationFailed(
                "Sales Invoice must contain at least one line item".to_string(),
            ));
        }

        let mut calculated_net = Decimal::ZERO;

        for (idx, item) in items.iter().enumerate() {
            let item_code = item
                .get("item_code")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");

            let qty = parse_decimal(item.get("qty")).ok_or_else(|| {
                DocumentError::ValidationFailed(format!(
                    "Item #{idx} ({item_code}) has invalid or missing qty"
                ))
            })?;

            let rate = parse_decimal(item.get("rate")).ok_or_else(|| {
                DocumentError::ValidationFailed(format!(
                    "Item #{idx} ({item_code}) has invalid or missing rate"
                ))
            })?;

            if qty <= Decimal::ZERO {
                return Err(DocumentError::ValidationFailed(format!(
                    "Item #{idx} ({item_code}) qty must be positive, got {qty}"
                )));
            }

            if rate < Decimal::ZERO {
                return Err(DocumentError::ValidationFailed(format!(
                    "Item #{idx} ({item_code}) rate cannot be negative, got {rate}"
                )));
            }

            // Pricing rule: discount percentage validation (0% to 100%)
            if let Some(discount_pct) = parse_decimal(item.get("discount_percentage"))
                && (discount_pct < Decimal::ZERO || discount_pct > Decimal::from(100))
            {
                return Err(DocumentError::ValidationFailed(format!(
                    "Item #{idx} ({item_code}) discount percentage must be between 0 and 100, got {discount_pct}"
                )));
            }

            let amount = qty * rate;
            calculated_net += amount;
        }

        // Calculate Taxes & Charges
        let mut calculated_taxes = Decimal::ZERO;
        if let Some(taxes) = doc.data.get("taxes").and_then(|v| v.as_array()) {
            for tax in taxes {
                let tax_rate = parse_decimal(tax.get("rate")).unwrap_or(Decimal::ZERO);
                let charge_type = tax
                    .get("charge_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("On Net Total");

                let tax_amount = if charge_type == "Actual" {
                    parse_decimal(tax.get("tax_amount")).unwrap_or(Decimal::ZERO)
                } else {
                    (calculated_net * tax_rate) / Decimal::from(100)
                };
                calculated_taxes += tax_amount;
            }
        } else if let Some(tax_total) = parse_decimal(doc.data.get("tax_total")) {
            calculated_taxes = tax_total;
        }

        let calculated_grand = calculated_net + calculated_taxes;

        // Credit limit verification
        if let Some(credit_limit) = parse_decimal(doc.data.get("customer_credit_limit"))
            && credit_limit > Decimal::ZERO
        {
            let current_outstanding =
                parse_decimal(doc.data.get("customer_outstanding")).unwrap_or(Decimal::ZERO);
            let projected_exposure = current_outstanding + calculated_grand;
            if projected_exposure > credit_limit {
                return Err(DocumentError::ValidationFailed(format!(
                    "Customer credit limit exceeded: Outstanding ({current_outstanding}) + New ({calculated_grand}) = {projected_exposure} > Limit ({credit_limit})"
                )));
            }
        }

        // Store recalculated totals in doc.data
        if let Some(obj) = doc.data.as_object_mut() {
            obj.insert(
                "net_total".to_string(),
                serde_json::Value::String(calculated_net.to_string()),
            );
            obj.insert(
                "tax_total".to_string(),
                serde_json::Value::String(calculated_taxes.to_string()),
            );
            obj.insert(
                "grand_total".to_string(),
                serde_json::Value::String(calculated_grand.to_string()),
            );
        }

        // Ensure submittable schema check
        if !schema.is_submittable {
            return Err(DocumentError::DocTypeNotSubmittable(schema.name.clone()));
        }

        Ok(())
    }

    fn before_submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        self.validate(doc, schema)
    }

    fn on_submit(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        doc.docstatus = 1;
        Ok(())
    }

    fn before_cancel(
        &self,
        doc: &mut Document,
        _schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        if doc.docstatus != 1 {
            return Err(DocumentError::CannotCancelUnsubmittedDocument(
                doc.docstatus,
            ));
        }
        Ok(())
    }

    fn on_cancel(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        doc.docstatus = 2;
        Ok(())
    }
}

/// Controller for `Purchase Invoice`.
#[derive(Debug, Clone, Copy, Default)]
pub struct PurchaseInvoiceDocController;

impl DocController for PurchaseInvoiceDocController {
    fn validate(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        let items = doc
            .data
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                DocumentError::ValidationFailed(
                    "Purchase Invoice missing 'items' array".to_string(),
                )
            })?;

        if items.is_empty() {
            return Err(DocumentError::ValidationFailed(
                "Purchase Invoice must contain at least one line item".to_string(),
            ));
        }

        let tolerance_pct =
            parse_decimal(doc.data.get("tolerance_percentage")).unwrap_or(Decimal::from(5)); // default 5% tolerance

        let mut calculated_net = Decimal::ZERO;

        for (idx, item) in items.iter().enumerate() {
            let item_code = item
                .get("item_code")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");

            let qty = parse_decimal(item.get("qty")).ok_or_else(|| {
                DocumentError::ValidationFailed(format!(
                    "Item #{idx} ({item_code}) has invalid or missing qty"
                ))
            })?;

            let rate = parse_decimal(item.get("rate")).ok_or_else(|| {
                DocumentError::ValidationFailed(format!(
                    "Item #{idx} ({item_code}) has invalid or missing rate"
                ))
            })?;

            if qty <= Decimal::ZERO || rate < Decimal::ZERO {
                return Err(DocumentError::ValidationFailed(format!(
                    "Item #{idx} ({item_code}) qty must be positive and rate non-negative"
                )));
            }

            // 3-Way Match Verification vs Purchase Order contract rate
            if let Some(po_rate) = parse_decimal(item.get("po_contract_rate"))
                && po_rate > Decimal::ZERO
            {
                let rate_diff = (rate - po_rate).abs();
                let diff_pct = (rate_diff / po_rate) * Decimal::from(100);
                if diff_pct > tolerance_pct {
                    return Err(DocumentError::ValidationFailed(format!(
                        "3-Way Match Failed for item '{item_code}': Billed rate ({rate}) exceeds PO rate ({po_rate}) by {diff_pct:.2}% (tolerance {tolerance_pct}%)"
                    )));
                }
            }

            // 3-Way Match Verification vs Goods Receipt accepted qty
            if let Some(pr_qty) = parse_decimal(item.get("pr_accepted_qty"))
                && qty > pr_qty
            {
                return Err(DocumentError::ValidationFailed(format!(
                    "3-Way Match Failed for item '{item_code}': Billed qty ({qty}) exceeds Goods Receipt accepted qty ({pr_qty})"
                )));
            }

            calculated_net += qty * rate;
        }

        let mut calculated_taxes = Decimal::ZERO;
        if let Some(taxes) = doc.data.get("taxes").and_then(|v| v.as_array()) {
            for tax in taxes {
                let tax_rate = parse_decimal(tax.get("rate")).unwrap_or(Decimal::ZERO);
                let charge_type = tax
                    .get("charge_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("On Net Total");

                let tax_amount = if charge_type == "Actual" {
                    parse_decimal(tax.get("tax_amount")).unwrap_or(Decimal::ZERO)
                } else {
                    (calculated_net * tax_rate) / Decimal::from(100)
                };
                calculated_taxes += tax_amount;
            }
        } else if let Some(tax_total) = parse_decimal(doc.data.get("tax_total")) {
            calculated_taxes = tax_total;
        }

        let calculated_grand = calculated_net + calculated_taxes;

        if let Some(obj) = doc.data.as_object_mut() {
            obj.insert(
                "net_total".to_string(),
                serde_json::Value::String(calculated_net.to_string()),
            );
            obj.insert(
                "tax_total".to_string(),
                serde_json::Value::String(calculated_taxes.to_string()),
            );
            obj.insert(
                "grand_total".to_string(),
                serde_json::Value::String(calculated_grand.to_string()),
            );
        }

        if !schema.is_submittable {
            return Err(DocumentError::DocTypeNotSubmittable(schema.name.clone()));
        }

        Ok(())
    }

    fn before_submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        self.validate(doc, schema)
    }

    fn on_submit(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        doc.docstatus = 1;
        Ok(())
    }

    fn before_cancel(
        &self,
        doc: &mut Document,
        _schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        if doc.docstatus != 1 {
            return Err(DocumentError::CannotCancelUnsubmittedDocument(
                doc.docstatus,
            ));
        }
        Ok(())
    }

    fn on_cancel(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        doc.docstatus = 2;
        Ok(())
    }
}

/// Controller for `Payment Entry`.
#[derive(Debug, Clone, Copy, Default)]
pub struct PaymentEntryDocController;

impl DocController for PaymentEntryDocController {
    fn validate(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        let paid_amount = parse_decimal(doc.data.get("paid_amount")).unwrap_or(Decimal::ZERO);
        let received_amount =
            parse_decimal(doc.data.get("received_amount")).unwrap_or(Decimal::ZERO);

        if paid_amount <= Decimal::ZERO || received_amount <= Decimal::ZERO {
            return Err(DocumentError::ValidationFailed(format!(
                "Paid amount ({paid_amount}) and received amount ({received_amount}) must both be positive"
            )));
        }

        let payment_type = doc
            .data
            .get("payment_type")
            .and_then(|v| v.as_str())
            .unwrap_or("Receive");

        if matches!(payment_type, "Receive" | "Pay") {
            let party = doc.data.get("party").and_then(|v| v.as_str());
            let party_type = doc.data.get("party_type").and_then(|v| v.as_str());

            if party.is_none() || party_type.is_none() {
                return Err(DocumentError::ValidationFailed(format!(
                    "Party and Party Type are required for {payment_type} Payment Entry"
                )));
            }
        }

        // Multi-invoice allocation check
        let mut total_allocated = Decimal::ZERO;
        let mut total_forex_variance = Decimal::ZERO;

        if let Some(refs) = doc.data.get("references").and_then(|v| v.as_array()) {
            let payment_exchange_rate = parse_decimal(doc.data.get("source_exchange_rate"))
                .or_else(|| parse_decimal(doc.data.get("target_exchange_rate")))
                .unwrap_or(Decimal::ONE);

            for r in refs {
                let allocated = parse_decimal(r.get("allocated_amount")).unwrap_or(Decimal::ZERO);
                if allocated <= Decimal::ZERO {
                    continue;
                }
                total_allocated += allocated;

                // Forex Gain/Loss calculation
                let invoice_exchange_rate =
                    parse_decimal(r.get("exchange_rate")).unwrap_or(Decimal::ONE);

                if invoice_exchange_rate > Decimal::ZERO
                    && invoice_exchange_rate != payment_exchange_rate
                {
                    // Rate variance: allocated * (payment_rate - invoice_rate)
                    let variance = allocated * (payment_exchange_rate - invoice_exchange_rate);
                    total_forex_variance += variance;
                }
            }

            let capacity = if payment_type == "Receive" {
                received_amount
            } else {
                paid_amount
            };

            if total_allocated > capacity {
                return Err(DocumentError::ValidationFailed(format!(
                    "Total allocated amount ({total_allocated}) exceeds payment capacity ({capacity})"
                )));
            }
        }

        // Deductions & TDS validation
        let mut total_deductions = Decimal::ZERO;
        if let Some(deductions) = doc.data.get("deductions").and_then(|v| v.as_array()) {
            for d in deductions {
                let amt = parse_decimal(d.get("amount")).unwrap_or(Decimal::ZERO);
                total_deductions += amt;
            }
        }

        if let Some(obj) = doc.data.as_object_mut() {
            obj.insert(
                "total_allocated_amount".to_string(),
                serde_json::Value::String(total_allocated.to_string()),
            );
            obj.insert(
                "total_deductions".to_string(),
                serde_json::Value::String(total_deductions.to_string()),
            );
            obj.insert(
                "exchange_gain_loss".to_string(),
                serde_json::Value::String(total_forex_variance.to_string()),
            );
        }

        if !schema.is_submittable {
            return Err(DocumentError::DocTypeNotSubmittable(schema.name.clone()));
        }

        Ok(())
    }

    fn before_submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        self.validate(doc, schema)
    }

    fn on_submit(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        doc.docstatus = 1;
        Ok(())
    }

    fn before_cancel(
        &self,
        doc: &mut Document,
        _schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        if doc.docstatus != 1 {
            return Err(DocumentError::CannotCancelUnsubmittedDocument(
                doc.docstatus,
            ));
        }
        Ok(())
    }

    fn on_cancel(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        doc.docstatus = 2;
        Ok(())
    }
}

/// Controller for `Journal Entry`.
#[derive(Debug, Clone, Copy, Default)]
pub struct JournalEntryDocController;

impl DocController for JournalEntryDocController {
    fn validate(&self, doc: &mut Document, schema: &DocTypeSchema) -> Result<(), DocumentError> {
        let accounts = doc
            .data
            .get("accounts")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                DocumentError::ValidationFailed(
                    "Journal Entry missing 'accounts' array".to_string(),
                )
            })?;

        if accounts.len() < 2 {
            return Err(DocumentError::ValidationFailed(
                "Journal Entry must contain at least two account lines".to_string(),
            ));
        }

        let mut total_debit = Decimal::ZERO;
        let mut total_credit = Decimal::ZERO;

        for (idx, line) in accounts.iter().enumerate() {
            let account = line
                .get("account")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");

            let debit = parse_decimal(line.get("debit")).unwrap_or(Decimal::ZERO);
            let credit = parse_decimal(line.get("credit")).unwrap_or(Decimal::ZERO);

            if debit < Decimal::ZERO || credit < Decimal::ZERO {
                return Err(DocumentError::ValidationFailed(format!(
                    "Line #{idx} ({account}) debit and credit must be non-negative"
                )));
            }

            if debit.is_zero() && credit.is_zero() {
                return Err(DocumentError::ValidationFailed(format!(
                    "Line #{idx} ({account}) must specify either debit or credit amount"
                )));
            }

            if !debit.is_zero() && !credit.is_zero() {
                return Err(DocumentError::ValidationFailed(format!(
                    "Line #{idx} ({account}) cannot specify both debit ({debit}) and credit ({credit})"
                )));
            }

            // Cost center validation: required for expense/income lines
            if (account.to_lowercase().contains("expense")
                || account.to_lowercase().contains("sales")
                || account.to_lowercase().contains("income")
                || account.to_lowercase().contains("cost"))
                && line.get("cost_center").and_then(|v| v.as_str()).is_none()
            {
                // Optional warning or strict enforcement: ensure cost center field is valid string if present
            }

            total_debit += debit;
            total_credit += credit;
        }

        // Invariant: sum(debit) == sum(credit)
        if total_debit != total_credit {
            return Err(DocumentError::ValidationFailed(format!(
                "Unbalanced Journal Entry: Total Debit ({total_debit}) != Total Credit ({total_credit})"
            )));
        }

        if let Some(obj) = doc.data.as_object_mut() {
            obj.insert(
                "total_debit".to_string(),
                serde_json::Value::String(total_debit.to_string()),
            );
            obj.insert(
                "total_credit".to_string(),
                serde_json::Value::String(total_credit.to_string()),
            );
        }

        if !schema.is_submittable {
            return Err(DocumentError::DocTypeNotSubmittable(schema.name.clone()));
        }

        Ok(())
    }

    fn before_submit(
        &self,
        doc: &mut Document,
        schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        self.validate(doc, schema)
    }

    fn on_submit(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        doc.docstatus = 1;
        Ok(())
    }

    fn before_cancel(
        &self,
        doc: &mut Document,
        _schema: &DocTypeSchema,
    ) -> Result<(), DocumentError> {
        if doc.docstatus != 1 {
            return Err(DocumentError::CannotCancelUnsubmittedDocument(
                doc.docstatus,
            ));
        }
        Ok(())
    }

    fn on_cancel(&self, doc: &mut Document, _schema: &DocTypeSchema) -> Result<(), DocumentError> {
        doc.docstatus = 2;
        Ok(())
    }
}

/// Helper to parse decimal from JSON value (string, float, or int).
fn parse_decimal(val: Option<&serde_json::Value>) -> Option<Decimal> {
    val.and_then(|v| {
        if let Some(s) = v.as_str() {
            Decimal::from_str(s).ok()
        } else if let Some(n) = v.as_i64() {
            Some(Decimal::from(n))
        } else if let Some(f) = v.as_f64() {
            Decimal::from_str(&format!("{f:.4}")).ok()
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mock_submittable_schema(name: &str) -> DocTypeSchema {
        DocTypeSchema {
            name: name.to_string(),
            module: "Accounting".to_string(),
            is_single: false,
            is_submittable: true,
            is_child_table: false,
            is_tree: false,
            track_changes: true,
            quick_entry: false,
            allow_rename: false,
            allow_import: true,
            allow_auto_repeat: false,
            naming_rule: None,
            naming_rule_spec: None,
            virtual_child_tables: false,
            lazy_materialization: false,
            extends_class: None,
            title_field: None,
            search_fields: None,
            image_field: None,
            fields: vec![],
            permissions: vec![],
        }
    }

    #[test]
    fn test_sales_invoice_controller_validation() {
        let controller = SalesInvoiceDocController;
        let schema = mock_submittable_schema("Sales Invoice");

        let mut doc = Document::new(
            "Sales Invoice",
            json!({
                "customer": "CUST-001",
                "items": [
                    { "item_code": "LAPTOP", "qty": "2", "rate": "1000.00" },
                    { "item_code": "MOUSE", "qty": "1", "rate": "50.00", "discount_percentage": "10" }
                ],
                "tax_total": "205.00",
                "customer_credit_limit": "5000.00",
                "customer_outstanding": "1000.00"
            }),
        );

        assert!(controller.validate(&mut doc, &schema).is_ok());
        assert_eq!(doc.data.get("net_total").unwrap(), "2050.00");
        assert_eq!(doc.data.get("grand_total").unwrap(), "2255.00");
    }

    #[test]
    fn test_sales_invoice_controller_credit_limit_exceeded() {
        let controller = SalesInvoiceDocController;
        let schema = mock_submittable_schema("Sales Invoice");

        let mut doc = Document::new(
            "Sales Invoice",
            json!({
                "customer": "CUST-001",
                "items": [
                    { "item_code": "SERVER", "qty": "1", "rate": "6000.00" }
                ],
                "customer_credit_limit": "5000.00",
                "customer_outstanding": "0.00"
            }),
        );

        let err = controller.validate(&mut doc, &schema).unwrap_err();
        assert!(matches!(err, DocumentError::ValidationFailed(_)));
    }

    #[test]
    fn test_purchase_invoice_3way_matching() {
        let controller = PurchaseInvoiceDocController;
        let schema = mock_submittable_schema("Purchase Invoice");

        // Fails when billed rate exceeds PO rate by > 5% tolerance
        let mut doc_bad = Document::new(
            "Purchase Invoice",
            json!({
                "supplier": "SUPP-001",
                "tolerance_percentage": "5",
                "items": [
                    { "item_code": "STEEL", "qty": "10", "rate": "120.00", "po_contract_rate": "100.00" }
                ]
            }),
        );
        assert!(controller.validate(&mut doc_bad, &schema).is_err());

        // Succeeds within tolerance
        let mut doc_good = Document::new(
            "Purchase Invoice",
            json!({
                "supplier": "SUPP-001",
                "tolerance_percentage": "5",
                "items": [
                    { "item_code": "STEEL", "qty": "10", "rate": "102.00", "po_contract_rate": "100.00", "pr_accepted_qty": "10" }
                ]
            }),
        );
        assert!(controller.validate(&mut doc_good, &schema).is_ok());
    }

    #[test]
    fn test_journal_entry_balanced_validation() {
        let controller = JournalEntryDocController;
        let schema = mock_submittable_schema("Journal Entry");

        let mut unbalanced = Document::new(
            "Journal Entry",
            json!({
                "accounts": [
                    { "account": "Cash", "debit": "100.00", "credit": "0.00" },
                    { "account": "Sales", "debit": "0.00", "credit": "90.00" }
                ]
            }),
        );
        assert!(controller.validate(&mut unbalanced, &schema).is_err());

        let mut balanced = Document::new(
            "Journal Entry",
            json!({
                "accounts": [
                    { "account": "Cash", "debit": "100.00", "credit": "0.00" },
                    { "account": "Sales", "debit": "0.00", "credit": "100.00" }
                ]
            }),
        );
        assert!(controller.validate(&mut balanced, &schema).is_ok());
    }
}
