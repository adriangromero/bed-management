use crate::domain::constants::{FIRST_BED_INDEX, LAST_BED_INDEX, VALID_UNITS};
use crate::domain::patient::Patient;

/// Possible bed states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BedState {
    /// Bed occupied by a patient
    Occupied(Patient),
    /// Empty bed
    Vacant,
    /// Blocked bed (cannot be used)
    Blocked,
}

/// Hospital bed (we only store the state; the bed number is the key in the HashMap)
#[derive(Debug, Clone)]
pub struct Bed {
    pub state: BedState,
}

impl Bed {
    /// Creates a new VACANT bed, validating that the number follows PDF rules.
    /// - Format: UXX → U is the unit, XX is the index (01..38).
    /// - Valid units: 1, 2, 4, 5.
    pub fn new(number: u16) -> Self {
        // Validate bed number using shared CONSTANTS
        let unit = number / 100;
        let bed_in_unit = number % 100;

        // Valid units come from VALID_UNITS constant
        if !VALID_UNITS.contains(&unit) {
            panic!("Invalid unit: {}", unit);
        }

        // Bed indices come from FIRST_BED_INDEX..=LAST_BED_INDEX range
        if !(FIRST_BED_INDEX..=LAST_BED_INDEX).contains(&bed_in_unit) {
            panic!("Invalid bed number: {}", bed_in_unit);
        }

        Bed {
            state: BedState::Vacant,
        }
    }

    /// Is the bed available (vacant)?
    pub fn is_available(&self) -> bool {
        matches!(self.state, BedState::Vacant)
    }

    /// Is the bed blocked?
    pub fn is_blocked(&self) -> bool {
        matches!(self.state, BedState::Blocked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::patient::Gender;

    #[test]
    fn test_create_valid_bed() {
        let _bed = Bed::new(101); // unit 1, index 1
    }

    #[test]
    #[should_panic(expected = "Invalid unit")]
    fn test_invalid_unit_panics() {
        Bed::new(301); // Unit 3 doesn't exist
    }

    #[test]
    #[should_panic(expected = "Invalid bed number")]
    fn test_invalid_bed_number_panics() {
        Bed::new(139); // Index 39 out of range 1..=38
    }

    #[test]
    fn test_bed_state_transitions() {
        let mut bed = Bed::new(102);

        // Initially vacant
        assert!(bed.is_available());
        assert!(!bed.is_blocked());

        // Occupy with a dummy patient
        let patient = Patient::new(12345, "Test".into(), 25, Gender::Male, false, false);
        bed.state = BedState::Occupied(patient);
        assert!(!bed.is_available());
        assert!(!bed.is_blocked());

        // Block the bed
        bed.state = BedState::Blocked;
        assert!(!bed.is_available());
        assert!(bed.is_blocked());

        // Make vacant again
        bed.state = BedState::Vacant;
        assert!(bed.is_available());
        assert!(!bed.is_blocked());
    }

    #[test]
    fn test_bed_state_equality() {
        let bed1 = Bed::new(101);
        let bed2 = Bed::new(102);

        // Both start vacant
        assert_eq!(bed1.state, bed2.state);
        assert_eq!(bed1.state, BedState::Vacant);
    }

    #[test]
    fn test_valid_bed_numbers() {
        // Test all valid units with edge cases
        let valid_beds = vec![
            101, // Unit 1, first bed
            138, // Unit 1, last bed
            201, // Unit 2, first bed
            238, // Unit 2, last bed
            401, // Unit 4, first bed
            438, // Unit 4, last bed
            501, // Unit 5, first bed
            538, // Unit 5, last bed
        ];

        for bed_num in valid_beds {
            let bed = Bed::new(bed_num);
            assert!(bed.is_available());
        }
    }
}
