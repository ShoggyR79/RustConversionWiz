use clap::{App, Arg};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self};
use conversion_wiz::ConversionGraph;

#[derive(Serialize, Deserialize)]
struct UnitConfig {
    name: String,
    aliases: Vec<String>,
    intermediate: bool,
}

#[derive(Serialize, Deserialize)]
struct ConversionScale {
    from: String,
    to: String,
    factor: f64,
}

#[derive(Serialize, Deserialize)]
struct ConversionOffset {
    from: String,
    to: String,
    offset: f64,
}

#[derive(Serialize, Deserialize)]
struct Config {
    units: Vec<UnitConfig>,
    conversions_scale: Vec<ConversionScale>,
    conversions_offset: Vec<ConversionOffset>,
}

fn main() {
    let matches = App::new("Temperature Converter")
        .version("1.0")
        .about("Converts between different temperature units")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    let config_file = matches.value_of("data").unwrap_or("data.json");

    let config_data = fs::read_to_string(config_file).expect("Unable to read file");
    let config: Config = serde_json::from_str(&config_data).expect("JSON was not well-formatted");

    let mut graph = ConversionGraph::new();

    // Populate the graph with units
    for unit in config.units {
        graph
            .add_unit(&unit.name, unit.aliases.iter().map(AsRef::as_ref).collect(), unit.intermediate)
            .expect("Error adding unit");
    }

    // Add scale conversions
    for conv in config.conversions_scale {
        graph
            .add_scale_edge(&conv.from, &conv.to, conv.factor)
            .expect("Error adding scale conversion");
    }

    // Add offset conversions
    for conv in config.conversions_offset {
        graph
            .add_offset_edge(&conv.from, &conv.to, conv.offset)
            .expect("Error adding offset conversion");
    }

    loop {
        println!("Enter first unit of conversion query or 'exit' to quit:");
        println!("or type 'list' to list all units");
        let mut unit1 = String::new();
        io::stdin()
            .read_line(&mut unit1)
            .expect("Failed to read line");
        let unit1 = unit1.trim();

        if unit1.eq_ignore_ascii_case("exit") {
            break;
        } else if unit1.eq_ignore_ascii_case("list") {
            println!("Units:");
            let mut index = 1;
            for unit in graph.units_formatted(){
                println!("\t{}: {}", index, unit);
                index += 1;
            }
            continue;
        }
        if !graph.contains_unit(unit1) {
            println!("Please enter a valid unit.");
            continue;
        }
        println!("Enter second unit of conversion query:");
        let mut unit2 = String::new();
        io::stdin()
            .read_line(&mut unit2)
            .expect("Failed to read line");
        let unit2 = unit2.trim();
        if unit2.eq_ignore_ascii_case("exit") {
            break;
        }
        if !graph.contains_unit(unit2) {
            println!("Please enter a valid unit.");
            continue;
        }
        println!("Enter value to convert:");
        let mut value_str = String::new();
        io::stdin()
            .read_line(&mut value_str)
            .expect("Failed to read line");
        let value_str = value_str.trim();
        if value_str.eq_ignore_ascii_case("exit") {
            break;
        }

        let value = match value_str.parse::<f64>() {
            Ok(value) => value,
            Err(_) => {
                println!("Please enter a valid number.");
                continue;
            }
        };

        let result = match graph.convert(unit1, unit2, value) {
            Ok(result) => result,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };

        println!("{} {} = {} {}", value, unit1, result, unit2);
    }

}
