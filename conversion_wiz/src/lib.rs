/// Conversion Model

use std::collections::HashMap;
use std::fmt;

/// Define a custom error type for conversion errors.
#[derive(Debug)]
pub enum ConversionError {
    EmptyUnitName,
    EmptyAlias,
    DuplicateUnit(String),
    DuplicateAlias(String),
    UnitNotFound(String),
    ConversionRateZero,
    ConversionRateBothValues,
    ConversionPathNotFound(String, String),
    MissingConversionFactor,
}

impl std::error::Error for ConversionError {}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::EmptyUnitName => write!(f, "Unit name cannot be empty"),
            ConversionError::EmptyAlias => write!(f, "Unit alias cannot be empty"),
            ConversionError::DuplicateUnit(name) => write!(f, "Unit {} already exists", name),
            ConversionError::DuplicateAlias(alias) => write!(f, "Alias {} already exists", alias),
            ConversionError::UnitNotFound(name) => write!(f, "Cannot find unit {}", name),
            ConversionError::ConversionRateZero => write!(f, "Conversion rate cannot be 0"),
            ConversionError::ConversionPathNotFound(from, to) => write!(f, "No conversion path found from '{}' to '{}'", from, to),
            ConversionError::ConversionRateBothValues => write!(f, "One of the conversion rates must be unchaged (1 for scale, 0 for offset)"),
            ConversionError::MissingConversionFactor => write!(f, "Conversion factor missing in the graph"),
        }
    }
}

/// `Unit` struct to represent a measurement unit.
/// It includes the official name of the unit and any aliases it may have.
pub struct Unit {
    /// The canonical name of the unit, e.g., "Kelvin".
    name: String,
    /// A list of alternative names or abbreviations for the unit, e.g., ["K"].
    aliases: Vec<String>,
    /// boolean to indicate if unit is intermediate - i.e. not shown to user
    intermediate: bool,
}


/// `Unit` implementation
impl Unit {
    /// Create a new `Unit` with the given name and aliases.
    ///
    /// # Error
    ///
    /// Error if the name is empty or if any of the aliases are empty.
    pub fn new(name: &str, aliases: Vec<&str>, intermediate: bool) -> Result<Self, ConversionError> {
        if name.is_empty() {
            return Err(ConversionError::EmptyUnitName);
        }
        for alias in &aliases {
            if alias.is_empty() {
                return Err(ConversionError::EmptyAlias);
            }
        }

        let mut aliases = aliases.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        if !aliases.contains(&name.to_string()) {
            aliases.push(name.to_string());
        }

        Ok(Self {
            name: name.to_string(),
            aliases,
            intermediate,
        })
    }

    /// Get the list of aliases for the unit.
    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    /// get formatted string of unit and aliases
    /// Kilojoule (kJ, kJoule)
    pub fn format_string(&self) -> String {
        let mut formatted = self.name.clone();
        if !self.aliases.is_empty() {
            formatted.push_str(" (");
            formatted.push_str(&self.aliases.join(", "));
            formatted.push_str(")");
        }
        formatted
    }
}

/// `ConversionFactor` struct to represent a conversion rate between two units.
/// it is used to convert from one unit to another.
/// It includes a scale factor and an offset.
pub struct ConversionFactor {
    scale: f64, // for multiplication
    offset: f64, // for addition
}

impl ConversionFactor {
    pub fn new(scale: f64, offset: f64) -> Self {
        Self {
            scale,
            offset,
        }
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn offset(&self) -> f64 {
        self.offset
    }
}

/// `ConversionGraph` struct to represent the entire conversion system.
/// It includes a collection of units and the edges that represent conversion rates between units.
/// The conversion rates are stored in a nested `HashMap` where the key is the target unit
/// and the value is the conversion factor to go from the outer unit to the inner unit.
pub struct ConversionGraph {
    /// A map of unit names to `Unit` structs, allowing quick access to unit details.
    name_to_units: HashMap<String, Unit>,
    /// a map of aliases to unit names
    aliases_to_name: HashMap<String, String>,
    /// A nested map where each unit name maps to another `HashMap`.
    /// This inner `HashMap` represents the conversion rates to other units.
    /// For example, edges["meter"]["kilometer"] might be 0.001.
    edges: HashMap<String, HashMap<String, ConversionFactor>>,
}

/// `ConversionGraph` implementation
impl ConversionGraph {
    /// Constructs a new, empty `ConversionGraph`.
    ///
    /// # Examples
    ///
    /// ```
    /// let graph = ConversionGraph::new();
    /// ```
    pub fn new() -> Self {
        Self {
            name_to_units: HashMap::new(),
            aliases_to_name: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Adds a new unit to the `ConversionGraph`.
    ///
    /// # Arguments
    ///
    /// * `name` - The canonical name of the unit.
    /// * `aliases` - A vector of aliases (alternative names) for the unit.
    ///
    /// # Errors
    ///
    /// Returns `ConversionError::EmptyUnitName` if the unit name is empty.
    /// Returns `ConversionError::DuplicateUnit` if the unit name already exists in the graph.
    /// 
    /// # Examples
    ///
    /// ```
    /// graph.add_unit("Meter", vec!["m", "metre"]).expect("Failed to add unit");
    /// ```
    pub fn add_unit(&mut self, name: &str, aliases: Vec<&str>, intermediate: bool) -> Result<(), ConversionError> {
        if name.is_empty() {
            return Err(ConversionError::EmptyUnitName);
        }
        if self.name_to_units.contains_key(name) {
            return Err(ConversionError::DuplicateUnit(name.to_string()));
        }

        let unit = Unit::new(name, aliases, intermediate)?;
        for alias in unit.aliases() {
            if self.aliases_to_name.contains_key(alias) {
                return Err(ConversionError::DuplicateAlias(alias.to_string()));
            }
            self.aliases_to_name.insert(alias.to_string(), name.to_string());
        }
        self.name_to_units.entry(name.to_string()).or_insert(unit);
        Ok(())
    }

    pub fn contains_unit(&self, name: &str) -> bool {
        // see if name is in aliases_to_name
        self.aliases_to_name.contains_key(name) 
    }
    /// Adds a new conversion rate between two units.
    ///
    /// # Arguments
    ///
    /// * `from` - The unit name to convert from.
    /// * `to` - The unit name to convert to.
    /// * `scale` - The scale factor for the conversion.
    /// * `offset` - The offset for the conversion.
    ///
    /// # Errors
    ///
    /// Returns `ConversionError::UnitNotFound` if either unit is not found in the graph.
    /// Returns `ConversionError::ConversionRateZero` if the conversion rate is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// graph.add_edge("Meter", "Kilometer", 0.001, 0.0).expect("Failed to add conversion");
    /// ```
    pub fn add_edge(&mut self, from: &str, to: &str, scale: f64, offset: f64) -> Result<(), ConversionError> {
        if scale == 0.0 {
            return Err(ConversionError::ConversionRateZero);
        }
        if scale != 1.0 && offset != 0.0 {
            return Err(ConversionError::ConversionRateBothValues);
        }
        let from_name = self.aliases_to_name.get(from)
            .ok_or_else(|| ConversionError::UnitNotFound(from.to_string()))?;
        let to_name = self.aliases_to_name.get(to)
            .ok_or_else(|| ConversionError::UnitNotFound(to.to_string()))?;

        self.edges.entry(from_name.to_string()).or_insert_with(HashMap::new);
        self.edges.entry(to_name.to_string()).or_insert_with(HashMap::new);
        
        let conversion = ConversionFactor::new(scale, offset);
        let opposite_conversion = ConversionFactor::new(1.0 / scale, -offset);
        
        self.edges.get_mut(from_name).unwrap().insert(to_name.to_string(), conversion);
        self.edges.get_mut(to_name).unwrap().insert(from_name.to_string(), opposite_conversion);
        Ok(())
    }
    
    /// see add_edge docs
    pub fn add_scale_edge(&mut self, from: &str, to: &str, scale: f64) -> Result<(), ConversionError> {
        self.add_edge(from, to, scale, 0.0)
    }
    pub fn add_offset_edge(&mut self, from: &str, to: &str, offset: f64) -> Result<(), ConversionError> {
        self.add_edge(from, to, 1.0, offset)
    }

    /// Get the conversion rate from one unit to another.
    ///
    /// # Error
    ///
    /// Error if either of the units do not exist in the graph.
    pub fn convert(&self, from: &str, to: &str, value: f64) -> Result<f64, ConversionError> {
        let from_name = self.aliases_to_name.get(from)
            .ok_or_else(|| ConversionError::UnitNotFound(from.to_string()))?;
        let to_name = self.aliases_to_name.get(to)
            .ok_or_else(|| ConversionError::UnitNotFound(to.to_string()))?;

        if from_name == to_name {
            return Ok(value); // No conversion needed if units are the same.
        }

        let mut queue = std::collections::VecDeque::new();
        let mut visited = HashMap::new();
        let mut parents = HashMap::new();

        // Initialize the BFS
        queue.push_back(from_name.as_str());
        visited.insert(from_name.as_str(), true);

        // Perform the BFS
        while let Some(current_unit) = queue.pop_front() {
            if current_unit == to_name.as_str() {
                // Found a path to the target unit.
                break;
            }

            // Visit all adjacent units (i.e., conversions)
            if let Some(edges) = self.edges.get(current_unit) {
                // print edges
            
                for (adj_unit, _) in edges {
                    if !visited.contains_key(adj_unit.as_str()) {
                        queue.push_back(adj_unit);
                        visited.insert(adj_unit, true);
                        parents.insert(adj_unit, current_unit);
                    }
                }
            }
        }

        if !parents.contains_key(&to_name.to_string()) {
            return Err(ConversionError::ConversionPathNotFound(from.to_string(), to.to_string()));
        }

        
        let mut cur_value = value;
        let mut current_unit = to_name.to_string();
        // need to add unit to vector to reverse
        let mut stack = Vec::new();
        while let Some(&parent_unit) = parents.get(&current_unit) {
            let factor = self.edges.get(parent_unit)
                .and_then(|edges| edges.get(&current_unit))
                .ok_or(ConversionError::MissingConversionFactor)?;
            stack.push(factor);
            current_unit = parent_unit.to_string();
        }
        while let Some(factor) = stack.pop() {
            cur_value = cur_value * factor.scale() + factor.offset();
        }

        Ok(cur_value)
    }


    /// get a list of all units formatted as strings
    pub fn units_formatted(&self) -> Vec<String> {
        // self.name_to_units.values().map(|u| u.format_string()).collect()
        //grab units from self.name_to_units if !intermediate
        let mut units = Vec::new();
        for unit in self.name_to_units.values() {
            if !unit.intermediate {
                units.push(unit.format_string());
            }
        }
        units
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    const REL_TOL: f64 = 1e-9;

    #[test]
    fn test_unit_new_valid() {
        let u = Unit::new("Kelvin", vec!["K"]).expect("Failed to create unit");
        assert_eq!(u.name(), "Kelvin");
        assert_eq!(u.aliases().len(), 2); // Includes the name itself as an alias
    }

    #[test]
    fn test_unit_new_empty_name() {
        assert!(matches!(Unit::new("", vec!["K"]), Err(ConversionError::EmptyUnitName)));
    }

    #[test]
    fn test_unit_new_empty_alias() {
        assert!(matches!(Unit::new("Kelvin", vec![""]), Err(ConversionError::EmptyAlias)));
    }

    #[test]
    fn test_conversion_graph_add_unit_valid() {
        let mut graph = ConversionGraph::new();
        assert!(graph.add_unit("Kelvin", vec!["K"]).is_ok());
    }

    #[test]
    fn test_conversion_graph_add_duplicate_unit() {
        let mut graph = ConversionGraph::new();
        let _ = graph.add_unit("Kelvin", vec!["K"]);
        assert!(graph.add_unit("Kelvin", vec!["K"]).is_err());
    }

    #[test]
    fn test_conversion_graph_add_duplicate_alias() {
        let mut graph = ConversionGraph::new();
        let _ = graph.add_unit("Kelvin", vec!["K"]);
        assert!(graph.add_unit("Rankine", vec!["K"]).is_err());
    }

    #[test]
    fn test_conversion_graph_add_edge_valid() {
        let mut graph = ConversionGraph::new();
        let _ = graph.add_unit("Kelvin", vec!["K"]);
        let _ = graph.add_unit("Rankine", vec!["R"]);
        assert!(graph.add_edge("K", "R", 1.8, 0.0).is_ok());
    }

    #[test]
    fn test_conversion_graph_add_edge_zero_rate() {
        let mut graph = ConversionGraph::new();
        let _ = graph.add_unit("Kelvin", vec!["K"]);
        let _ = graph.add_unit("Rankine", vec!["R"]);
        assert!(graph.add_edge("K", "R", 0.0, 0.0).is_err());
    }


    #[test]
    fn test_conversion_graph_convert_valid() {
        let mut graph = ConversionGraph::new();
        let _ = graph.add_unit("Kelvin", vec!["K"]);
        let _ = graph.add_unit("Rankine", vec!["R"]);
        let _ = graph.add_edge("K", "R", 1.8, 0.0);
        let converted_value = graph.convert("K", "R", 100.0).expect("Conversion should be successful");
        assert_relative_eq!(converted_value, 180.0, max_relative = REL_TOL); // Check only scale as offset is zero
    }

    /// Additional tests to cover conversion with offsets
    #[test]
    fn test_conversion_with_offset() {
        let mut graph = ConversionGraph::new();
        let _ = graph.add_unit("Celsius", vec!["C"]);
        let _ = graph.add_unit("Kelvin", vec!["K"]);
        let _ = graph.add_edge("C", "K", 1.0, 273.15);
        let converted_value = graph.convert("C", "K", 15.0).expect("Conversion should be successful");
        assert_relative_eq!(converted_value, 288.15, max_relative = REL_TOL);
        let converted_value_reverse = graph.convert("K", "C", 288.15).expect("Conversion should be successful");
        assert_relative_eq!(converted_value_reverse, 15.0, max_relative = REL_TOL);
    }


    #[test]
    fn test_conversion_graph_non_direct_route() {
        let mut graph = ConversionGraph::new();
        let _ = graph.add_unit("A", vec!["a"]);
        let _ = graph.add_unit("B", vec!["b"]);
        let _ = graph.add_unit("C", vec!["c"]);
        let _ = graph.add_edge("A", "B", 2.0, 0.0);
        let _ = graph.add_edge("B", "C", 1.0, 3.0);

        /// Test conversion from A to C which requires a conversion from A to B, then B to C.
        let converted_value = graph.convert("A", "C", 1.0).expect("Conversion should be successful");
        assert_relative_eq!(converted_value, 5.0, max_relative = REL_TOL); /// 1 A = 2 B, 1 B = 3 C, thus 1 A = 6 C
        let converted_value_reverse = graph.convert("C", "A", 5.0).expect("Conversion should be successful");
        assert_relative_eq!(converted_value_reverse, 1.0, max_relative = REL_TOL);
    }

    #[test]
    fn test_conversion_graph_nonexistent_route() {
        let mut graph = ConversionGraph::new();
        let _ = graph.add_unit("A", vec!["a"]);
        let _ = graph.add_unit("C", vec!["c"]);

        /// No direct conversion edge between A and C
        let conversion_result = graph.convert("A", "C", 0.0);
        assert!(matches!(conversion_result, Err(ConversionError::ConversionPathNotFound(_, _))));
    }
}
