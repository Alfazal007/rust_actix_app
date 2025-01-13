#[derive(serde::Serialize)]
pub struct ValidationErrorsToBeReturned {
    pub errors: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct GeneralError {
    pub errors: String,
}
