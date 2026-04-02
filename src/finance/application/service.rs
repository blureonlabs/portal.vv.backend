use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::common::types::Role;
use crate::finance::domain::{
    entity::{CashHandover, CreateExpense, CreateHandover, Expense, ExpenseCategory},
    repository::FinanceRepository,
};

pub struct FinanceService {
    repo: Arc<dyn FinanceRepository>,
}

impl FinanceService {
    pub fn new(repo: Arc<dyn FinanceRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_expenses(
        &self,
        actor_role: &Role,
        actor_driver_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Expense>, AppError> {
        let effective = if *actor_role == Role::Driver {
            actor_driver_id
        } else {
            driver_id
        };
        self.repo.list_expenses(effective, from, to).await
    }

    pub async fn create_expense(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        driver_id: Option<Uuid>,
        amount_aed: Decimal,
        category: ExpenseCategory,
        date: NaiveDate,
        receipt_url: Option<String>,
        notes: Option<String>,
    ) -> Result<Expense, AppError> {
        match actor_role {
            Role::SuperAdmin | Role::Accountant => {}
            _ => return Err(AppError::Forbidden("Only super_admin or accountant can log expenses".into())),
        }
        self.repo
            .create_expense(CreateExpense {
                driver_id,
                entered_by: actor_id,
                amount_aed,
                category,
                date,
                receipt_url,
                notes,
            })
            .await
    }

    pub async fn list_handovers(
        &self,
        actor_role: &Role,
        actor_driver_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<CashHandover>, AppError> {
        let effective = if *actor_role == Role::Driver {
            actor_driver_id
        } else {
            driver_id
        };
        self.repo.list_handovers(effective, from, to).await
    }

    pub async fn create_handover(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        driver_id: Uuid,
        amount_aed: Decimal,
    ) -> Result<CashHandover, AppError> {
        match actor_role {
            Role::SuperAdmin | Role::Accountant => {}
            _ => return Err(AppError::Forbidden("Only super_admin or accountant can record handovers".into())),
        }
        self.repo
            .create_handover(CreateHandover {
                driver_id,
                amount_aed,
                verified_by: actor_id,
            })
            .await
    }
}
