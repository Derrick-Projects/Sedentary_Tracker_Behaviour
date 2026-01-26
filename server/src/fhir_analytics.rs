use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use uuid::Uuid;

use crate::state::AppState;

// LOINC Configuration - Load from environment variables
fn loinc_code() -> String {
    env::var("LOINC_CODE").unwrap_or_else(|_| "87705-0".to_string())
}

fn loinc_display() -> String {
    env::var("LOINC_DISPLAY").unwrap_or_else(|_| "Sedentary activity 24 hour".to_string())
}

fn loinc_system() -> String {
    env::var("LOINC_SYSTEM").unwrap_or_else(|_| "http://loinc.org".to_string())
}

fn fhir_system() -> String {
    env::var("FHIR_SYSTEM").unwrap_or_else(|_| "http://unitsofmeasure.org".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryParams {
    #[serde(default = "default_period")]
    period: String,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_period() -> String {
    "daily".to_string()
}

fn default_limit() -> i64 {
    30
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirObservation {
    resource_type: String,
    id: String,
    status: String,
    code: CodeableConcept,
    subject: Reference,
    effective_date_time: String,
    value_quantity: Option<ValueQuantity>,
    component: Vec<ObservationComponent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeableConcept {
    coding: Vec<Coding>,
    text: String,
}

#[derive(Debug, Serialize)]
pub struct Coding {
    system: String,
    code: String,
    display: String,
}

#[derive(Debug, Serialize)]
pub struct Reference {
    reference: String,
}

#[derive(Debug, Serialize)]
pub struct ValueQuantity {
    value: f64,
    unit: String,
    system: String,
    code: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationComponent {
    code: CodeableConcept,
    value_quantity: Option<ValueQuantity>,
    value_integer: Option<i32>,
    value_string: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FhirBundle {
    #[serde(rename = "resourceType")]
    resource_type: String,
    #[serde(rename = "type")]
    bundle_type: String,
    total: usize,
    entry: Vec<BundleEntry>,
}

#[derive(Debug, Serialize)]
pub struct BundleEntry {
    resource: FhirObservation,
}

/// Get user's activity summary observations in FHIR format
/// Endpoint: GET /api/fhir/analytics/user/:user_id
pub async fn get_user_analytics(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid user ID format"
                })),
            )
                .into_response();
        }
    };

    let result = sqlx::query!(
        r#"
        SELECT
            id,
            date,
            period_type,
            sedentary_minutes,
            fidget_minutes,
            active_minutes,
            total_minutes,
            sedentary_percentage,
            active_percentage,
            dominant_state,
            activity_score,
            alert_count,
            longest_sedentary_period,
            created_at
        FROM activity_summary
        WHERE user_id = $1 AND period_type = $2
        ORDER BY date DESC
        LIMIT $3
        "#,
        user_uuid,
        params.period,
        params.limit
    )
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(rows) => {
            let observations: Vec<FhirObservation> = rows
                .iter()
                .map(|row| {
                    let observation_id = format!("activity-summary-{}", row.id);
                    let subject_ref = format!("Patient/{}", user_id);

                    // Calculate sedentary hours per 24h (LOINC 87705-0 expected unit)
                    let sedentary_hours_24h: f64 = if row.total_minutes > 0.0 {
                        ((row.sedentary_minutes / row.total_minutes) * 24.0) as f64
                    } else {
                        0.0
                    };

                    FhirObservation {
                        resource_type: "Observation".to_string(),
                        id: observation_id,
                        status: "final".to_string(),
                        code: CodeableConcept {
                            coding: vec![Coding {
                                system: loinc_system(),
                                code: loinc_code(),
                                display: loinc_display(),
                            }],
                            text: loinc_display(),
                        },
                        subject: Reference {
                            reference: subject_ref,
                        },
                        effective_date_time: row.created_at.to_rfc3339(),
                        value_quantity: Some(ValueQuantity {
                            value: sedentary_hours_24h,
                            unit: "h/(24.h)".to_string(),
                            system: fhir_system(),
                            code: "h/(24.h)".to_string(),
                        }),
                        component: vec![
                            ObservationComponent {
                                code: CodeableConcept {
                                    coding: vec![Coding {
                                        system: "http://loinc.org".to_string(),
                                        code: "CUSTOM-ACTIVITY-SCORE".to_string(),
                                        display: "Activity Score".to_string(),
                                    }],
                                    text: "Activity Score (0-100)".to_string(),
                                },
                                value_integer: Some(row.activity_score),
                                value_quantity: None,
                                value_string: None,
                            },
                            ObservationComponent {
                                code: CodeableConcept {
                                    coding: vec![Coding {
                                        system: "http://loinc.org".to_string(),
                                        code: "CUSTOM-DOMINANT-STATE".to_string(),
                                        display: "Dominant Activity State".to_string(),
                                    }],
                                    text: "Dominant State".to_string(),
                                },
                                value_string: Some(row.dominant_state.clone()),
                                value_quantity: None,
                                value_integer: None,
                            },
                            ObservationComponent {
                                code: CodeableConcept {
                                    coding: vec![Coding {
                                        system: "http://loinc.org".to_string(),
                                        code: "CUSTOM-ALERT-COUNT".to_string(),
                                        display: "Sedentary Alert Count".to_string(),
                                    }],
                                    text: "Number of 20-minute sedentary alerts".to_string(),
                                },
                                value_integer: Some(row.alert_count),
                                value_quantity: None,
                                value_string: None,
                            },
                            ObservationComponent {
                                code: CodeableConcept {
                                    coding: vec![Coding {
                                        system: "http://loinc.org".to_string(),
                                        code: "CUSTOM-ACTIVE-MINUTES".to_string(),
                                        display: "Active Minutes".to_string(),
                                    }],
                                    text: "Total active minutes".to_string(),
                                },
                                value_quantity: Some(ValueQuantity {
                                    value: row.active_minutes as f64,
                                    unit: "min".to_string(),
                                    system: fhir_system(),
                                    code: "min".to_string(),
                                }),
                                value_integer: None,
                                value_string: None,
                            },
                        ],
                    }
                })
                .collect();

            let bundle = FhirBundle {
                resource_type: "Bundle".to_string(),
                bundle_type: "searchset".to_string(),
                total: observations.len(),
                entry: observations
                    .into_iter()
                    .map(|obs| BundleEntry { resource: obs })
                    .collect(),
            };

            (StatusCode::OK, Json(bundle)).into_response()
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to fetch analytics data"
                })),
            )
                .into_response()
        }
    }
}

/// Get latest analytics for all users (aggregated)
/// Endpoint: GET /api/fhir/analytics/latest
pub async fn get_latest_analytics(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        r#"
        SELECT DISTINCT ON (user_id)
            id,
            user_id,
            date,
            period_type,
            sedentary_minutes,
            activity_score,
            dominant_state,
            created_at
        FROM activity_summary
        WHERE period_type = $1
        ORDER BY user_id, date DESC
        LIMIT $2
        "#,
        params.period,
        params.limit
    )
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(rows) => {
            let summary: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    json!({
                        "userId": row.user_id,
                        "date": row.date,
                        "activityScore": row.activity_score,
                        "dominantState": row.dominant_state,
                        "sedentaryHours24h": (row.sedentary_minutes / 60.0),
                        "loincCode": loinc_code()
                    })
                })
                .collect();

            (StatusCode::OK, Json(summary)).into_response()
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to fetch analytics data"
                })),
            )
                .into_response()
        }
    }
}
