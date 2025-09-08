use bed_management::domain::hospital::Hospital;
use bed_management::domain::patient::{Gender, Patient};

#[test]
fn test_complete_lifecycle() {
    let mut hospital = Hospital::new();

    // Test complete patient lifecycle: admit -> mark infectious -> unmark -> switch -> discharge
    let p1 = Patient::new(10001, "P1".to_string(), 30, Gender::Male, false, false);
    let p2 = Patient::new(10002, "P2".to_string(), 35, Gender::Male, false, false);
    let p3 = Patient::new(10003, "P3".to_string(), 25, Gender::Female, false, false);
    let p4 = Patient::new(10004, "P4".to_string(), 28, Gender::Female, false, false);

    // Admit patients in different rooms
    assert!(hospital.admit_patient(p1, 101).is_ok());
    assert!(hospital.admit_patient(p2, 102).is_ok()); // Same room as p1
    assert!(hospital.admit_patient(p3, 201).is_ok());
    assert!(hospital.admit_patient(p4, 202).is_ok()); // Same room as p3

    // Mark p1 as infectious - should move p2 to another bed
    assert!(hospital.mark_patient_as_infected(10001).is_ok());

    // Unmark as infectious
    assert!(hospital.unmark_patient_as_infected(10001).is_ok());

    // Switch two patients
    assert!(hospital.switch_patients(10003, 10004).is_ok());

    // Discharge patient
    assert!(hospital.discharge_patient(10001).is_ok());
}

#[test]
fn test_pediatric_unit() {
    let mut hospital = Hospital::new();

    // Rule: Children under 13 MUST be in unit 5
    let child = Patient::new(20001, "Child".to_string(), 10, Gender::Male, false, false);

    // Should fail in units 1, 2, 4

    // Should fail in units 1, 2, 4
    assert!(hospital.admit_patient(child.clone(), 101).is_err());
    assert!(hospital.admit_patient(child.clone(), 201).is_err());
    assert!(hospital.admit_patient(child.clone(), 401).is_err());

    // Should succeed in unit 5
    assert!(hospital.admit_patient(child, 501).is_ok());

    // Rule: Under-16 can only share with other under-16
    let teen = Patient::new(20002, "Teen".to_string(), 14, Gender::Female, false, false);
    assert!(hospital.admit_patient(teen, 101).is_ok());

    // Adult cannot share room with teen
    let adult = Patient::new(20003, "Adult".to_string(), 25, Gender::Female, false, false);
    assert!(hospital.admit_patient(adult, 102).is_err());
}

#[test]
fn test_vip_and_infectious() {
    let mut hospital = Hospital::new();

    // Rule: VIP patients block the adjacent bed
    let vip = Patient::new(30001, "VIP".to_string(), 50, Gender::Male, false, true);
    assert!(hospital.admit_patient(vip, 101).is_ok());

    // Bed 102 should be blocked
    let normal = Patient::new(30002, "Normal".to_string(), 40, Gender::Male, false, false);
    assert!(hospital.admit_patient(normal.clone(), 102).is_err());

    // But other beds are available
    assert!(hospital.admit_patient(normal, 103).is_ok());
}

#[test]
fn test_gender_mixing() {
    let mut hospital = Hospital::new();

    // Rule: Same gender only in shared rooms
    let male = Patient::new(40001, "Male".to_string(), 30, Gender::Male, false, false);
    let female = Patient::new(
        40002,
        "Female".to_string(),
        30,
        Gender::Female,
        false,
        false,
    );

    assert!(hospital.admit_patient(male, 201).is_ok());
    // Female cannot share room with male
    assert!(hospital.admit_patient(female, 202).is_err());
}

#[test]
fn test_move_patient_rules() {
    let mut hospital = Hospital::new();

    let p = Patient::new(50001, "P".to_string(), 30, Gender::Male, false, false);

    // Admit and move to valid bed
    assert!(hospital.admit_patient(p, 101).is_ok());
    assert!(hospital.move_patient(50001, 201).is_ok());

    // Cannot move to non-existent bed
    assert!(hospital.move_patient(50001, 301).is_err());
}

#[test]
fn test_switch_patients_validation() {
    let mut hospital = Hospital::new();

    // Case 1: Same gender switch - should work
    let m1 = Patient::new(60001, "M1".to_string(), 30, Gender::Male, false, false);
    let m2 = Patient::new(60002, "M2".to_string(), 35, Gender::Male, false, false);

    hospital.admit_patient(m1, 101).unwrap();
    hospital.admit_patient(m2, 201).unwrap();
    assert!(hospital.switch_patients(60001, 60002).is_ok());

    // Case 2: Child under 13 cannot leave unit 5
    let child = Patient::new(60003, "Child".to_string(), 10, Gender::Male, false, false);
    let adult = Patient::new(60004, "Adult".to_string(), 30, Gender::Male, false, false);

    hospital.admit_patient(child, 501).unwrap();
    hospital.admit_patient(adult, 401).unwrap();
    // Switch would move child out of unit 5 - should fail
    assert!(hospital.switch_patients(60003, 60004).is_err());
}

#[test]
fn test_mark_infected_moves_roommate() {
    let mut hospital = Hospital::new();

    let p1 = Patient::new(70001, "P1".to_string(), 30, Gender::Male, false, false);
    let p2 = Patient::new(70002, "P2".to_string(), 35, Gender::Male, false, false);

    hospital.admit_patient(p1, 101).unwrap();
    hospital.admit_patient(p2, 102).unwrap(); // Roommate

    // When marking p1 as infected, p2 should be moved automatically
    assert!(hospital.mark_patient_as_infected(70001).is_ok());

    // Bed 102 should now be blocked (not available)
    let test = Patient::new(99999, "Test".to_string(), 30, Gender::Male, false, false);
    let available = hospital.get_available_beds_for_patient(&test);
    assert!(!available.contains(&102));
}

#[test]
fn test_all_rules_together() {
    let mut hospital = Hospital::new();

    // Complex scenario testmultiple rules
    let patients = [
        Patient::new(80001, "P1".to_string(), 30, Gender::Male, false, false),
        Patient::new(80002, "P2".to_string(), 35, Gender::Male, false, false),
        Patient::new(80003, "P3".to_string(), 25, Gender::Female, false, false),
        Patient::new(80004, "P4".to_string(), 10, Gender::Male, false, false), // Child
        Patient::new(80005, "P5".to_string(), 60, Gender::Male, false, true),  // VIP
    ];

    // Admit patients following all rules
    assert!(hospital.admit_patient(patients[0].clone(), 101).is_ok());
    assert!(hospital.admit_patient(patients[1].clone(), 102).is_ok());
    assert!(hospital.admit_patient(patients[2].clone(), 201).is_ok());
    assert!(hospital.admit_patient(patients[3].clone(), 501).is_ok()); // Child in unit 5
    assert!(hospital.admit_patient(patients[4].clone(), 401).is_ok()); // VIP blocks 402

    // Test various operations
    assert!(hospital.mark_patient_as_infected(80001).is_ok());
    assert!(hospital.move_patient(80003, 203).is_ok());
    assert!(hospital.unmark_patient_as_infected(80001).is_ok());
    assert!(hospital.discharge_patient(80005).is_ok());

    // Verify final state
    let test = Patient::new(99999, "Test".to_string(), 30, Gender::Male, false, false);
    let available = hospital.get_available_beds_for_patient(&test);
    assert!(available.contains(&401)); // VIP discharged
    assert!(available.contains(&402)); // No longer blocked
}
