use super::*;

#[test]
fn test_coding_serialization() {
    let coding = Coding {
        system: "http://loinc.org".to_string(),
        code: "80502-8".to_string(),
        display: "Sedentary behavior duration".to_string(),
    };

    let json = serde_json::to_string(&coding).unwrap();
    assert!(json.contains("\"system\":\"http://loinc.org\""));
    assert!(json.contains("\"code\":\"80502-8\""));
    assert!(json.contains("\"display\":\"Sedentary behavior duration\""));
}

#[test]
fn test_coding_deserialization() {
    let json = r#"{"system": "http://loinc.org", "code": "80502-8", "display": "Test"}"#;
    let coding: Coding = serde_json::from_str(json).unwrap();

    assert_eq!(coding.system, "http://loinc.org");
    assert_eq!(coding.code, "80502-8");
    assert_eq!(coding.display, "Test");
}

#[test]
fn test_codeable_concept_single_coding() {
    let concept = CodeableConcept {
        coding: vec![Coding {
            system: "http://loinc.org".to_string(),
            code: "TEST".to_string(),
            display: "Test Code".to_string(),
        }],
    };

    assert_eq!(concept.coding.len(), 1);
    assert_eq!(concept.coding[0].code, "TEST");
}

#[test]
fn test_codeable_concept_multiple_codings() {
    let concept = CodeableConcept {
        coding: vec![
            Coding {
                system: "http://loinc.org".to_string(),
                code: "CODE1".to_string(),
                display: "First Code".to_string(),
            },
            Coding {
                system: "http://snomed.info/sct".to_string(),
                code: "CODE2".to_string(),
                display: "Second Code".to_string(),
            },
        ],
    };

    assert_eq!(concept.coding.len(), 2);
}

#[test]
fn test_codeable_concept_empty() {
    let concept = CodeableConcept { coding: vec![] };
    assert!(concept.coding.is_empty());
}

//Reference Tests

#[test]
fn test_reference_patient() {
    let reference = Reference {
        reference: "Patient/12345".to_string(),
    };

    assert!(reference.reference.starts_with("Patient/"));
}

#[test]
fn test_reference_device() {
    let reference = Reference {
        reference: "Device/arduino-001".to_string(),
    };

    assert!(reference.reference.starts_with("Device/"));
}

//FhirObservation Tests

#[test]
fn test_fhir_observation_with_string_value() {
    let obs = FhirObservation {
        resourceType: "Observation".to_string(),
        id: "obs-123".to_string(),
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
        effectiveDateTime: "2026-01-06T10:00:00Z".to_string(),
        valueString: Some("ACTIVE".to_string()),
        valueInteger: None,
    };

    let json = serde_json::to_string(&obs).unwrap();
    assert!(json.contains("\"resourceType\":\"Observation\""));
    assert!(json.contains("\"valueString\":\"ACTIVE\""));
    assert!(!json.contains("valueInteger")); // Should be skipped when None
}

#[test]
fn test_fhir_observation_with_integer_value() {
    let obs = FhirObservation {
        resourceType: "Observation".to_string(),
        id: "obs-timer-456".to_string(),
        status: "final".to_string(),
        code: CodeableConcept {
            coding: vec![Coding {
                system: "http://loinc.org".to_string(),
                code: "CUSTOM-TIMER".to_string(),
                display: "Inactive Duration".to_string(),
            }],
        },
        subject: Reference {
            reference: "Patient/example".to_string(),
        },
        effectiveDateTime: "2026-01-06T10:00:00Z".to_string(),
        valueString: None,
        valueInteger: Some(600),
    };

    let json = serde_json::to_string(&obs).unwrap();
    assert!(json.contains("\"valueInteger\":600"));
    assert!(!json.contains("valueString")); // Should be skipped when None
}

#[test]
fn test_fhir_observation_status_values() {
    for status in ["registered", "preliminary", "final", "amended", "cancelled"].iter() {
        let obs = FhirObservation {
            resourceType: "Observation".to_string(),
            id: "test".to_string(),
            status: status.to_string(),
            code: CodeableConcept { coding: vec![] },
            subject: Reference {
                reference: "Patient/1".to_string(),
            },
            effectiveDateTime: "2026-01-06T10:00:00Z".to_string(),
            valueString: None,
            valueInteger: None,
        };

        assert_eq!(obs.status, *status);
    }
}

#[test]
fn test_fhir_observation_roundtrip() {
    let original = FhirObservation {
        resourceType: "Observation".to_string(),
        id: "roundtrip-test".to_string(),
        status: "final".to_string(),
        code: CodeableConcept {
            coding: vec![Coding {
                system: "http://loinc.org".to_string(),
                code: "TEST".to_string(),
                display: "Test".to_string(),
            }],
        },
        subject: Reference {
            reference: "Patient/test".to_string(),
        },
        effectiveDateTime: "2026-01-06T12:00:00Z".to_string(),
        valueString: Some("test-value".to_string()),
        valueInteger: None,
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: FhirObservation = serde_json::from_str(&json).unwrap();

    assert_eq!(original, restored);
}

#[test]
fn test_fhir_observation_clone() {
    let obs = FhirObservation {
        resourceType: "Observation".to_string(),
        id: "clone-test".to_string(),
        status: "final".to_string(),
        code: CodeableConcept { coding: vec![] },
        subject: Reference {
            reference: "Patient/1".to_string(),
        },
        effectiveDateTime: "2026-01-06T10:00:00Z".to_string(),
        valueString: Some("STILL".to_string()),
        valueInteger: None,
    };

    let cloned = obs.clone();
    assert_eq!(obs, cloned);
}
