use axum::http::StatusCode;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateEmployeeRequest, Employee, EmploymentPeriod, RehireEmployeeRequest, UpdateEmployeeRequest,
};
use crate::repositories::employee_repository;
use crate::repositories::user_repository;

pub struct EmployeeService;

impl EmployeeService {
    pub async fn create_employee(
        pool: &PgPool,
        workspace_id: Uuid,
        req: CreateEmployeeRequest,
    ) -> Result<Employee, StatusCode> {
        // Optional: Check if user_id exists if provided
        if let Some(user_id) = req.user_id {
            user_repository::get_user_by_id(pool, user_id)
                .await
                .map_err(|e| {
                    eprintln!("Error checking user_id for new employee: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
                .ok_or(StatusCode::BAD_REQUEST)?; // User not found
        }

        let employee = Employee {
            id: Uuid::new_v4(),
            workspace_id,
            user_id: req.user_id,
            full_name: req.full_name,
            employee_id_number: req.employee_id_number,
            hire_date: req.hire_date,
            termination_date: None,
            tags: req.tags.unwrap_or_default(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut tx = pool
            .begin()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        employee_repository::create_employee_tx(&mut tx, &employee)
            .await
            .map_err(|e| {
                eprintln!("Error creating employee: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        tx.commit()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(employee)
    }

    pub async fn rehire_employee(
        pool: &PgPool,
        employee_id: Uuid,
        workspace_id: Uuid,
        req: RehireEmployeeRequest,
    ) -> Result<Employee, StatusCode> {
        // Verify employee belongs to workspace and is currently inactive
        let employee = employee_repository::get_employee_by_id(pool, employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        if employee.is_active {
            return Err(StatusCode::BAD_REQUEST); // Already active
        }

        // Check if new hire_date is after last termination_date
        if let Some(term_date) = employee.termination_date {
            if req.hire_date <= term_date {
                return Err(StatusCode::BAD_REQUEST);
            }
        }

        let tx = pool
            .begin()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Update employee to be active
        employee_repository::update_employee(
            pool,
            employee_id,
            workspace_id,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Create new employment period
        let period = EmploymentPeriod {
            id: Uuid::new_v4(),
            employee_id,
            hire_date: req.hire_date,
            termination_date: None,
            is_rehire: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        employee_repository::create_employment_period(pool, &period)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        tx.commit()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        employee_repository::get_employee_by_id(pool, employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)
    }

    pub async fn get_employment_history(
        pool: &PgPool,
        employee_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Vec<EmploymentPeriod>, StatusCode> {
        // Verify employee belongs to workspace
        let _ = employee_repository::get_employee_by_id(pool, employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        employee_repository::get_employment_history(pool, employee_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn get_employee_by_id(
        pool: &PgPool,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Employee, StatusCode> {
        employee_repository::get_employee_by_id(pool, id, workspace_id)
            .await
            .map_err(|e| {
                eprintln!("Error getting employee: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::NOT_FOUND)
    }

    pub async fn get_all_employees(
        pool: &PgPool,
        workspace_id: Uuid,
        search_query: Option<String>,
    ) -> Result<Vec<Employee>, StatusCode> {
        employee_repository::get_all_employees(pool, workspace_id, search_query)
            .await
            .map_err(|e| {
                eprintln!("Error getting all employees: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    pub async fn update_employee(
        pool: &PgPool,
        id: Uuid,
        workspace_id: Uuid,
        req: UpdateEmployeeRequest,
    ) -> Result<Employee, StatusCode> {
        // Optional: Check if user_id exists if provided
        if let Some(user_id) = req.user_id {
            user_repository::get_user_by_id(pool, user_id)
                .await
                .map_err(|e| {
                    eprintln!("Error checking user_id for update employee: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
                .ok_or(StatusCode::BAD_REQUEST)?; // User not found
        }

        employee_repository::update_employee(
            pool,
            id,
            workspace_id,
            req.user_id,
            req.full_name,
            req.employee_id_number,
            req.hire_date,
            req.termination_date,
            req.tags,
            req.is_active,
        )
        .await
        .map_err(|e| {
            eprintln!("Error updating employee: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)
    }

    pub async fn delete_employee(
        pool: &PgPool,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<(), StatusCode> {
        let rows_affected = employee_repository::delete_employee(pool, id, workspace_id)
            .await
            .map_err(|e| {
                eprintln!("Error deleting employee: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if rows_affected == 0 {
            Err(StatusCode::NOT_FOUND)
        } else {
            Ok(())
        }
    }

    pub async fn get_department_history(
        pool: &PgPool,
        employee_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeeDepartmentHistory>, StatusCode> {
        // Verify employee belongs to workspace
        let _ = employee_repository::get_employee_by_id(pool, employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        crate::repositories::department_repository::get_employee_department_history(
            pool,
            employee_id,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn transfer_employee(
        pool: &PgPool,
        employee_id: Uuid,
        workspace_id: Uuid,
        req: crate::models::TransferEmployeeRequest,
    ) -> Result<crate::models::EmployeeDepartmentHistory, StatusCode> {
        // Verify employee belongs to workspace
        let _ = employee_repository::get_employee_by_id(pool, employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        crate::repositories::department_repository::transfer_employee(
            pool,
            employee_id,
            req.department_id,
            req.effective_date,
            req.notes,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
