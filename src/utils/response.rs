use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AppResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
    pub errors: Option<Vec<String>>,
}

impl<T: Serialize> AppResponse<T> {
    pub fn success() -> Self {
        Self {
            success: true,
            message: None,
            data: Some(data),
            errors: None,
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            message: Some(message),
             Some(data),
            errors: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message: Some(message),
             None,
            errors: None,
        }
    }

    pub fn validation_errors(errors: Vec<String>) -> Self {
        Self {
            success: false,
            message: Some("Validation failed".to_string()),
             None,
            errors: Some(errors),
        }
    }
}
