// src/utils/response.rs
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AppResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,  // ← Имя поля: data
    pub errors: Option<Vec<String>>,
}

impl<T: Serialize> AppResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: None,
            data: Some(data),  // ✅ ИСПРАВЛЕНО: добавлено имя поля "data:"
            errors: None,
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            message: Some(message),
            data: Some(data),  // ✅ ИСПРАВЛЕНО: добавлено имя поля "data:"
            errors: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message: Some(message),
            data: None,  // ✅ ИСПРАВЛЕНО: добавлено имя поля "data:"
            errors: None,
        }
    }

    pub fn validation_errors(errors: Vec<String>) -> Self {
        Self {
            success: false,
            message: Some("Validation failed".to_string()),
            data: None,  // ✅ ИСПРАВЛЕНО: добавлено имя поля "data:"
            errors: Some(errors),
        }
    }
}
