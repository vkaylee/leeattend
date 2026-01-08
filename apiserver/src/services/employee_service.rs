use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use axum::http::StatusCode;

use crate::models::{Employee, CreateEmployeeRequest, UpdateEmployeeRequest};
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
            first_name: req.first_name,
            last_name: req.last_name,
            employee_id_number: req.employee_id_number,
            hire_date: req.hire_date,
            termination_date: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        employee_repository::create_employee(pool, &employee)
            .await
            .map_err(|e| {
                eprintln!("Error creating employee: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        Ok(employee)
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
    ) -> Result<Vec<Employee>, StatusCode> {
        employee_repository::get_all_employees(pool, workspace_id)
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
            req.first_name,
            req.last_name,
            req.employee_id_number,
            req.hire_date,
            req.termination_date,
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
}
