/// Patient gender
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
}

/// Structure that represents a patient
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patient {
    /// Clinical Record Number (5 digits)
    pub clinical_record_number: u32,
    /// Patient name
    pub name: String,
    /// Patient age
    pub age: u8,
    /// Patient gender
    pub gender: Gender,
    /// Whether the patient has an infectious disease
    pub is_infected: bool,
    /// Whether the patient is a VIP
    pub is_vip: bool,
}

impl Patient {
    /// Creates a new patient
    pub fn new(
        clinical_record_number: u32,
        name: String,
        age: u8,
        gender: Gender,
        is_infected: bool,
        is_vip: bool,
    ) -> Self {
        // Validate that the clinical record number has 5 digits
        if !(10000..=99999).contains(&clinical_record_number) {
            panic!("The clinical record number must have 5 digits");
        }

        Patient {
            clinical_record_number,
            name,
            age,
            gender,
            is_infected,
            is_vip,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_patient() {
        let patient = Patient::new(
            12345,
            "John Smith".to_string(),
            25,
            Gender::Male,
            false,
            false,
        );

        assert_eq!(patient.clinical_record_number, 12345);
        assert_eq!(patient.name, "John Smith");
        assert_eq!(patient.age, 25);
        assert_eq!(patient.gender, Gender::Male);
        assert!(!patient.is_infected);
        assert!(!patient.is_vip);
    }

    #[test]
    #[should_panic(expected = "The clinical record number must have 5 digits")]
    fn test_invalid_record_number() {
        Patient::new(123, "Test".to_string(), 25, Gender::Male, false, false);
    }
}
