mod tests {
    use super::super::{ApprovalWorkflowService, WorkflowCallbackHandler};
    use crate::diesel_pool::TenantAuthToken;
    use crate::errors::AppError;
    use crate::models::{
        ApprovalActionLog, ApprovalWorkflow, ApprovalWorkflowInstance, ApprovalWorkflowStep,
        ApprovalWorkflowTask, Employee, LeaveStatus, WorkflowInstanceStepSnapshot,
        WorkspaceConfigEntry,
    };
    use crate::repositories::{
        department_repository::DepartmentRepository,
        leave_request_repository::LeaveRequestRepository,
        overtime_request_repository::OvertimeRequestRepository, ApprovalWorkflowRepository,
        WorkspaceConfigRepository, WorkspaceRepository,
    };
    use async_trait::async_trait;
    use chrono::{DateTime, NaiveDate, Utc};
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::Mutex;
    use uuid::Uuid;

    struct MockState {
        workflows: HashMap<Uuid, ApprovalWorkflow>,
        steps: HashMap<Uuid, Vec<ApprovalWorkflowStep>>,
        active_workflows: HashMap<String, ApprovalWorkflow>,
        employees: HashMap<Uuid, Employee>,
        departments: HashMap<Uuid, crate::models::Department>,
        employee_departments: HashMap<Uuid, crate::models::EmployeeDepartmentHistory>,
        configs: HashMap<String, WorkspaceConfigEntry>,
        admins: Vec<Uuid>,
        members: Vec<crate::models::WorkspaceMemberDetail>,
        instances: HashMap<Uuid, ApprovalWorkflowInstance>,
        action_logs: Vec<ApprovalActionLog>,
        leave_statuses: HashMap<Uuid, LeaveStatus>,
        ot_requests: HashMap<Uuid, crate::models::OvertimeRequest>,
        tasks: HashMap<Uuid, ApprovalWorkflowTask>,
        leave_requests: HashMap<Uuid, crate::models::LeaveRequest>,
    }

    impl MockState {
        fn new() -> Self {
            Self {
                workflows: HashMap::new(),
                steps: HashMap::new(),
                active_workflows: HashMap::new(),
                employees: HashMap::new(),
                departments: HashMap::new(),
                employee_departments: HashMap::new(),
                configs: HashMap::new(),
                admins: Vec::new(),
                members: Vec::new(),
                instances: HashMap::new(),
                action_logs: Vec::new(),
                leave_statuses: HashMap::new(),
                ot_requests: HashMap::new(),
                tasks: HashMap::new(),
                leave_requests: HashMap::new(),
            }
        }
    }

    struct MockApprovalWorkflowRepository {
        pub create_workflow_called: std::sync::atomic::AtomicBool,
        pub state: Arc<Mutex<MockState>>,
    }

    #[async_trait]
    impl ApprovalWorkflowRepository for MockApprovalWorkflowRepository {
        async fn create_workflow(
            &self,
            _auth_token: &TenantAuthToken,
            workflow: ApprovalWorkflow,
        ) -> Result<ApprovalWorkflow, AppError> {
            self.create_workflow_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            let mut state = self.state.lock().unwrap();
            state.workflows.insert(workflow.id, workflow.clone());
            Ok(workflow)
        }

        async fn create_steps(
            &self,
            _auth_token: &TenantAuthToken,
            steps: Vec<ApprovalWorkflowStep>,
        ) -> Result<Vec<ApprovalWorkflowStep>, AppError> {
            if !steps.is_empty() {
                let workflow_id = steps[0].workflow_id;
                let mut state = self.state.lock().unwrap();
                state.steps.insert(workflow_id, steps.clone());
            }
            Ok(steps)
        }

        async fn get_active_workflow(
            &self,
            _auth_token: &TenantAuthToken,
            _workspace_id: Uuid,
            entity_type: &str,
        ) -> Result<Option<ApprovalWorkflow>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.active_workflows.get(entity_type).cloned())
        }

        async fn get_steps(
            &self,
            _auth_token: &TenantAuthToken,
            workflow_id: Uuid,
        ) -> Result<Vec<ApprovalWorkflowStep>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.steps.get(&workflow_id).cloned().unwrap_or_default())
        }

        async fn log_action(
            &self,
            _auth_token: &TenantAuthToken,
            log: ApprovalActionLog,
        ) -> Result<ApprovalActionLog, AppError> {
            let mut state = self.state.lock().unwrap();
            state.action_logs.push(log.clone());
            Ok(log)
        }

        async fn get_action_logs(
            &self,
            _auth_token: &TenantAuthToken,
            entity_id: Uuid,
        ) -> Result<Vec<ApprovalActionLog>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state
                .action_logs
                .iter()
                .filter(|l| l.entity_id == entity_id)
                .cloned()
                .collect())
        }

        async fn create_instance(
            &self,
            _auth_token: &TenantAuthToken,
            instance: ApprovalWorkflowInstance,
        ) -> Result<ApprovalWorkflowInstance, AppError> {
            let mut state = self.state.lock().unwrap();
            state.instances.insert(instance.id, instance.clone());
            Ok(instance)
        }

        async fn update_instance(
            &self,
            _auth_token: &TenantAuthToken,
            instance: ApprovalWorkflowInstance,
        ) -> Result<ApprovalWorkflowInstance, AppError> {
            let mut state = self.state.lock().unwrap();
            state.instances.insert(instance.id, instance.clone());
            Ok(instance)
        }

        async fn get_instance_by_entity(
            &self,
            _auth_token: &TenantAuthToken,
            entity_type: &str,
            entity_id: Uuid,
        ) -> Result<Option<ApprovalWorkflowInstance>, AppError> {
            let state = self.state.lock().unwrap();
            let instance = state
                .instances
                .values()
                .find(|i| i.entity_type == entity_type && i.entity_id == entity_id)
                .cloned();
            Ok(instance)
        }

        async fn get_instance_by_id(
            &self,
            _auth_token: &TenantAuthToken,
            instance_id: Uuid,
        ) -> Result<Option<ApprovalWorkflowInstance>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.instances.get(&instance_id).cloned())
        }

        async fn get_workflow_by_id(
            &self,
            _auth_token: &TenantAuthToken,
            workflow_id: Uuid,
        ) -> Result<Option<ApprovalWorkflow>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.workflows.get(&workflow_id).cloned())
        }

        async fn list_workflows(
            &self,
            _auth_token: &TenantAuthToken,
            _workspace_id: Uuid,
            entity_type: Option<String>,
            page: i64,
            limit: i64,
        ) -> Result<(Vec<ApprovalWorkflow>, i64), AppError> {
            let state = self.state.lock().unwrap();
            let mut list: Vec<ApprovalWorkflow> = state.workflows.values().cloned().collect();
            if let Some(et) = entity_type {
                list.retain(|w| w.entity_type == et);
            }
            let total = list.len() as i64;
            let start = ((page - 1) * limit) as usize;
            let end = (start + limit as usize).min(list.len());
            let paginated = if start < list.len() {
                list[start..end].to_vec()
            } else {
                vec![]
            };
            Ok((paginated, total))
        }

        async fn update_workflow(
            &self,
            _auth_token: &TenantAuthToken,
            workflow: ApprovalWorkflow,
        ) -> Result<ApprovalWorkflow, AppError> {
            let mut state = self.state.lock().unwrap();
            state.workflows.insert(workflow.id, workflow.clone());
            Ok(workflow)
        }

        async fn delete_workflow(
            &self,
            _auth_token: &TenantAuthToken,
            workflow_id: Uuid,
        ) -> Result<(), AppError> {
            let mut state = self.state.lock().unwrap();
            state.workflows.remove(&workflow_id);
            Ok(())
        }

        async fn delete_steps_by_workflow_id(
            &self,
            _auth_token: &TenantAuthToken,
            workflow_id: Uuid,
        ) -> Result<(), AppError> {
            let mut state = self.state.lock().unwrap();
            state.steps.remove(&workflow_id);
            Ok(())
        }

        async fn get_steps_for_workflows(
            &self,
            _auth_token: &TenantAuthToken,
            workflow_ids: Vec<Uuid>,
        ) -> Result<Vec<ApprovalWorkflowStep>, AppError> {
            let state = self.state.lock().unwrap();
            let mut steps = Vec::new();
            for wid in workflow_ids {
                if let Some(s) = state.steps.get(&wid) {
                    steps.extend(s.clone());
                }
            }
            Ok(steps)
        }

        async fn create_tasks(
            &self,
            _auth_token: &TenantAuthToken,
            tasks: Vec<ApprovalWorkflowTask>,
        ) -> Result<Vec<ApprovalWorkflowTask>, AppError> {
            let mut state = self.state.lock().unwrap();
            for task in &tasks {
                state.tasks.insert(task.id(), task.clone());
            }
            Ok(tasks)
        }

        async fn update_task(
            &self,
            _auth_token: &TenantAuthToken,
            task: ApprovalWorkflowTask,
        ) -> Result<ApprovalWorkflowTask, AppError> {
            let mut state = self.state.lock().unwrap();
            state.tasks.insert(task.id(), task.clone());
            Ok(task)
        }

        async fn get_active_tasks_for_instance(
            &self,
            _auth_token: &TenantAuthToken,
            instance_id: Uuid,
        ) -> Result<Vec<ApprovalWorkflowTask>, AppError> {
            let state = self.state.lock().unwrap();
            let mut res = Vec::new();
            for task in state.tasks.values() {
                if task.instance_id() == instance_id && task.status() == "pending" {
                    res.push(task.clone());
                }
            }
            Ok(res)
        }

        async fn get_pending_tasks_for_user(
            &self,
            _auth_token: &TenantAuthToken,
            user_id: Uuid,
            role_ids: Vec<Uuid>,
            permissions: Vec<String>,
            managed_dept_ids: Vec<Uuid>,
        ) -> Result<Vec<ApprovalWorkflowTask>, AppError> {
            let state = self.state.lock().unwrap();
            let mut res = Vec::new();
            for task in state.tasks.values() {
                if task.status() != "pending" {
                    continue;
                }
                match task.assignee_type() {
                    "specific_user" => {
                        if task.assignee_target_id() == Some(user_id) {
                            res.push(task.clone());
                        }
                    }
                    "role" => {
                        if let Some(target_id) = task.assignee_target_id() {
                            if role_ids.contains(&target_id) {
                                res.push(task.clone());
                            }
                        }
                    }
                    "permission" => {
                        if let Some(target_name) = task.assignee_target_name() {
                            if permissions.contains(&target_name.to_string()) {
                                res.push(task.clone());
                            }
                        }
                    }
                    "department_manager" => {
                        if let Some(target_id) = task.assignee_target_id() {
                            if managed_dept_ids.contains(&target_id) {
                                res.push(task.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(res)
        }
    }

    struct MockWorkspaceConfigRepository {
        pub state: Arc<Mutex<MockState>>,
    }

    #[async_trait]
    impl WorkspaceConfigRepository for MockWorkspaceConfigRepository {
        async fn create_entry(
            &self,
            _a: &TenantAuthToken,
            e: WorkspaceConfigEntry,
        ) -> Result<WorkspaceConfigEntry, AppError> {
            let mut state = self.state.lock().unwrap();
            state.configs.insert(e.config_key.clone(), e.clone());
            Ok(e)
        }
        async fn get_effective_config(
            &self,
            _a: &TenantAuthToken,
            _w: Uuid,
            k: &str,
            _s: &str,
            _id: Option<Uuid>,
            _d: NaiveDate,
        ) -> Result<Option<WorkspaceConfigEntry>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.configs.get(k).cloned())
        }
        async fn list_entries(
            &self,
            _a: &TenantAuthToken,
            _w: Uuid,
            _k: Option<&str>,
        ) -> Result<Vec<WorkspaceConfigEntry>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.configs.values().cloned().collect())
        }
        async fn delete_entry(&self, _a: &TenantAuthToken, _id: Uuid) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct MockWorkspaceRepository {
        pub state: Arc<Mutex<MockState>>,
    }

    #[async_trait]
    impl WorkspaceRepository for MockWorkspaceRepository {
        async fn get_admins_with_permissions(
            &self,
            _w: Uuid,
            _p: &[&str],
        ) -> Result<Vec<Uuid>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.admins.clone())
        }
        async fn get_workspace_members(
            &self,
            _w: Uuid,
        ) -> Result<Vec<crate::models::WorkspaceMemberDetail>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.members.clone())
        }
        async fn create_workspace_with_owner(
            &self,
            _id: Uuid,
            name: &str,
            _owner_id: Uuid,
            _custom_db_url_encrypted: Option<String>,
        ) -> Result<crate::models::Workspace, AppError> {
            Ok(crate::models::Workspace::new(
                _id,
                name.to_string(),
                "YYYY-MM-DD".to_string(),
                "HH:mm:ss".to_string(),
                "UTC".to_string(),
                "#000000".to_string(),
                "en".to_string(),
                None,
                None,
                "standard".to_string(),
                rust_decimal::Decimal::from(1),
                Utc::now(),
                None,
            ))
        }
        async fn get_workspace_by_id(
            &self,
            _id: Uuid,
        ) -> Result<Option<crate::models::Workspace>, AppError> {
            Ok(None)
        }
        async fn get_all_workspaces(&self) -> Result<Vec<crate::models::Workspace>, AppError> {
            Ok(vec![])
        }
        async fn get_user_workspaces(
            &self,
            _user_id: Uuid,
        ) -> Result<Vec<crate::models::Workspace>, AppError> {
            Ok(vec![])
        }
        async fn clear_workspace_data(
            &self,
            _auth_token: &TenantAuthToken,
            _group: &str,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn add_member(&self, _w: Uuid, _u: Uuid, _r: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn update_member_role(&self, _w: Uuid, _u: Uuid, _r: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn remove_member(&self, _w: Uuid, _u: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn find_primary_role(&self, _w: Uuid, _u: Uuid) -> Result<Option<Uuid>, AppError> {
            Ok(None)
        }
        async fn get_all_extra_roles(
            &self,
            _w: Uuid,
        ) -> Result<Vec<crate::models::MemberExtraRole>, AppError> {
            Ok(vec![])
        }
        async fn find_extra_roles(
            &self,
            _w: Uuid,
            _u: Uuid,
        ) -> Result<Vec<crate::models::MemberExtraRole>, AppError> {
            Ok(vec![])
        }
        async fn find_extra_permissions(
            &self,
            _w: Uuid,
            _u: Uuid,
        ) -> Result<Vec<String>, AppError> {
            Ok(vec![])
        }
        async fn get_member_extra_permissions(
            &self,
            _w: Uuid,
            _u: Uuid,
        ) -> Result<Vec<crate::models::MemberExtraPermission>, AppError> {
            Ok(vec![])
        }
        async fn assign_extra_permission(
            &self,
            _w: Uuid,
            _u: Uuid,
            _p: String,
            _e: DateTime<Utc>,
            _a: Uuid,
        ) -> Result<crate::models::MemberExtraPermission, AppError> {
            unimplemented!()
        }
        async fn revoke_extra_permission(
            &self,
            _w: Uuid,
            _u: Uuid,
            _p: Uuid,
        ) -> Result<u64, AppError> {
            Ok(0)
        }
        async fn assign_extra_role(
            &self,
            _w: Uuid,
            _u: Uuid,
            _r: Uuid,
            _e: DateTime<Utc>,
        ) -> Result<crate::models::MemberExtraRole, AppError> {
            unimplemented!()
        }
        async fn revoke_extra_role(&self, _w: Uuid, _r: Uuid) -> Result<u64, AppError> {
            Ok(0)
        }
        async fn get_workspace_roles(
            &self,
            _w: Uuid,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<crate::models::WorkspaceRole>, i64), AppError> {
            Ok((vec![], 0))
        }
        async fn create_workspace_role(
            &self,
            _w: Uuid,
            _n: String,
            _d: Option<String>,
            _c: String,
            _s: String,
        ) -> Result<crate::models::WorkspaceRole, AppError> {
            unimplemented!()
        }
        async fn update_workspace_role(
            &self,
            _w: Uuid,
            _r: Uuid,
            _n: Option<String>,
            _d: Option<String>,
            _c: Option<String>,
        ) -> Result<Option<crate::models::WorkspaceRole>, AppError> {
            Ok(None)
        }
        async fn delete_workspace_role(&self, _w: Uuid, _r: Uuid) -> Result<u64, AppError> {
            Ok(0)
        }
        async fn get_role_permissions_by_role_id(
            &self,
            _w: Uuid,
            _r: Uuid,
        ) -> Result<Vec<String>, AppError> {
            Ok(vec![])
        }
        async fn role_exists_and_is_not_system(
            &self,
            _w: Uuid,
            _r: Uuid,
        ) -> Result<bool, AppError> {
            Ok(false)
        }
        async fn set_role_permissions(&self, _r: Uuid, _p: Vec<String>) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_permissions_for_roles(&self, _r: &[Uuid]) -> Result<Vec<String>, AppError> {
            Ok(vec![])
        }
        async fn update_workspace(
            &self,
            _id: Uuid,
            _n: Option<String>,
            _df: Option<String>,
            _tf: Option<String>,
            _tz: Option<String>,
            _tc: Option<String>,
            _dl: Option<String>,
            _l: Option<String>,
            _t: Option<String>,
        ) -> Result<crate::models::Workspace, AppError> {
            unimplemented!()
        }
        async fn is_any_role_global(&self, _r: &[Uuid]) -> Result<bool, AppError> {
            Ok(false)
        }
        async fn update_workspace_byod_url(
            &self,
            _w: Uuid,
            _u: Option<String>,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_users_with_permission(
            &self,
            _workspace_id: Uuid,
            _permission: &str,
        ) -> Result<Vec<Uuid>, AppError> {
            Ok(vec![])
        }
    }

    struct MockEmployeeRepo {
        pub state: Arc<Mutex<MockState>>,
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeeCoreRepository for MockEmployeeRepo {
        async fn get_employee_by_id(
            &self,
            id: Uuid,
            _a: &TenantAuthToken,
        ) -> Result<Option<Employee>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.employees.get(&id).cloned())
        }
        async fn find_by_number(
            &self,
            _a: &TenantAuthToken,
            _n: &str,
        ) -> Result<Option<Employee>, AppError> {
            Ok(None)
        }
        async fn get_employee_by_id_number(
            &self,
            _n: &str,
            _a: &TenantAuthToken,
        ) -> Result<Option<Employee>, AppError> {
            Ok(None)
        }
        async fn get_employees_by_ids(
            &self,
            _ids: &[Uuid],
            _a: &TenantAuthToken,
        ) -> Result<Vec<Employee>, AppError> {
            Ok(vec![])
        }
        async fn get_all_employees(
            &self,
            _a: &TenantAuthToken,
            _o: crate::models::EmployeeFilterOptions,
        ) -> Result<(Vec<Employee>, i64), AppError> {
            Ok((vec![], 0))
        }
        async fn get_active_employees(
            &self,
            _a: &TenantAuthToken,
        ) -> Result<Vec<Employee>, AppError> {
            Ok(vec![])
        }
        async fn get_active_employee_ids(
            &self,
            _a: &TenantAuthToken,
            _f: Option<&Vec<Uuid>>,
        ) -> Result<Vec<Uuid>, AppError> {
            Ok(vec![])
        }
        async fn get_paginated_employee_ids(
            &self,
            _a: &TenantAuthToken,
            _args: crate::models::EmployeePaginationArgs<'_>,
        ) -> Result<(Vec<crate::repositories::employee::EmployeeBasicInfo>, i64), AppError>
        {
            Ok((vec![], 0))
        }
        async fn get_employee_departments_at(
            &self,
            _a: &TenantAuthToken,
            _d: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError> {
            Ok(std::collections::HashMap::new())
        }
        async fn update_employee(
            &self,
            _id: Uuid,
            _a: &TenantAuthToken,
            _r: &crate::models::UpdateEmployeeRequest,
        ) -> Result<Option<Employee>, AppError> {
            Ok(None)
        }
        async fn create_employee(&self, _e: &Employee) -> Result<(), AppError> {
            Ok(())
        }
        async fn create_employee_transactional(
            &self,
            _a: &TenantAuthToken,
            _e: &Employee,
            _r: Option<Uuid>,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn create_employee_skip_duplicates(&self, _e: &Employee) -> Result<bool, AppError> {
            Ok(false)
        }
        async fn rehire_employee(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<Employee>, AppError> {
            Ok(None)
        }
        async fn delete_employee(&self, _id: Uuid, _a: &TenantAuthToken) -> Result<u64, AppError> {
            Ok(0)
        }
        async fn get_existing_employee_ids_by_numbers(
            &self,
            _a: &TenantAuthToken,
            _n: &[String],
        ) -> Result<Vec<String>, AppError> {
            Ok(vec![])
        }
        async fn get_employee_ids_by_numbers(
            &self,
            _a: &TenantAuthToken,
            _n: &[String],
        ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
            Ok(std::collections::HashMap::new())
        }
        async fn bulk_create_employees_skip_duplicates(
            &self,
            _a: &TenantAuthToken,
            _e: &[Employee],
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_managed_departments(
            &self,
            _a: &TenantAuthToken,
            _u: Uuid,
        ) -> Result<Vec<Uuid>, AppError> {
            Ok(vec![])
        }
        async fn get_employee_id_by_user_id(
            &self,
            _a: &TenantAuthToken,
            _u: Uuid,
        ) -> Result<Option<Uuid>, AppError> {
            Ok(None)
        }
        async fn get_employee_timezone_fallback(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
        ) -> Result<chrono_tz::Tz, AppError> {
            Ok(chrono_tz::UTC)
        }
        async fn get_employee_timezone_at_date(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _d: NaiveDate,
        ) -> Result<chrono_tz::Tz, AppError> {
            Ok(chrono_tz::UTC)
        }
        async fn get_headquarters_location_id(
            &self,
            _a: &TenantAuthToken,
        ) -> Result<Option<Uuid>, AppError> {
            Ok(None)
        }
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeeWorkingConditionRepository for MockEmployeeRepo {
        async fn get_working_conditions(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<crate::models::WorkingCondition>, i64), AppError> {
            Ok((vec![], 0))
        }
        async fn create_working_condition(
            &self,
            _a: &TenantAuthToken,
            _c: &crate::models::WorkingCondition,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn delete_working_condition(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _e: Uuid,
        ) -> Result<bool, AppError> {
            Ok(false)
        }
        async fn get_working_conditions_at(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _d: NaiveDate,
        ) -> Result<Vec<crate::models::WorkingCondition>, AppError> {
            Ok(vec![])
        }
        async fn get_working_conditions_batch(
            &self,
            _a: &TenantAuthToken,
            _ids: &[Uuid],
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, Vec<crate::models::WorkingCondition>>, AppError>
        {
            Ok(std::collections::HashMap::new())
        }
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeePayrollConfigRepository for MockEmployeeRepo {
        async fn upsert_employee_payroll_config(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _w: Decimal,
            _h: Decimal,
            _d: NaiveDate,
        ) -> Result<crate::models::EmployeePayrollConfig, AppError> {
            unimplemented!()
        }
        async fn get_employee_payroll_config_at(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<crate::models::EmployeePayrollConfig>, AppError> {
            Ok(None)
        }
        async fn get_employee_payroll_configs(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
        ) -> Result<Vec<crate::models::EmployeePayrollConfig>, AppError> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeeRecruitmentRepository for MockEmployeeRepo {
        async fn set_employee_recruitment_source(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _r: Option<Uuid>,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn create_recruitment_source(
            &self,
            _s: &crate::models::RecruitmentSource,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_recruitment_sources(
            &self,
            _a: &TenantAuthToken,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<crate::models::RecruitmentSource>, i64), AppError> {
            Ok((vec![], 0))
        }
        async fn get_recruitment_source_by_id(
            &self,
            _id: Uuid,
            _a: &TenantAuthToken,
        ) -> Result<Option<crate::models::RecruitmentSource>, AppError> {
            Ok(None)
        }
        async fn update_recruitment_source(
            &self,
            _id: Uuid,
            _a: &TenantAuthToken,
            _r: &crate::models::UpdateRecruitmentSourceRequest,
        ) -> Result<Option<crate::models::RecruitmentSource>, AppError> {
            Ok(None)
        }
        async fn delete_recruitment_source(
            &self,
            _id: Uuid,
            _a: &TenantAuthToken,
        ) -> Result<u64, AppError> {
            Ok(0)
        }
        async fn get_recruitment_source_breakdown(
            &self,
            _a: &TenantAuthToken,
        ) -> Result<Vec<crate::models::RecruitmentSourceBreakdown>, AppError> {
            Ok(vec![])
        }
        async fn get_turnover_analytics(
            &self,
            _a: &TenantAuthToken,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<crate::models::TurnoverAnalytics, AppError> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl crate::repositories::employee::EmployeeSelfServiceRepository for MockEmployeeRepo {
        async fn get_employee_profile(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
        ) -> Result<Option<crate::models::EmployeeProfile>, AppError> {
            Ok(None)
        }
        async fn upsert_employee_profile(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _r: &crate::models::UpdateEmployeeProfileRequest,
        ) -> Result<crate::models::EmployeeProfile, AppError> {
            unimplemented!()
        }
        async fn get_emergency_contacts(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
        ) -> Result<Vec<crate::models::EmployeeEmergencyContact>, AppError> {
            Ok(vec![])
        }
        async fn upsert_emergency_contact(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _r: &crate::models::CreateEmergencyContactRequest,
        ) -> Result<crate::models::EmployeeEmergencyContact, AppError> {
            unimplemented!()
        }
        async fn delete_emergency_contact(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _c: Uuid,
        ) -> Result<u64, AppError> {
            Ok(0)
        }
        async fn get_employee_documents(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
        ) -> Result<Vec<crate::models::EmployeeDocument>, AppError> {
            Ok(vec![])
        }
        async fn create_employee_document(
            &self,
            _d: &crate::models::EmployeeDocument,
        ) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[async_trait]
    impl crate::repositories::diesel_employee_repository::assignments::EmployeeAssignmentRepository
        for MockEmployeeRepo
    {
        async fn list_employee_assignments(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
        ) -> Result<Vec<crate::models::employee_assignment::EmployeeAssignment>, AppError> {
            Ok(vec![])
        }
        async fn get_employee_assignment(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _as_id: Uuid,
        ) -> Result<Option<crate::models::employee_assignment::EmployeeAssignment>, AppError>
        {
            Ok(None)
        }
        async fn create_employee_assignment(
            &self,
            _a: &TenantAuthToken,
            _as_: &crate::models::employee_assignment::NewEmployeeAssignment,
        ) -> Result<crate::models::employee_assignment::EmployeeAssignment, AppError> {
            unimplemented!()
        }
        async fn update_employee_assignment(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _as_id: Uuid,
            _r: &crate::models::employee_assignment::UpdateEmployeeAssignmentRequest,
        ) -> Result<crate::models::employee_assignment::EmployeeAssignment, AppError> {
            unimplemented!()
        }
        async fn delete_employee_assignment(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _as_id: Uuid,
        ) -> Result<u64, AppError> {
            Ok(0)
        }
    }

    struct MockDepartmentRepo {
        pub state: Arc<Mutex<MockState>>,
    }

    #[async_trait]
    impl DepartmentRepository for MockDepartmentRepo {
        async fn get_current_department(
            &self,
            _a: &TenantAuthToken,
            id: Uuid,
        ) -> Result<Option<crate::models::EmployeeDepartmentHistory>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.employee_departments.get(&id).cloned())
        }
        async fn get_department_by_id(
            &self,
            _a: &TenantAuthToken,
            id: Uuid,
        ) -> Result<Option<crate::models::Department>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.departments.get(&id).cloned())
        }
        async fn create_department(
            &self,
            _a: &TenantAuthToken,
            name: String,
            _de: Option<String>,
            _m: Option<Uuid>,
            _d: Option<Uuid>,
        ) -> Result<crate::models::Department, AppError> {
            Ok(crate::models::Department {
                id: Uuid::new_v4(),
                workspace_id: _a.workspace_id(),
                name,
                description: _de,
                manager_id: _m,
                default_working_pattern_id: _d,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                active_employee_count: 0,
            })
        }
        async fn get_department_by_name(
            &self,
            _a: &TenantAuthToken,
            _n: &str,
        ) -> Result<Option<Uuid>, AppError> {
            Ok(None)
        }
        async fn get_department_ids_by_names(
            &self,
            _a: &TenantAuthToken,
            _n: &[String],
        ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
            Ok(std::collections::HashMap::new())
        }
        async fn bulk_create_departments_skip_duplicates(
            &self,
            _a: &TenantAuthToken,
            _d: &[crate::models::Department],
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn update_department(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _r: crate::models::UpdateDepartmentRequest,
        ) -> Result<crate::models::Department, AppError> {
            unimplemented!()
        }
        async fn delete_department(&self, _a: &TenantAuthToken, _id: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_departments(
            &self,
            _a: &TenantAuthToken,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<crate::models::Department>, i64), AppError> {
            Ok((vec![], 0))
        }
        async fn transfer_employee(
            &self,
            _a: &TenantAuthToken,
            _e: Uuid,
            _d: Uuid,
            _ef: NaiveDate,
            _n: Option<String>,
        ) -> Result<crate::models::EmployeeDepartmentHistory, AppError> {
            unimplemented!()
        }
        async fn get_employee_department_history(
            &self,
            _a: &TenantAuthToken,
            _e: Uuid,
        ) -> Result<Vec<crate::models::EmployeeDepartmentHistory>, AppError> {
            Ok(vec![])
        }
        async fn get_employee_department_history_batch(
            &self,
            _a: &TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<
            std::collections::HashMap<Uuid, Vec<crate::models::EmployeeDepartmentHistory>>,
            AppError,
        > {
            Ok(std::collections::HashMap::new())
        }
        async fn get_current_departments_batch(
            &self,
            _a: &TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<
            std::collections::HashMap<Uuid, crate::models::EmployeeDepartmentHistory>,
            AppError,
        > {
            Ok(std::collections::HashMap::new())
        }
        async fn upsert_department_payroll_config(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _w: Decimal,
            _h: Decimal,
            _d: NaiveDate,
        ) -> Result<crate::models::DepartmentPayrollConfig, AppError> {
            unimplemented!()
        }
        async fn get_department_payroll_config_at(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<crate::models::DepartmentPayrollConfig>, AppError> {
            Ok(None)
        }
        async fn get_department_payroll_configs(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
        ) -> Result<Vec<crate::models::DepartmentPayrollConfig>, AppError> {
            Ok(vec![])
        }
        async fn get_department_payroll_configs_batch(
            &self,
            _a: &TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<
            std::collections::HashMap<Uuid, Vec<crate::models::DepartmentPayrollConfig>>,
            AppError,
        > {
            Ok(std::collections::HashMap::new())
        }
    }

    struct MockLeaveRequestRepo {
        pub state: Arc<Mutex<MockState>>,
    }

    #[async_trait]
    impl LeaveRequestRepository for MockLeaveRequestRepo {
        async fn create_leave_request(
            &self,
            _a: &TenantAuthToken,
            r: &crate::models::LeaveRequest,
        ) -> Result<(), AppError> {
            let mut state = self.state.lock().unwrap();
            state.leave_requests.insert(r.id, r.clone());
            Ok(())
        }
        async fn get_leave_request_by_id(
            &self,
            _a: &TenantAuthToken,
            id: Uuid,
        ) -> Result<Option<crate::models::LeaveRequest>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.leave_requests.get(&id).cloned())
        }
        async fn update_leave_request_status(
            &self,
            _a: &TenantAuthToken,
            id: Uuid,
            s: LeaveStatus,
            _app: Option<Uuid>,
        ) -> Result<(), AppError> {
            let mut state = self.state.lock().unwrap();
            state.leave_statuses.insert(id, s);
            Ok(())
        }
        async fn get_employee_leave_requests(
            &self,
            _a: &TenantAuthToken,
            _e_id: Uuid,
        ) -> Result<Vec<crate::models::LeaveRequest>, AppError> {
            Ok(vec![])
        }
        async fn get_workspace_leave_requests(
            &self,
            _a: &TenantAuthToken,
            _s: Option<LeaveStatus>,
        ) -> Result<Vec<crate::models::LeaveRequest>, AppError> {
            Ok(vec![])
        }
        async fn get_approved_leave_breakdown(
            &self,
            _e_id: Uuid,
            _s: NaiveDate,
            _e: NaiveDate,
            _a: &TenantAuthToken,
        ) -> Result<crate::repositories::leave_request_repository::LeaveBreakdown, AppError>
        {
            unimplemented!()
        }
        async fn get_employee_department_for_request(
            &self,
            _r_id: Uuid,
            _a: &TenantAuthToken,
        ) -> Result<Option<Uuid>, AppError> {
            Ok(None)
        }
        async fn get_leave_types(
            &self,
            _a: &TenantAuthToken,
        ) -> Result<Vec<(String, String)>, AppError> {
            Ok(vec![])
        }
        async fn get_system_generated_holiday_leave(
            &self,
            _a: &TenantAuthToken,
            _e_id: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<(Uuid, Uuid, String)>, AppError> {
            Ok(None)
        }
        async fn cancel_leave_request(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_employee_leave_history_requests(
            &self,
            _a: &TenantAuthToken,
            _e_id: Uuid,
            _y: i32,
            _lt: Option<Uuid>,
        ) -> Result<
            Vec<crate::repositories::leave_request_repository::LeaveHistoryRequestItem>,
            AppError,
        > {
            Ok(vec![])
        }
        async fn check_leave_type_exists(
            &self,
            _a: &TenantAuthToken,
            _lt: Uuid,
        ) -> Result<bool, AppError> {
            Ok(false)
        }
        async fn get_leave_type_name(
            &self,
            _a: &TenantAuthToken,
            _lt: Uuid,
        ) -> Result<Option<String>, AppError> {
            Ok(None)
        }
        async fn get_unpaid_holiday_days_without_leave(
            &self,
            _a: &TenantAuthToken,
            _e_id: Uuid,
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<i64, AppError> {
            Ok(0)
        }
        async fn get_batch_leave_days(
            &self,
            _a: &TenantAuthToken,
            _ids: &[Uuid],
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, i64>, AppError> {
            Ok(std::collections::HashMap::new())
        }
        async fn get_approved_leave_dates(
            &self,
            _a: &TenantAuthToken,
            _ids: &[Uuid],
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<std::collections::HashSet<(Uuid, NaiveDate)>, AppError> {
            Ok(std::collections::HashSet::new())
        }
        async fn get_approved_leave_details(
            &self,
            _a: &TenantAuthToken,
            _ids: &[Uuid],
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), bool>, AppError> {
            Ok(std::collections::HashMap::new())
        }
        async fn get_approved_leave_periods(
            &self,
            _a: &TenantAuthToken,
            _e_id: Uuid,
            _s: NaiveDate,
            _e: NaiveDate,
        ) -> Result<Vec<(NaiveDate, NaiveDate)>, AppError> {
            Ok(vec![])
        }
    }

    struct MockOvertimeRequestRepository {
        pub state: Arc<Mutex<MockState>>,
    }

    #[async_trait]
    impl OvertimeRequestRepository for MockOvertimeRequestRepository {
        async fn create_request(
            &self,
            _a: &TenantAuthToken,
            r: crate::models::OvertimeRequest,
            _e: Vec<crate::models::OvertimeRequestEmployee>,
        ) -> Result<crate::models::OvertimeRequest, AppError> {
            let mut state = self.state.lock().unwrap();
            state.ot_requests.insert(r.id, r.clone());
            Ok(r)
        }
        async fn update_request(
            &self,
            _a: &TenantAuthToken,
            r: crate::models::OvertimeRequest,
        ) -> Result<crate::models::OvertimeRequest, AppError> {
            let mut state = self.state.lock().unwrap();
            state.ot_requests.insert(r.id, r.clone());
            Ok(r)
        }
        async fn get_request(
            &self,
            _a: &TenantAuthToken,
            _w: Uuid,
            id: Uuid,
        ) -> Result<Option<crate::models::OvertimeRequest>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state.ot_requests.get(&id).cloned())
        }
        async fn get_request_employees(
            &self,
            _a: &TenantAuthToken,
            _id: Uuid,
        ) -> Result<Vec<crate::models::OvertimeRequestEmployee>, AppError> {
            Ok(vec![])
        }
        async fn update_employee_consent(
            &self,
            _a: &TenantAuthToken,
            _r_id: Uuid,
            _e_id: Uuid,
            _s: &str,
            _re: Option<String>,
        ) -> Result<crate::models::OvertimeRequestEmployee, AppError> {
            unimplemented!()
        }
    }

    struct MockDailySummaryRepository;

    #[async_trait]
    impl crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository
        for MockDailySummaryRepository
    {
        async fn get_summary(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _date: NaiveDate,
        ) -> Result<Option<crate::models::AttendanceDailySummary>, AppError> {
            Ok(None)
        }

        async fn get_summary_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<Option<crate::models::AttendanceDailySummary>, AppError> {
            Ok(None)
        }

        async fn upsert_calculated_summary(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _date: NaiveDate,
            _actual_work_minutes: i32,
            _system_detected_ot_minutes: i32,
            _new_status: crate::models::OtStatus,
            _is_holiday: bool,
            _holiday_name: Option<String>,
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn claim_ot(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _date: NaiveDate,
            _claimed_minutes: i32,
            _reason: Option<String>,
        ) -> Result<usize, AppError> {
            Ok(0)
        }

        async fn approve_ot(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _date: NaiveDate,
            _approved_minutes: i32,
            _manager_note: Option<String>,
            _status: crate::models::OtStatus,
        ) -> Result<usize, AppError> {
            Ok(0)
        }

        async fn upsert_paper_ot(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _date: NaiveDate,
            _claimed_minutes: i32,
            _reason: String,
            _entered_by_user_id: Uuid,
            _photo_url: Option<String>,
        ) -> Result<crate::models::AttendanceDailySummary, AppError> {
            unimplemented!()
        }

        async fn get_batch_ot_statuses(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[Uuid],
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), String>, AppError> {
            Ok(std::collections::HashMap::new())
        }

        async fn get_summaries(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Option<Uuid>,
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<Vec<crate::models::AttendanceDailySummary>, AppError> {
            Ok(vec![])
        }

        async fn get_distinct_work_days_from_summary(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<Vec<NaiveDate>, AppError> {
            Ok(vec![])
        }
    }

    struct TestLeaveWorkflowCallbackHandler {
        leave_request_repo: Arc<MockLeaveRequestRepo>,
    }

    #[async_trait]
    impl WorkflowCallbackHandler for TestLeaveWorkflowCallbackHandler {
        async fn on_workflow_completed(
            &self,
            auth_token: &crate::diesel_pool::TenantAuthToken,
            entity_id: Uuid,
            status: &str,
            user_id: Uuid,
        ) -> Result<(), AppError> {
            let leave_status = match status {
                "approved" => LeaveStatus::Approved,
                "rejected" => LeaveStatus::Rejected,
                _ => LeaveStatus::Pending,
            };
            self.leave_request_repo
                .update_leave_request_status(auth_token, entity_id, leave_status, Some(user_id))
                .await
        }
    }

    fn setup_test_service() -> (ApprovalWorkflowService, Arc<Mutex<MockState>>) {
        let state = Arc::new(Mutex::new(MockState::new()));
        let leave_repo = Arc::new(MockLeaveRequestRepo {
            state: state.clone(),
        });
        let mut handlers: std::collections::HashMap<String, Arc<dyn WorkflowCallbackHandler>> =
            std::collections::HashMap::new();
        handlers.insert(
            "leave_request".to_string(),
            Arc::new(TestLeaveWorkflowCallbackHandler {
                leave_request_repo: leave_repo.clone(),
            }),
        );
        let service = ApprovalWorkflowService::new(
            Arc::new(MockApprovalWorkflowRepository {
                create_workflow_called: std::sync::atomic::AtomicBool::new(false),
                state: state.clone(),
            }),
            Arc::new(MockWorkspaceConfigRepository {
                state: state.clone(),
            }),
            Arc::new(MockWorkspaceRepository {
                state: state.clone(),
            }),
            Arc::new(MockEmployeeRepo {
                state: state.clone(),
            }),
            Arc::new(MockDepartmentRepo {
                state: state.clone(),
            }),
            leave_repo,
            Arc::new(MockOvertimeRequestRepository {
                state: state.clone(),
            }),
            Arc::new(MockDailySummaryRepository),
        )
        .with_handlers(handlers);
        (service, state)
    }

    fn mock_employee(id: Uuid, workspace_id: Uuid, user_id: Option<Uuid>, name: &str) -> Employee {
        Employee::new(
            id,
            workspace_id,
            user_id,
            name.to_string(),
            format!("EMP_{name}"),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            None,
            vec![],
            true,
            Utc::now(),
            Utc::now(),
            None,
            Uuid::new_v4(),
            None,
        )
    }

    fn mock_delegation_config(
        workspace_id: Uuid,
        user_id: Uuid,
        delegated_user_id: Uuid,
    ) -> WorkspaceConfigEntry {
        WorkspaceConfigEntry {
            id: Uuid::new_v4(),
            workspace_id,
            scope_type: "workspace".to_string(),
            scope_id: None,
            config_key: format!("delegation:{user_id}"),
            value_int: None,
            value_decimal: None,
            value_text: Some(delegated_user_id.to_string()),
            value_bool: None,
            value_json: None,
            effective_from: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            effective_to: None,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_workflow_success() {
        let (service, state) = setup_test_service();
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let workflow = ApprovalWorkflow {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(), // deliberate mismatch
            entity_type: "overtime_request".to_string(),
            name: "Default OT Workflow".to_string(),
            description: None,
            total_steps: 1,
            is_active: true,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let step = ApprovalWorkflowStep {
            id: Uuid::new_v4(),
            workflow_id: Uuid::new_v4(),
            step_order: 1,
            step_name: "Manager Approval".to_string(),
            approver_type: "manager".to_string(),
            approver_config: serde_json::json!({}),
            action_required: "approve".to_string(),
            can_skip: false,
            created_at: Utc::now(),
        };

        let (created_wf, created_steps) = service
            .create_workflow(&token, workflow, vec![step])
            .await
            .unwrap();

        // Ensure workspace_id is enforced
        assert_eq!(created_wf.workspace_id, ws_id);

        // Ensure step has correct workflow_id
        assert_eq!(created_steps.len(), 1);
        assert_eq!(created_steps[0].workflow_id, created_wf.id);

        let lock = state.lock().unwrap();
        assert!(lock.workflows.contains_key(&created_wf.id));
    }

    #[tokio::test]
    async fn test_self_approval_prevention() {
        let (service, state) = setup_test_service();
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let employee_user_id = Uuid::new_v4();
        let employee_id = Uuid::new_v4();
        let beneficiary = mock_employee(employee_id, ws_id, Some(employee_user_id), "John Doe");

        let dept_id = Uuid::new_v4();
        let department = crate::models::Department {
            id: dept_id,
            workspace_id: ws_id,
            name: "Engineering".to_string(),
            description: None,
            manager_id: Some(employee_id),
            default_working_pattern_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            active_employee_count: 1,
        };

        let dept_history = crate::models::EmployeeDepartmentHistory {
            id: Uuid::new_v4(),
            employee_id,
            department_id: dept_id,
            effective_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            notes: None,
            created_at: Utc::now(),
        };

        let admin_user_id = Uuid::new_v4();

        {
            let mut s = state.lock().unwrap();
            s.employees.insert(employee_id, beneficiary);
            s.departments.insert(dept_id, department);
            s.employee_departments.insert(employee_id, dept_history);
            s.admins.push(admin_user_id);
        }

        let workflow = ApprovalWorkflow {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            entity_type: "leave_request".to_string(),
            name: "Self-Approval Test".to_string(),
            description: None,
            total_steps: 1,
            is_active: true,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let step = ApprovalWorkflowStep {
            id: Uuid::new_v4(),
            workflow_id: workflow.id,
            step_order: 1,
            step_name: "Manager approval (self)".to_string(),
            approver_type: "department_manager".to_string(),
            approver_config: serde_json::json!({}),
            action_required: "approve".to_string(),
            can_skip: false,
            created_at: Utc::now(),
        };

        {
            let mut s = state.lock().unwrap();
            s.active_workflows
                .insert("leave_request".to_string(), workflow.clone());
            s.steps.insert(workflow.id, vec![step]);
        }

        let entity_id = Uuid::new_v4();
        let leave_req = crate::models::LeaveRequest {
            id: entity_id,
            employee_id,
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
            reason: None,
            status: LeaveStatus::Pending,
            duration_type: crate::models::enums::LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            requested_by_user_id: Some(employee_user_id),
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        {
            let mut s = state.lock().unwrap();
            s.leave_requests.insert(entity_id, leave_req);
        }

        let instance = service
            .submit_to_workflow(&token, "leave_request".to_string(), entity_id, employee_id)
            .await
            .unwrap();

        // 1. Verify manager (the beneficiary) cannot see the pending task
        let manager_tasks = service
            .get_pending_tasks_for_user(&token, employee_user_id)
            .await
            .unwrap();
        assert!(manager_tasks.is_empty());

        // 2. But the task is actually created and assigned to the department manager (dept_id)
        let active_tasks = state
            .lock()
            .unwrap()
            .tasks
            .values()
            .filter(|t| t.instance_id() == instance.id && t.status() == "pending")
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(active_tasks.len(), 1);
        assert_eq!(active_tasks[0].assignee_type(), "department_manager");
        assert_eq!(active_tasks[0].assignee_target_id(), Some(dept_id));

        // 3. Verify admin can approve the request
        let approved_instance = service
            .process_approval_action(
                &token,
                admin_user_id,
                instance.id,
                "approve".to_string(),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(approved_instance.status, "approved");
    }

    #[tokio::test]
    async fn test_self_approval_prevention_can_skip() {
        let (service, state) = setup_test_service();
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let employee_user_id = Uuid::new_v4();
        let employee_id = Uuid::new_v4();
        let beneficiary = mock_employee(employee_id, ws_id, Some(employee_user_id), "John Doe");

        let dept_id = Uuid::new_v4();
        let department = crate::models::Department {
            id: dept_id,
            workspace_id: ws_id,
            name: "Engineering".to_string(),
            description: None,
            manager_id: Some(employee_id),
            default_working_pattern_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            active_employee_count: 1,
        };

        let dept_history = crate::models::EmployeeDepartmentHistory {
            id: Uuid::new_v4(),
            employee_id,
            department_id: dept_id,
            effective_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            notes: None,
            created_at: Utc::now(),
        };

        {
            let mut s = state.lock().unwrap();
            s.employees.insert(employee_id, beneficiary);
            s.departments.insert(dept_id, department);
            s.employee_departments.insert(employee_id, dept_history);
        }

        let workflow = ApprovalWorkflow {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            entity_type: "leave_request".to_string(),
            name: "Self-Approval Skip Test".to_string(),
            description: None,
            total_steps: 1,
            is_active: true,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let step = ApprovalWorkflowStep {
            id: Uuid::new_v4(),
            workflow_id: workflow.id,
            step_order: 1,
            step_name: "Manager approval (self)".to_string(),
            approver_type: "department_manager".to_string(),
            approver_config: serde_json::json!({}),
            action_required: "approve".to_string(),
            can_skip: true,
            created_at: Utc::now(),
        };

        {
            let mut s = state.lock().unwrap();
            s.active_workflows
                .insert("leave_request".to_string(), workflow.clone());
            s.steps.insert(workflow.id, vec![step]);
        }

        let entity_id = Uuid::new_v4();
        let leave_req = crate::models::LeaveRequest {
            id: entity_id,
            employee_id,
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
            reason: None,
            status: LeaveStatus::Pending,
            duration_type: crate::models::enums::LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            requested_by_user_id: Some(employee_user_id),
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        {
            let mut s = state.lock().unwrap();
            s.leave_requests.insert(entity_id, leave_req);
        }

        let instance = service
            .submit_to_workflow(&token, "leave_request".to_string(), entity_id, employee_id)
            .await
            .unwrap();

        let steps_snapshot: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(instance.steps_snapshot).unwrap();

        assert_eq!(steps_snapshot.len(), 1);
        assert!(steps_snapshot[0].resolved_approver_ids.is_empty());
    }

    #[tokio::test]
    async fn test_fallback_to_admin() {
        let (service, state) = setup_test_service();
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let employee_id = Uuid::new_v4();
        let beneficiary = mock_employee(employee_id, ws_id, Some(Uuid::new_v4()), "Jane Doe");

        let admin_user_id = Uuid::new_v4();

        {
            let mut s = state.lock().unwrap();
            s.employees.insert(employee_id, beneficiary);
            s.admins.push(admin_user_id);
        }

        let entity_id = Uuid::new_v4();
        let instance = service
            .submit_to_workflow(&token, "ot_request".to_string(), entity_id, employee_id)
            .await
            .unwrap();

        let steps_snapshot: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(instance.steps_snapshot).unwrap();

        assert_eq!(steps_snapshot.len(), 1);
        assert_eq!(
            steps_snapshot[0].step_name,
            "Direct Manager Approval (Fallback)"
        );

        let admin_tasks = service
            .get_pending_tasks_for_user(&token, admin_user_id)
            .await
            .unwrap();
        assert_eq!(admin_tasks.len(), 1);
        assert_eq!(admin_tasks[0].assignee_target_id(), Some(admin_user_id));
    }

    #[tokio::test]
    async fn test_ooo_delegation() {
        let (service, state) = setup_test_service();
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let employee_id = Uuid::new_v4();
        let beneficiary = mock_employee(employee_id, ws_id, Some(Uuid::new_v4()), "Jane Doe");

        let manager_user_id = Uuid::new_v4();
        let delegated_user_id = Uuid::new_v4();

        let workflow = ApprovalWorkflow {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            entity_type: "leave_request".to_string(),
            name: "Delegation Test".to_string(),
            description: None,
            total_steps: 1,
            is_active: true,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let step = ApprovalWorkflowStep {
            id: Uuid::new_v4(),
            workflow_id: workflow.id,
            step_order: 1,
            step_name: "Specific User Step".to_string(),
            approver_type: "specific_user".to_string(),
            approver_config: serde_json::json!({
                "user_id": manager_user_id.to_string(),
            }),
            action_required: "approve".to_string(),
            can_skip: false,
            created_at: Utc::now(),
        };

        let delegation_config = mock_delegation_config(ws_id, manager_user_id, delegated_user_id);

        {
            let mut s = state.lock().unwrap();
            s.employees.insert(employee_id, beneficiary);
            s.active_workflows
                .insert("leave_request".to_string(), workflow.clone());
            s.steps.insert(workflow.id, vec![step]);
            s.configs
                .insert(delegation_config.config_key.clone(), delegation_config);
        }

        let entity_id = Uuid::new_v4();
        let instance = service
            .submit_to_workflow(&token, "leave_request".to_string(), entity_id, employee_id)
            .await
            .unwrap();

        let steps_snapshot: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(instance.steps_snapshot).unwrap();

        assert_eq!(steps_snapshot.len(), 1);

        let manager_tasks = service
            .get_pending_tasks_for_user(&token, manager_user_id)
            .await
            .unwrap();
        assert_eq!(manager_tasks.len(), 1);
        assert_eq!(manager_tasks[0].assignee_target_id(), Some(manager_user_id));

        let delegated_tasks = service
            .get_pending_tasks_for_user(&token, delegated_user_id)
            .await
            .unwrap();
        assert_eq!(delegated_tasks.len(), 1);
        assert_eq!(
            delegated_tasks[0].assignee_target_id(),
            Some(manager_user_id)
        );
    }

    #[tokio::test]
    async fn test_consensus_modes() {
        let (service, state) = setup_test_service();
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let entity_id = Uuid::new_v4();

        let steps_all = vec![WorkflowInstanceStepSnapshot {
            step_order: 1,
            step_name: "Consensus All Step".to_string(),
            approver_type: "specific_user".to_string(),
            approver_config: serde_json::json!({
                "consensus_mode": "all",
            }),
            resolved_approver_ids: vec![user_a, user_b],
            consensus_mode: "all".to_string(),
            action_required: "approve".to_string(),
            can_skip: false,
            status: "pending".to_string(),
            sla_hours: None,
            escalation_action: None,
            approvers_acted: vec![],
        }];

        let instance_all = ApprovalWorkflowInstance {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            workflow_id: None,
            entity_type: "leave_request".to_string(),
            entity_id,
            status: "pending".to_string(),
            current_step_order: 1,
            steps_snapshot: serde_json::to_value(&steps_all).unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let instance_all_id = instance_all.id;

        let task_a = ApprovalWorkflowTask::new(
            instance_all_id,
            1,
            "specific_user".to_string(),
            Some(user_a),
            None,
        );
        let task_b = ApprovalWorkflowTask::new(
            instance_all_id,
            1,
            "specific_user".to_string(),
            Some(user_b),
            None,
        );

        {
            let mut s = state.lock().unwrap();
            s.instances.insert(instance_all_id, instance_all);
            s.tasks.insert(task_a.id(), task_a);
            s.tasks.insert(task_b.id(), task_b);
        }

        let res1 = service
            .process_approval_action(
                &token,
                user_a,
                instance_all_id,
                "approve".to_string(),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(res1.status, "pending");

        let steps_snapshot1: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(res1.steps_snapshot).unwrap();
        assert_eq!(steps_snapshot1[0].status, "pending");
        assert_eq!(steps_snapshot1[0].approvers_acted.len(), 1);

        let res2 = service
            .process_approval_action(
                &token,
                user_b,
                instance_all_id,
                "approve".to_string(),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(res2.status, "approved");

        let steps_snapshot2: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(res2.steps_snapshot).unwrap();
        assert_eq!(steps_snapshot2[0].status, "approved");
        assert_eq!(steps_snapshot2[0].approvers_acted.len(), 2);
    }

    #[tokio::test]
    async fn test_consensus_mode_any() {
        let (service, state) = setup_test_service();
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let entity_id = Uuid::new_v4();

        let steps_any = vec![WorkflowInstanceStepSnapshot {
            step_order: 1,
            step_name: "Consensus Any Step".to_string(),
            approver_type: "specific_user".to_string(),
            approver_config: serde_json::json!({
                "consensus_mode": "any",
            }),
            resolved_approver_ids: vec![user_a, user_b],
            consensus_mode: "any".to_string(),
            action_required: "approve".to_string(),
            can_skip: false,
            status: "pending".to_string(),
            sla_hours: None,
            escalation_action: None,
            approvers_acted: vec![],
        }];

        let instance_any = ApprovalWorkflowInstance {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            workflow_id: None,
            entity_type: "leave_request".to_string(),
            entity_id,
            status: "pending".to_string(),
            current_step_order: 1,
            steps_snapshot: serde_json::to_value(&steps_any).unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let instance_any_id = instance_any.id;

        let task_a = ApprovalWorkflowTask::new(
            instance_any_id,
            1,
            "specific_user".to_string(),
            Some(user_a),
            None,
        );
        let task_b = ApprovalWorkflowTask::new(
            instance_any_id,
            1,
            "specific_user".to_string(),
            Some(user_b),
            None,
        );

        {
            let mut s = state.lock().unwrap();
            s.instances.insert(instance_any_id, instance_any);
            s.tasks.insert(task_a.id(), task_a);
            s.tasks.insert(task_b.id(), task_b);
        }

        let res = service
            .process_approval_action(
                &token,
                user_a,
                instance_any_id,
                "approve".to_string(),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(res.status, "approved");

        let steps_snapshot: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(res.steps_snapshot).unwrap();
        assert_eq!(steps_snapshot[0].status, "approved");
    }

    #[tokio::test]
    async fn test_action_transitions() {
        let (service, state) = setup_test_service();
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let user_a = Uuid::new_v4();
        let entity_id = Uuid::new_v4();

        let steps_reject = vec![WorkflowInstanceStepSnapshot {
            step_order: 1,
            step_name: "Step".to_string(),
            approver_type: "specific_user".to_string(),
            approver_config: serde_json::json!({}),
            resolved_approver_ids: vec![user_a],
            consensus_mode: "any".to_string(),
            action_required: "approve".to_string(),
            can_skip: false,
            status: "pending".to_string(),
            sla_hours: None,
            escalation_action: None,
            approvers_acted: vec![],
        }];

        let instance_reject = ApprovalWorkflowInstance {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            workflow_id: None,
            entity_type: "leave_request".to_string(),
            entity_id,
            status: "pending".to_string(),
            current_step_order: 1,
            steps_snapshot: serde_json::to_value(&steps_reject).unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let task_reject = ApprovalWorkflowTask::new(
            instance_reject.id,
            1,
            "specific_user".to_string(),
            Some(user_a),
            None,
        );

        {
            let mut s = state.lock().unwrap();
            s.instances
                .insert(instance_reject.id, instance_reject.clone());
            s.tasks.insert(task_reject.id(), task_reject);
        }

        let res_reject = service
            .process_approval_action(
                &token,
                user_a,
                instance_reject.id,
                "reject".to_string(),
                Some("Not allowed".to_string()),
                None,
            )
            .await
            .unwrap();

        assert_eq!(res_reject.status, "rejected");

        let steps_snapshot_reject: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(res_reject.steps_snapshot).unwrap();
        assert_eq!(steps_snapshot_reject[0].status, "rejected");

        {
            let s = state.lock().unwrap();
            assert_eq!(
                *s.leave_statuses.get(&entity_id).unwrap(),
                LeaveStatus::Rejected
            );
        }

        let instance_return = ApprovalWorkflowInstance {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            workflow_id: None,
            entity_type: "leave_request".to_string(),
            entity_id,
            status: "pending".to_string(),
            current_step_order: 1,
            steps_snapshot: serde_json::to_value(&steps_reject).unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let task_return = ApprovalWorkflowTask::new(
            instance_return.id,
            1,
            "specific_user".to_string(),
            Some(user_a),
            None,
        );

        {
            let mut s = state.lock().unwrap();
            s.instances
                .insert(instance_return.id, instance_return.clone());
            s.tasks.insert(task_return.id(), task_return);
        }

        let res_return = service
            .process_approval_action(
                &token,
                user_a,
                instance_return.id,
                "return".to_string(),
                Some("Please correct date".to_string()),
                None,
            )
            .await
            .unwrap();

        assert_eq!(res_return.status, "returned");

        let steps_snapshot_return: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(res_return.steps_snapshot).unwrap();
        assert_eq!(steps_snapshot_return[0].status, "returned");

        {
            let s = state.lock().unwrap();
            assert_eq!(
                *s.leave_statuses.get(&entity_id).unwrap(),
                LeaveStatus::Pending
            );
        }

        let instance_reassign = ApprovalWorkflowInstance {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            workflow_id: None,
            entity_type: "leave_request".to_string(),
            entity_id,
            status: "pending".to_string(),
            current_step_order: 1,
            steps_snapshot: serde_json::to_value(&steps_reject).unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user_reassigned = Uuid::new_v4();

        let task_reassign = ApprovalWorkflowTask::new(
            instance_reassign.id,
            1,
            "specific_user".to_string(),
            Some(user_a),
            None,
        );

        {
            let mut s = state.lock().unwrap();
            s.instances
                .insert(instance_reassign.id, instance_reassign.clone());
            s.tasks.insert(task_reassign.id(), task_reassign);
        }

        let res_reassign = service
            .process_approval_action(
                &token,
                user_a,
                instance_reassign.id,
                "reassign".to_string(),
                Some("Delegating to B".to_string()),
                Some(user_reassigned),
            )
            .await
            .unwrap();

        assert_eq!(res_reassign.status, "pending");

        let steps_snapshot_reassign: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(res_reassign.steps_snapshot).unwrap();
        assert_eq!(steps_snapshot_reassign[0].status, "pending");
        assert_eq!(
            steps_snapshot_reassign[0].resolved_approver_ids,
            vec![user_reassigned]
        );
    }

    #[tokio::test]
    async fn test_multi_step_advancement() {
        let (service, state) = setup_test_service();
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let entity_id = Uuid::new_v4();

        let steps = vec![
            WorkflowInstanceStepSnapshot {
                step_order: 1,
                step_name: "Step 1".to_string(),
                approver_type: "specific_user".to_string(),
                approver_config: serde_json::json!({
                    "user_id": user_a.to_string(),
                }),
                resolved_approver_ids: vec![user_a],
                consensus_mode: "any".to_string(),
                action_required: "approve".to_string(),
                can_skip: false,
                status: "pending".to_string(),
                sla_hours: None,
                escalation_action: None,
                approvers_acted: vec![],
            },
            WorkflowInstanceStepSnapshot {
                step_order: 2,
                step_name: "Step 2".to_string(),
                approver_type: "specific_user".to_string(),
                approver_config: serde_json::json!({
                    "user_id": user_b.to_string(),
                }),
                resolved_approver_ids: vec![user_b],
                consensus_mode: "any".to_string(),
                action_required: "approve".to_string(),
                can_skip: false,
                status: "queued".to_string(),
                sla_hours: None,
                escalation_action: None,
                approvers_acted: vec![],
            },
        ];

        let instance = ApprovalWorkflowInstance {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            workflow_id: None,
            entity_type: "leave_request".to_string(),
            entity_id,
            status: "pending".to_string(),
            current_step_order: 1,
            steps_snapshot: serde_json::to_value(&steps).unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let task_1 = ApprovalWorkflowTask::new(
            instance.id,
            1,
            "specific_user".to_string(),
            Some(user_a),
            None,
        );

        {
            let mut s = state.lock().unwrap();
            s.instances.insert(instance.id, instance.clone());
            s.tasks.insert(task_1.id(), task_1);
        }

        let res1 = service
            .process_approval_action(
                &token,
                user_a,
                instance.id,
                "approve".to_string(),
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(res1.status, "pending");
        assert_eq!(res1.current_step_order, 2);

        let steps_snapshot1: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(res1.steps_snapshot).unwrap();
        assert_eq!(steps_snapshot1[0].status, "approved");
        assert_eq!(steps_snapshot1[1].status, "pending");

        let res2 = service
            .process_approval_action(
                &token,
                user_b,
                instance.id,
                "approve".to_string(),
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(res2.status, "approved");

        let steps_snapshot2: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(res2.steps_snapshot).unwrap();
        assert_eq!(steps_snapshot2[1].status, "approved");

        {
            let s = state.lock().unwrap();
            assert_eq!(
                *s.leave_statuses.get(&entity_id).unwrap(),
                LeaveStatus::Approved
            );
        }
    }

    #[tokio::test]
    async fn test_custom_workflow_callback_handler() {
        let (mut service, state) = setup_test_service();
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let entity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let called_state = std::sync::Arc::new(std::sync::Mutex::new(None));
        let handler = MockCallbackHandler {
            called: called_state.clone(),
        };
        let mut handlers = std::collections::HashMap::new();
        handlers.insert(
            "custom_entity".to_string(),
            Arc::new(handler) as Arc<dyn WorkflowCallbackHandler>,
        );
        service = service.with_handlers(handlers);

        let instance = ApprovalWorkflowInstance {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            workflow_id: None,
            entity_type: "custom_entity".to_string(),
            entity_id,
            status: "pending".to_string(),
            current_step_order: 1,
            steps_snapshot: serde_json::json!([{
                "step_order": 1,
                "step_name": "Step 1",
                "approver_type": "specific_user",
                "approver_config": {},
                "resolved_approver_ids": [user_id.to_string()],
                "consensus_mode": "any",
                "action_required": "approve",
                "can_skip": false,
                "status": "pending",
                "sla_hours": null,
                "escalation_action": null,
                "approvers_acted": []
            }]),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let task = ApprovalWorkflowTask::new(
            instance.id,
            1,
            "specific_user".to_string(),
            Some(user_id),
            None,
        );

        {
            let mut s = state.lock().unwrap();
            s.instances.insert(instance.id, instance.clone());
            s.tasks.insert(task.id(), task);
        }

        let res = service
            .process_approval_action(
                &token,
                user_id,
                instance.id,
                "approve".to_string(),
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(res.status, "approved");

        let result_guard = called_state.lock().unwrap();
        assert!(result_guard.is_some());
        let (called_id, called_status) = result_guard.as_ref().unwrap();
        assert_eq!(*called_id, entity_id);
        assert_eq!(called_status, "approved");
    }

    struct MockCallbackHandler {
        called: std::sync::Arc<std::sync::Mutex<Option<(Uuid, String)>>>,
    }

    #[async_trait::async_trait]
    impl WorkflowCallbackHandler for MockCallbackHandler {
        async fn on_workflow_completed(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            entity_id: Uuid,
            status: &str,
            _user_id: Uuid,
        ) -> Result<(), AppError> {
            let mut guard = self.called.lock().unwrap();
            *guard = Some((entity_id, status.to_string()));
            Ok(())
        }
    }
}
