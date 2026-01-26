use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FhirObservation {
    pub resourceType: String,
    pub id: String,
    pub status: String,
    pub code: CodeableConcept,
    pub subject: Reference,
    pub effectiveDateTime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valueString: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valueInteger: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CodeableConcept {
    pub coding: Vec<Coding>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Coding {
    pub system: String,
    pub code: String,
    pub display: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Reference {
    pub reference: String,
}

// GET /api/fhir/observation/latest
pub async fn get_latest_observation(
    State(state): State<AppState>,
) -> Result<Json<Vec<FhirObservation>>, StatusCode> {
    // 1. Fetch the latest reading from the NEW table (sedentary_log)
    let rec = sqlx::query!(
        r#"
        SELECT id, state, timer_seconds, created_at 
        FROM sedentary_log 
        ORDER BY created_at DESC 
        LIMIT 1
        "#
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match rec {
        Some(row) => {
            let timestamp = row.created_at.map(|t| t.to_rfc3339()).unwrap_or_default();
            let id = row.id.to_string();

            // 2. Map "Sedentary State" to FHIR Observation
            let state_obs = FhirObservation {
                resourceType: "Observation".to_string(),
                id: format!("{}-state", id),
                status: "final".to_string(),
                code: CodeableConcept {
                    coding: vec![Coding {
                        system: "http://loinc.org".to_string(),
                        code: "CUSTOM-STATE".to_string(),
                        display: "Sedentary State".to_string(),
                    }],
                },
                subject: Reference {
                    reference: "Patient/example".to_string(),
                },
                effectiveDateTime: timestamp.clone(),
                valueString: Some(row.state),
                valueInteger: None,
            };

            // 3. Map "Inactive Timer" to FHIR Observation
            let timer_obs = FhirObservation {
                resourceType: "Observation".to_string(),
                id: format!("{}-timer", id),
                status: "final".to_string(),
                code: CodeableConcept {
                    coding: vec![Coding {
                        system: "http://loinc.org".to_string(),
                        code: "CUSTOM-TIMER".to_string(),
                        display: "Inactive Duration (Seconds)".to_string(),
                    }],
                },
                subject: Reference {
                    reference: "Patient/example".to_string(),
                },
                effectiveDateTime: timestamp,
                valueString: None,
                valueInteger: Some(row.timer_seconds.unwrap_or(0) as i64),
            };

            // Return both observations
            Ok(Json(vec![state_obs, timer_obs]))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[cfg(test)]
#[path = "fhir_tests.rs"]
mod tests;
