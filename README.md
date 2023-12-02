# Temperature Conversion Tool

This tool is a Rust-based command-line application that allows for conversion between various temperature units. It utilizes a graph-based approach to manage unit conversions, supporting both direct and intermediate conversions.

## Installation

To install and run this tool, follow these steps:

1. **Clone the Repository** (if applicable):
```
git clone https://github.com/ShoggyR79/RustConversionWiz.git
cd temperature-conversion
```

2. **Build the Project**:
```
cargo build --release
```
3. **Place/Modify** ```data.json``` **file in ./conversion_wiz**
4. **Run the Application**:
```
cargo run --release
```
5. **Alternatively: you can specify custom json input file**
```
./target/release/conversion_wiz.exe -c <json file>
```

## Understanding Key Concepts

### Units
Units in this context refer to measurement standards for temperature. Common examples include Celsius (°C), Fahrenheit (°F), and Kelvin (K). Each unit has a name and potentially multiple aliases for ease of use.

### Conversion Scale
The conversion scale represents the factor by which a value in one unit is multiplied to convert it to another unit. For instance, converting meters to kilometers involves multiplying by a scale factor of 0.001.

### Conversion Offset
The conversion offset is an additional value added or subtracted after applying the scale factor during unit conversion. This is commonly used when the conversion isn't a simple scale transformation. For example, Celsius to Fahrenheit conversion involves an offset of 32 after scaling.

**Note: factors are to be represented in f64**

## JSON Configuration Format

The tool requires a JSON configuration file that defines units and their conversions. Here's the format for the JSON file:

```json
{
 "units": [
     {
         "name": "unit_name",
         "aliases": ["alias1", "alias2"]
     },
     // ... other units ...
 ],
 "conversions_scale": [
     {
         "from": "unit1",
         "to": "unit2",
         "factor": scale_factor
     },
     // ... other scale-based conversions ...
 ],
 "conversions_offset": [
     {
         "from": "unit1",
         "to": "unit2",
         "offset": offset_value
     },
     // ... other offset-based conversions ...
 ]
}
```

**Note: a sample json file is provided to you at** ```/conversion_wiz/data.json```

## Tips for Intermediate Conversion
For complex conversions that cannot be expressed through a single scale or offset, such as Celsius to Fahrenheit or Kelvin to Farenheight, intermediate units must be used.

For example, to convert from Celsius to Fahrenheit ```F = (1.8 * C) + 32```:

1. Convert Celsius to C1: multiply C by 1.8.

2. Convert C1 to F: add 32 to C1.

**This means that there will be 3 units and 2 total edge from C <-> F**
This multi-step conversion ensures accuracy and flexibility in unit conversions and also enforce order of operations.


# More Info
Run ```cargo doc --open``` to view the complete documentation of the module"# RustConversionWiz" 
