use crate::models::{
    Employee, RecruitmentEffectivenessAnalytics, RecruitmentSource, RecruitmentSourceBreakdown,
    TurnoverAnalytics, WorkingCondition,
};
use std::sync::Mutex;
use uuid::Uuid;

pub struct MockEmployeeRepo {
    pub employee: Mutex<Option<Employee>>,
    pub employees: Mutex<Vec<Employee>>,
    pub total: Mutex<i64>,
    pub working_conditions: Mutex<Vec<WorkingCondition>>,
    pub delete_wc_result: Mutex<bool>,
    pub delete_emp_rows: Mutex<u64>,
    pub updated_employee: Mutex<Option<Employee>>,
    pub recruitment_sources: Mutex<Vec<RecruitmentSource>>,
    pub delete_rs_rows: Mutex<u64>,
    pub turnover: Mutex<Option<TurnoverAnalytics>>,
    pub updated_recruitment_source: Mutex<Option<RecruitmentSource>>,
    pub recruitment_source_breakdown: Mutex<Vec<RecruitmentSourceBreakdown>>,
    pub effectiveness: Mutex<Option<RecruitmentEffectivenessAnalytics>>,
    pub employee_id_by_user_id: Mutex<Option<Uuid>>,
}

impl MockEmployeeRepo {
    pub fn empty() -> Self {
        Self {
            employee: Mutex::new(None),
            employees: Mutex::new(vec![]),
            total: Mutex::new(0),
            working_conditions: Mutex::new(vec![]),
            delete_wc_result: Mutex::new(false),
            delete_emp_rows: Mutex::new(0),
            updated_employee: Mutex::new(None),
            recruitment_sources: Mutex::new(vec![]),
            delete_rs_rows: Mutex::new(0),
            turnover: Mutex::new(None),
            updated_recruitment_source: Mutex::new(None),
            recruitment_source_breakdown: Mutex::new(vec![]),
            effectiveness: Mutex::new(None),
            employee_id_by_user_id: Mutex::new(None),
        }
    }

    pub fn with_employee(emp: Employee) -> Self {
        let mut mock = Self::empty();
        mock.employee = Mutex::new(Some(emp));
        mock
    }
}
