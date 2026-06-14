use crate::errors::AppError;
use regex::Regex;
use std::collections::HashMap;

pub struct TemplateService;

impl TemplateService {
    /// Takes a template string containing variables like `{{EMPLOYEE_NAME}}`
    /// and a HashMap of variable names to their values, and returns the
    /// filled template string.
    pub fn fill_template(
        template: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, AppError> {
        // Regex to match {{VAR_NAME}}
        let re = Regex::new(r"\{\{([A-Z0-9_]+)\}\}").map_err(|e| {
            AppError::Internal(format!("Failed to compile regex for templates: {e}"))
        })?;

        let result = re.replace_all(template, |caps: &regex::Captures| {
            let key = &caps[1];
            if let Some(val) = variables.get(key) {
                val.clone()
            } else {
                // If variable not found, leave the placeholder intact
                caps[0].to_string()
            }
        });

        Ok(result.into_owned())
    }

    /// Fetches necessary employee data and uses `fill_template` to generate the document.
    #[allow(clippy::unused_async)]
    pub async fn generate_employee_document(
        // TODO: In the next iteration, we will inject state to query repos
        // state: &std::sync::Arc<crate::AppState>,
        // employee_id: uuid::Uuid,
        template_content: &str,
        variables_override: Option<HashMap<String, String>>,
    ) -> Result<String, AppError> {
        let mut variables = HashMap::new();

        // Default mocked variables for now (until connected to DB repos)
        variables.insert("COMPANY_NAME".to_string(), "LeeAttend Corp".to_string());
        variables.insert(
            "DOCUMENT_DATE".to_string(),
            chrono::Utc::now().format("%d/%m/%Y").to_string(),
        );

        // Merge overrides
        if let Some(overrides) = variables_override {
            for (k, v) in overrides {
                variables.insert(k, v);
            }
        }

        Self::fill_template(template_content, &variables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_template() {
        let template = "Hello {{FIRST_NAME}} {{LAST_NAME}}, your salary is {{SALARY}}.";
        let mut vars = HashMap::new();
        vars.insert("FIRST_NAME".to_string(), "John".to_string());
        vars.insert("LAST_NAME".to_string(), "Doe".to_string());
        vars.insert("SALARY".to_string(), "1000".to_string());

        let result = TemplateService::fill_template(template, &vars).unwrap();
        assert_eq!(result, "Hello John Doe, your salary is 1000.");
    }

    #[test]
    fn test_missing_variable_keeps_placeholder() {
        let template = "Hello {{FIRST_NAME}}, your ID is {{MISSING_ID}}.";
        let mut vars = HashMap::new();
        vars.insert("FIRST_NAME".to_string(), "Jane".to_string());

        let result = TemplateService::fill_template(template, &vars).unwrap();
        assert_eq!(result, "Hello Jane, your ID is {{MISSING_ID}}.");
    }
}
