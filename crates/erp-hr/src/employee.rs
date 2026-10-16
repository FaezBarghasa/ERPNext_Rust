//! Upstream ERPNext Employee Master, Department & Designation Engine (`erp-hr::employee`).
//!
//! Provides canonical ERPNext HR capabilities:
//! - Employee Master lifecycle: Active -> Suspended -> Left
//! - Employment Types: FullTime, PartTime, Contract, Intern, Commission
//! - Department hierarchy and Designation organizational mapping
//! - Employee Onboarding & Separation workflows.

use chrono::NaiveDate;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Employee operational errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EmployeeError {
    #[error("Employee '{0}' has already left the organization on {1}")]
    AlreadyLeft(CompactString, NaiveDate),
    #[error("Relieving date {relieving} cannot be prior to joining date {joining}")]
    InvalidRelievingDate {
        joining: NaiveDate,
        relieving: NaiveDate,
    },
    #[error("Employee '{0}' is suspended")]
    EmployeeSuspended(CompactString),
}

/// Employment contract type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EmploymentType {
    #[default]
    FullTime,
    PartTime,
    Contract,
    Intern,
    Commission,
}

/// Employee employment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EmployeeStatus {
    #[default]
    Active,
    Suspended,
    Left,
}

/// Department in organizational tree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Department {
    pub name: CompactString,
    pub department_name: CompactString,
    pub parent_department: Option<CompactString>,
    pub is_group: bool,
}

impl Department {
    pub fn new(
        name: impl Into<CompactString>,
        department_name: impl Into<CompactString>,
        parent_department: Option<impl Into<CompactString>>,
    ) -> Self {
        Self {
            name: name.into(),
            department_name: department_name.into(),
            parent_department: parent_department.map(Into::into),
            is_group: false,
        }
    }
}

/// Designation / Job Role.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Designation {
    pub name: CompactString,
    pub designation_name: CompactString,
    pub description: Option<CompactString>,
}

impl Designation {
    pub fn new(name: impl Into<CompactString>, designation_name: impl Into<CompactString>) -> Self {
        Self {
            name: name.into(),
            designation_name: designation_name.into(),
            description: None,
        }
    }
}

/// Canonical ERPNext Employee record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Employee {
    pub name: CompactString,
    pub first_name: CompactString,
    pub last_name: CompactString,
    pub employee_name: CompactString,
    pub department: CompactString,
    pub designation: CompactString,
    pub employment_type: EmploymentType,
    pub status: EmployeeStatus,
    pub date_of_joining: NaiveDate,
    pub date_of_birth: Option<NaiveDate>,
    pub relieving_date: Option<NaiveDate>,
    pub reports_to: Option<CompactString>,
    pub bank_name: Option<CompactString>,
    pub bank_account_no: Option<CompactString>,
    pub salary_structure: Option<CompactString>,
    pub leave_approver: Option<CompactString>,
}

impl Employee {
    pub fn new(
        name: impl Into<CompactString>,
        first_name: impl Into<CompactString>,
        last_name: impl Into<CompactString>,
        department: impl Into<CompactString>,
        designation: impl Into<CompactString>,
        date_of_joining: NaiveDate,
        employment_type: EmploymentType,
    ) -> Self {
        let f: CompactString = first_name.into();
        let l: CompactString = last_name.into();
        let full_name: CompactString = format!("{f} {l}").into();

        Self {
            name: name.into(),
            first_name: f,
            last_name: l,
            employee_name: full_name,
            department: department.into(),
            designation: designation.into(),
            employment_type,
            status: EmployeeStatus::Active,
            date_of_joining,
            date_of_birth: None,
            relieving_date: None,
            reports_to: None,
            bank_name: None,
            bank_account_no: None,
            salary_structure: None,
            leave_approver: None,
        }
    }

    /// Suspends employee pending review.
    pub fn suspend(&mut self) {
        self.status = EmployeeStatus::Suspended;
    }

    /// Reinstates suspended employee to Active.
    pub fn reinstate(&mut self) {
        self.status = EmployeeStatus::Active;
    }

    /// Executes employee separation / relieving.
    pub fn separate(&mut self, relieving_date: NaiveDate) -> Result<(), EmployeeError> {
        if relieving_date < self.date_of_joining {
            return Err(EmployeeError::InvalidRelievingDate {
                joining: self.date_of_joining,
                relieving: relieving_date,
            });
        }
        if self.status == EmployeeStatus::Left {
            return Err(EmployeeError::AlreadyLeft(
                self.name.clone(),
                self.relieving_date.unwrap_or(relieving_date),
            ));
        }

        self.relieving_date = Some(relieving_date);
        self.status = EmployeeStatus::Left;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_employee_onboarding_and_separation_lifecycle() {
        let join_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let mut emp = Employee::new(
            "HR-EMP-00001",
            "Alex",
            "Vance",
            "Firmware Engineering",
            "Senior Systems Engineer",
            join_date,
            EmploymentType::FullTime,
        );

        assert_eq!(emp.employee_name, "Alex Vance");
        assert_eq!(emp.status, EmployeeStatus::Active);

        // Suspend & reinstate
        emp.suspend();
        assert_eq!(emp.status, EmployeeStatus::Suspended);
        emp.reinstate();
        assert_eq!(emp.status, EmployeeStatus::Active);

        // Relieving prior to join date -> Error
        let bad_date = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        assert!(matches!(
            emp.separate(bad_date),
            Err(EmployeeError::InvalidRelievingDate { .. })
        ));

        // Proper separation
        let leave_date = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
        emp.separate(leave_date).unwrap();
        assert_eq!(emp.status, EmployeeStatus::Left);
        assert_eq!(emp.relieving_date, Some(leave_date));

        // Second separation -> Error
        assert!(matches!(
            emp.separate(leave_date),
            Err(EmployeeError::AlreadyLeft(_, _))
        ));
    }
}
