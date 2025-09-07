use crate::domain::bed::{Bed, BedState};
use crate::domain::constants::{FIRST_BED_INDEX, LAST_BED_INDEX, VALID_UNITS};
use crate::domain::patient::Patient;
use std::collections::HashMap;

/// Main structure that manages all hospital beds
pub struct Hospital {
    /// Bed map, indexed by bed number UXX (u16)
    pub beds: HashMap<u16, Bed>,
}

impl Hospital {
    /// Creates a hospital with ALL valid beds according to shared constants
    pub fn new() -> Self {
        let mut beds = HashMap::new();

        // Create beds for each unit (VALID_UNITS) and for each index FIRST..=LAST
        for &unit in &VALID_UNITS {
            for idx in FIRST_BED_INDEX..=LAST_BED_INDEX {
                let bed_number = unit * 100 + idx; // UXX format
                beds.insert(bed_number, Bed::new(bed_number));
            }
        }

        Hospital { beds }
    }

    /// Returns the roommate bed number (same room, consecutive number)
    #[inline]
    pub fn roommate_of(bed_number: u16) -> u16 {
        if bed_number % 2 == 0 {
            bed_number - 1
        } else {
            bed_number + 1
        }
    }

    // ----------------- Operations -----------------

    /// Admits a new patient to an empty bed (applies ALL rules from the PDF)
    pub fn admit_patient(&mut self, patient: Patient, bed_number: u16) -> Result<(), String> {
        // 1) Check bed exists
        let bed = self.beds.get(&bed_number).ok_or("Bed does not exist")?;

        // 2) Check bed is available
        if !bed.is_available() {
            return Err("Bed is not available".to_string());
        }

        // 3) Children under 13 ONLY in unit 5
        let unit = bed_number / 100;
        if patient.age < 13 && unit != 5 {
            return Err("Patients under 13 must be in unit 5".to_string());
        }

        // 4) Compatibility with roommate (if exists)
        let roommate_bed_number = Self::roommate_of(bed_number);
        if let Some(roommate_bed) = self.beds.get(&roommate_bed_number) {
            if let BedState::Occupied(roommate) = &roommate_bed.state {
                // Same gender rule
                if patient.gender != roommate.gender {
                    return Err("Roommates must have the same gender".to_string());
                }
                // Under 16 can only share with under 16
                if (patient.age < 16) != (roommate.age < 16) {
                    return Err(
                        "Patients under 16 can only share with other under-16 patients".to_string(),
                    );
                }
                // Cannot share with infectious or VIP patients
                if roommate.is_infected || roommate.is_vip {
                    return Err("Cannot share a room with an infectious or VIP patient".to_string());
                }
            }
            // If new patient is infectious or VIP, adjacent bed must be free to block it
            if (patient.is_infected || patient.is_vip) && !roommate_bed.is_available() {
                return Err(
                    "Patient requires the adjacent bed to be blocked, but it is not free"
                        .to_string(),
                );
            }
        }

        // 5) Admit the patient
        self.beds.get_mut(&bed_number).unwrap().state = BedState::Occupied(patient.clone());

        // 6) Block adjacent bed if needed (VIP or infectious)
        if patient.is_infected || patient.is_vip {
            if let Some(roommate_bed) = self.beds.get_mut(&roommate_bed_number) {
                if roommate_bed.is_available() {
                    roommate_bed.state = BedState::Blocked;
                }
            }
        }

        Ok(())
    }

    /// Moves a patient from current bed to another empty bed (with simple rollback if fails)
    pub fn move_patient(
        &mut self,
        clinical_record: u32,
        new_bed_number: u16,
    ) -> Result<(), String> {
        // Find the patient
        let (mut current_bed_number, mut patient_opt) = (0u16, None);
        for (bed_num, bed) in &self.beds {
            if let BedState::Occupied(p) = &bed.state {
                if p.clinical_record_number == clinical_record {
                    current_bed_number = *bed_num;
                    patient_opt = Some(p.clone());
                    break;
                }
            }
        }
        let patient = patient_opt.ok_or("Patient not found".to_string())?;

        // Remember current bed's roommate (to unblock if needed)
        let old_roommate_number = Self::roommate_of(current_bed_number);

        // Free the origin bed
        self.beds.get_mut(&current_bed_number).unwrap().state = BedState::Vacant;

        // If patient was VIP or infectious, roommate might have been blocked -> try to unblock
        if patient.is_infected || patient.is_vip {
            if let Some(old_rm) = self.beds.get_mut(&old_roommate_number) {
                if old_rm.is_blocked() {
                    old_rm.state = BedState::Vacant;
                }
            }
        }

        // Try to admit in destination
        let result = self.admit_patient(patient.clone(), new_bed_number);

        // If fails, rollback to original bed
        if result.is_err() {
            self.beds.get_mut(&current_bed_number).unwrap().state =
                BedState::Occupied(patient.clone());
            // Re-block if needed
            if patient.is_infected || patient.is_vip {
                if let Some(old_rm) = self.beds.get_mut(&old_roommate_number) {
                    if old_rm.is_available() {
                        old_rm.state = BedState::Blocked;
                    }
                }
            }
            return result;
        }

        Ok(())
    }

    /// Switches beds between two patients (checks rules; if something fails, nothing changes)
    pub fn switch_patients(
        &mut self,
        clinical_record1: u32,
        clinical_record2: u32,
    ) -> Result<(), String> {
        // Find both patients
        let (mut bed1_number, mut p1) = (0u16, None);
        let (mut bed2_number, mut p2) = (0u16, None);

        for (bed_num, bed) in &self.beds {
            if let BedState::Occupied(p) = &bed.state {
                if p.clinical_record_number == clinical_record1 {
                    bed1_number = *bed_num;
                    p1 = Some(p.clone());
                } else if p.clinical_record_number == clinical_record2 {
                    bed2_number = *bed_num;
                    p2 = Some(p.clone());
                }
            }
        }

        let p1 = p1.ok_or("First patient not found".to_string())?;
        let p2 = p2.ok_or("Second patient not found".to_string())?;

        // Rule: children under 13 only in unit 5
        if p1.age < 13 && (bed2_number / 100) != 5 {
            return Err("Cannot move a patient under 13 outside unit 5".to_string());
        }
        if p2.age < 13 && (bed1_number / 100) != 5 {
            return Err("Cannot move a patient under 13 outside unit 5".to_string());
        }

        // Compatibility with destination roommates (if they're not roommates to each other)
        let roommate1_number = Self::roommate_of(bed1_number);
        let roommate2_number = Self::roommate_of(bed2_number);

        if bed2_number != roommate1_number {
            if let Some(r1) = self.beds.get(&roommate1_number) {
                if let BedState::Occupied(rm) = &r1.state {
                    if p2.gender != rm.gender {
                        return Err("Switch would violate gender rule".to_string());
                    }
                    if (p2.age < 16) != (rm.age < 16) {
                        return Err("Switch would violate age rule".to_string());
                    }
                    if rm.is_infected || rm.is_vip {
                        return Err(
                            "Switch would violate rule: cannot share with infectious/VIP"
                                .to_string(),
                        );
                    }
                }
            }
        }
        if bed1_number != roommate2_number {
            if let Some(r2) = self.beds.get(&roommate2_number) {
                if let BedState::Occupied(rm) = &r2.state {
                    if p1.gender != rm.gender {
                        return Err("Switch would violate gender rule".to_string());
                    }
                    if (p1.age < 16) != (rm.age < 16) {
                        return Err("Switch would violate age rule".to_string());
                    }
                    if rm.is_infected || rm.is_vip {
                        return Err(
                            "Switch would violate rule: cannot share with infectious/VIP"
                                .to_string(),
                        );
                    }
                }
            }
        }

        // Perform the switch
        self.beds.get_mut(&bed1_number).unwrap().state = BedState::Occupied(p2);
        self.beds.get_mut(&bed2_number).unwrap().state = BedState::Occupied(p1);

        Ok(())
    }

    /// Marks or unmarks a patient as VIP (if marking VIP, moves roommate if present)
    pub fn set_patient_vip(&mut self, clinical_record: u32, is_vip: bool) -> Result<(), String> {
        // Find the patient
        let (mut bed_number, mut patient) = (0u16, None);
        for (bed_num, bed) in &self.beds {
            if let BedState::Occupied(p) = &bed.state {
                if p.clinical_record_number == clinical_record {
                    bed_number = *bed_num;
                    patient = Some(p.clone());
                    break;
                }
            }
        }
        let mut p = patient.ok_or("Patient not found".to_string())?;
        if p.is_vip == is_vip {
            return Ok(()); // No changes needed
        }

        // Apply and save
        p.is_vip = is_vip;
        self.beds.get_mut(&bed_number).unwrap().state = BedState::Occupied(p.clone());

        // Handle adjacent bed
        let roommate_number = Self::roommate_of(bed_number);
        if is_vip {
            // If there's a roommate, move them first
            if let Some(rm) = self.beds.get(&roommate_number) {
                if let BedState::Occupied(roommate) = &rm.state {
                    let roommate_crn = roommate.clinical_record_number;
                    let candidates = self.get_available_beds_for_patient(&roommate.clone());
                    if let Some(dest) = candidates.first() {
                        self.move_patient(roommate_crn, *dest)?;
                    } else {
                        return Err("No available bed to relocate roommate".to_string());
                    }
                }
            }
            // Now block the adjacent bed
            if let Some(rm) = self.beds.get_mut(&roommate_number) {
                rm.state = BedState::Blocked;
            }
        } else {
            // If no longer VIP and not infectious, unblock
            if !p.is_infected {
                if let Some(rm) = self.beds.get_mut(&roommate_number) {
                    if rm.is_blocked() {
                        rm.state = BedState::Vacant;
                    }
                }
            }
        }
        Ok(())
    }

    /// Marks a patient as infectious (if there's a roommate, they must be moved; if no space, error)
    pub fn mark_patient_as_infected(&mut self, clinical_record: u32) -> Result<(), String> {
        // Find the patient
        let (mut bed_number, mut patient) = (0u16, None);
        for (bed_num, bed) in &self.beds {
            if let BedState::Occupied(p) = &bed.state {
                if p.clinical_record_number == clinical_record {
                    bed_number = *bed_num;
                    patient = Some(p.clone());
                    break;
                }
            }
        }
        let mut p = patient.ok_or("Patient not found".to_string())?;
        if p.is_infected {
            return Ok(());
        }
        p.is_infected = true;

        let roommate_number = Self::roommate_of(bed_number);

        // If there's a roommate, move them to another valid bed
        let mut roommate_to_move: Option<Patient> = None;
        if let Some(rm_bed) = self.beds.get(&roommate_number) {
            if let BedState::Occupied(rm) = &rm_bed.state {
                roommate_to_move = Some(rm.clone());
            }
        }
        if let Some(roommate) = roommate_to_move {
            // Find a compatible free bed
            let candidates = self.get_available_beds_for_patient(&roommate);
            if let Some(dest) = candidates.into_iter().next() {
                self.move_patient(roommate.clinical_record_number, dest)?;
            } else {
                return Err("No available bed to move roommate".to_string());
            }
        }

        // Save patient as infectious and block adjacent bed
        self.beds.get_mut(&bed_number).unwrap().state = BedState::Occupied(p);
        if let Some(rm_bed) = self.beds.get_mut(&roommate_number) {
            if rm_bed.is_available() {
                rm_bed.state = BedState::Blocked;
            }
        }

        Ok(())
    }

    /// Unmarks a patient as infectious (if not VIP, adjacent bed can be unblocked)
    pub fn unmark_patient_as_infected(&mut self, clinical_record: u32) -> Result<(), String> {
        // Find the patient
        let (mut bed_number, mut patient) = (0u16, None);
        for (bed_num, bed) in &self.beds {
            if let BedState::Occupied(p) = &bed.state {
                if p.clinical_record_number == clinical_record {
                    bed_number = *bed_num;
                    patient = Some(p.clone());
                    break;
                }
            }
        }
        let mut p = patient.ok_or("Patient not found".to_string())?;
        if !p.is_infected {
            return Ok(());
        }
        p.is_infected = false;
        self.beds.get_mut(&bed_number).unwrap().state = BedState::Occupied(p.clone());

        // If also not VIP -> unblock if it was blocked
        if !p.is_vip {
            let roommate_number = Self::roommate_of(bed_number);
            if let Some(rm_bed) = self.beds.get_mut(&roommate_number) {
                if rm_bed.is_blocked() {
                    rm_bed.state = BedState::Vacant;
                }
            }
        }
        Ok(())
    }

    /// Discharges a patient (bed becomes Vacant and adjacent blocking is recalculated if needed)
    pub fn discharge_patient(&mut self, clinical_record: u32) -> Result<(), String> {
        // Find the patient
        let (mut bed_number, mut patient) = (0u16, None);
        for (bed_num, bed) in &self.beds {
            if let BedState::Occupied(p) = &bed.state {
                if p.clinical_record_number == clinical_record {
                    bed_number = *bed_num;
                    patient = Some(p.clone());
                    break;
                }
            }
        }
        let p = patient.ok_or("Patient not found".to_string())?;

        // Free the bed
        self.beds.get_mut(&bed_number).unwrap().state = BedState::Vacant;

        // If patient was VIP or infectious, adjacent bed might have been blocked: unblock it
        if p.is_infected || p.is_vip {
            let roommate_number = Self::roommate_of(bed_number);
            if let Some(rm_bed) = self.beds.get_mut(&roommate_number) {
                if rm_bed.is_blocked() {
                    rm_bed.state = BedState::Vacant;
                }
            }
        }

        Ok(())
    }

    // ----------------- Queries -----------------

    /// Finds a patient by CRN and returns (bed number, patient)
    pub fn find_patient_info(&self, clinical_record: u32) -> Option<(u16, Patient)> {
        for (bed_number, bed) in &self.beds {
            if let BedState::Occupied(patient) = &bed.state {
                if patient.clinical_record_number == clinical_record {
                    return Some((*bed_number, patient.clone()));
                }
            }
        }
        None
    }

    /// Counts beds by state (occupied, vacant, blocked)
    pub fn count_beds_by_state(&self) -> (usize, usize, usize) {
        let mut occupied = 0;
        let mut vacant = 0;
        let mut blocked = 0;

        for bed in self.beds.values() {
            match bed.state {
                BedState::Occupied(_) => occupied += 1,
                BedState::Vacant => vacant += 1,
                BedState::Blocked => blocked += 1,
            }
        }

        (occupied, vacant, blocked)
    }

    /// Prints to console the state of all beds (useful for manual demo)
    pub fn list_all_beds(&self) {
        for &unit in &VALID_UNITS {
            println!("\n--- Unit {} ---", unit);
            for idx in FIRST_BED_INDEX..=LAST_BED_INDEX {
                let bed_number = unit * 100 + idx;
                if let Some(bed) = self.beds.get(&bed_number) {
                    match &bed.state {
                        BedState::Occupied(p) => {
                            print!(
                                "Bed {}: OCCUPIED - {} ({})",
                                bed_number, p.name, p.clinical_record_number
                            );
                            if p.is_infected {
                                print!(" [INFECTIOUS]");
                            }
                            if p.is_vip {
                                print!(" [VIP]");
                            }
                            println!();
                        }
                        BedState::Vacant => println!("Bed {}: VACANT", bed_number),
                        BedState::Blocked => println!("Bed {}: BLOCKED", bed_number),
                    }
                }
            }
        }
    }

    /// Returns all available beds for a specific patient
    pub fn get_available_beds_for_patient(&self, patient: &Patient) -> Vec<u16> {
        let mut available = Vec::new();

        for (&bed_number, bed) in &self.beds {
            if !bed.is_available() {
                continue;
            }

            // Children under 13 -> only unit 5
            let unit = bed_number / 100;
            if patient.age < 13 && unit != 5 {
                continue;
            }

            let roommate_number = Self::roommate_of(bed_number);
            let mut can_admit = true;

            if let Some(roommate_bed) = self.beds.get(&roommate_number) {
                if let BedState::Occupied(roommate) = &roommate_bed.state {
                    if patient.gender != roommate.gender {
                        can_admit = false;
                    }
                    if (patient.age < 16) != (roommate.age < 16) {
                        can_admit = false;
                    }
                    if roommate.is_infected || roommate.is_vip {
                        can_admit = false;
                    }
                }
                if (patient.is_infected || patient.is_vip) && !roommate_bed.is_available() {
                    can_admit = false;
                }
            }

            if can_admit {
                available.push(bed_number);
            }
        }

        available.sort_unstable();
        available
    }
}

impl Default for Hospital {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::constants::TOTAL_BEDS; // Import ONLY in tests

    #[test]
    fn test_hospital_creates_all_beds_with_constants() {
        let h = Hospital::new();
        assert_eq!(h.beds.len(), TOTAL_BEDS);
        // Spot checks:
        assert!(h.beds.contains_key(&201));
        assert!(h.beds.contains_key(&238));
        assert!(h.beds.contains_key(&505));
        assert!(h.beds.contains_key(&506));
    }
}
