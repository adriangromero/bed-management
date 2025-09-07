mod domain;

use domain::constants::TOTAL_BEDS;
use domain::hospital::Hospital;
use domain::patient::{Gender, Patient};

fn main() {
    let mut hospital = Hospital::new();
    println!("Total beds: {}", TOTAL_BEDS);

    // --- Create test patients ---
    let maria = Patient::new(
        10001,
        "Maria Garcia".to_string(),
        30,
        Gender::Female,
        false,
        false,
    );
    let juan = Patient::new(
        10002,
        "John Lopez".to_string(),
        35,
        Gender::Male,
        false,
        false,
    );
    let carlos_nino = Patient::new(
        10003,
        "Carlos (10 years)".to_string(),
        10,
        Gender::Male,
        false,
        false,
    );
    let ana = Patient::new(
        10004,
        "Ana Martinez".to_string(),
        34,
        Gender::Female,
        false,
        false,
    );
    let pedro = Patient::new(
        10005,
        "Peter Sanchez".to_string(),
        35,
        Gender::Male,
        false,
        false,
    );

    // --------------------------------
    // 1) Test patient admission
    // --------------------------------
    println!(
        "Admit Maria to 201 -> {:?}",
        hospital.admit_patient(maria.clone(), 201)
    );

    // Should fail - different gender cannot share room
    println!(
        "Admit John to 202 -> {:?}",
        hospital.admit_patient(juan.clone(), 202)
    );

    // Should work - empty room
    println!(
        "Admit John to 205 -> {:?}",
        hospital.admit_patient(juan.clone(), 205)
    );

    // Test rule: children under 13 must go to unit 5
    println!(
        "Admit Carlos to 101 -> {:?}",
        hospital.admit_patient(carlos_nino.clone(), 101)
    );
    println!(
        "Admit Carlos to 501 -> {:?}",
        hospital.admit_patient(carlos_nino.clone(), 501)
    );

    // Admit another adult female
    println!(
        "Admit Ana to 203 -> {:?}",
        hospital.admit_patient(ana.clone(), 203)
    );

    // --------------------------------
    // 2) Test VIP functionality
    // --------------------------------
    println!(
        "Mark John as VIP -> {:?}",
        hospital.set_patient_vip(10002, true)
    );

    // Verify bed 206 is blocked (VIP blocks adjacent bed)
    let paciente_prueba = Patient::new(
        19000,
        "Test Patient".to_string(),
        40,
        Gender::Male,
        false,
        false,
    );
    println!(
        "Admit test patient to 206 -> {:?}",
        hospital.admit_patient(paciente_prueba.clone(), 206)
    );

    // --------------------------------
    // 3) Query available beds for a patient
    // --------------------------------
    let camas_disponibles = hospital.get_available_beds_for_patient(&paciente_prueba);
    print!(
        "Available beds for adult male ({}):",
        camas_disponibles.len()
    );
    for (index, numero_cama) in camas_disponibles.iter().take(10).enumerate() {
        if index == 0 {
            print!(" {}", numero_cama);
        } else {
            print!(", {}", numero_cama);
        }
    }
    if camas_disponibles.len() > 10 {
        print!(" ...");
    }
    println!();

    // --------------------------------
    // 4) Test infectious disease marking
    // --------------------------------
    println!(
        "Mark Maria as infectious -> {:?}",
        hospital.mark_patient_as_infected(10001)
    );
    println!(
        "Unmark Maria as infectious -> {:?}",
        hospital.unmark_patient_as_infected(10001)
    );

    // --------------------------------
    // 5) Test patient movement
    // --------------------------------
    println!(
        "Move John to bed 207 -> {:?}",
        hospital.move_patient(10002, 207)
    );

    // --------------------------------
    // 6) Test patient switching
    // --------------------------------
    println!(
        "Admit Pedro to 209 -> {:?}",
        hospital.admit_patient(pedro.clone(), 209)
    );
    println!(
        "Switch John and Pedro -> {:?}",
        hospital.switch_patients(10002, 10005)
    );

    // --------------------------------
    // 7) Test patient search functionality
    // --------------------------------
    if let Some((numero_cama, paciente)) = hospital.find_patient_info(10002) {
        println!(
            "Find patient 10002 => bed {}, name: {}",
            numero_cama, paciente.name
        );
    } else {
        println!("Patient 10002 not found");
    }

    // --------------------------------
    // 8) Test patient discharge
    // --------------------------------
    println!("Discharge John -> {:?}", hospital.discharge_patient(10002));

    // --------------------------------
    // 9) Display bed statistics
    // --------------------------------
    let (ocupadas, libres, bloqueadas) = hospital.count_beds_by_state();
    println!(
        "Summary: {} occupied, {} vacant, {} blocked",
        ocupadas, libres, bloqueadas
    );

    // --------------------------------
    // 10) Optional verbose output - show all beds if env var is set
    // --------------------------------
    if std::env::var("MOSTRAR_CAMAS").is_ok() {
        println!("\n=== FULL HOSPITAL STATE ===");
        hospital.list_all_beds();
    }
}
