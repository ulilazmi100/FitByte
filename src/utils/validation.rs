use validator::Validate;
use crate::errors::AppError;

pub fn validate_payload<T: Validate>(payload: &T) -> Result<(), AppError> {
    payload.validate()
        .map_err(|err| AppError::BadRequest(err.to_string()))
}

pub fn validate_preference(preference: &str) -> Result<(), AppError> {
    if !["CARDIO", "WEIGHT"].contains(&preference) {
        return Err(AppError::BadRequest("Preference must be either CARDIO or WEIGHT".to_string()));
    }
    Ok(())
}

pub fn validate_weight_unit(weight_unit: &str) -> Result<(), AppError> {
    if !["KG", "LBS"].contains(&weight_unit) {
        return Err(AppError::BadRequest("Weight unit must be either KG or LBS".to_string()));
    }
    Ok(())
}

pub fn validate_height_unit(height_unit: &str) -> Result<(), AppError> {
    if !["CM", "INCH"].contains(&height_unit) {
        return Err(AppError::BadRequest("Height unit must be either CM or INCH".to_string()));
    }
    Ok(())
}