/// Unidades válidas del hospital (PDF: 1, 2, 4 y 5).
pub const VALID_UNITS: [u16; 4] = [1, 2, 4, 5];

/// Índice de cama dentro de cada unidad (PDF: 01..38).
pub const FIRST_BED_INDEX: u16 = 1;
pub const LAST_BED_INDEX: u16 = 38;

/// Útil para tests o métricas (4 unidades * 38 camas = 152).
pub const TOTAL_BEDS: usize = VALID_UNITS.len() * (LAST_BED_INDEX as usize);
