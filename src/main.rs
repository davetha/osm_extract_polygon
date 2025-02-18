extern crate osmpbfreader;

use crate::output::output_handler::OutputHandlerConfiguration;
use crate::output::OverwriteConfiguration;
use clap::{crate_authors, crate_version, App, AppSettings, Arg};

mod converter;
mod osm_reader;
mod output;
mod utils;

fn main() {
    const INPUT_ARG: &str = "INPUT";
    const OUTPUT_FOLDER: &str = "OUTPUT";
    const MIN_ADMIN_LEVEL_ARG: &str = "MIN_ADMIN_LEVEL";
    const MAX_ADMIN_LEVEL_ARG: &str = "MAX_ADMIN_LEVEL";
    const OVERWRITE_ARG: &str = "OVERWRITE";
    const SKIP_ARG: &str = "SKIP";
    const GEOJSON_ARG: &str = "GEOJSON";

    // TODO: implement ability to skip features to filewriter
    //const SKIP_ADMIN_LEVEL_PROPERTY: &str = "SKIP_ADMIN_LEVEL_PROPERTY";


    let matches = App::new("OSM Extract Polygon")
        .version(crate_version!())
        .author(crate_authors!())
        .about(
            "Extracts administrative boundaries of OSM pbf files and produces polygon files compatible with Osmosis.",
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name(INPUT_ARG)
                .short("f")
                .long("file")
                .value_name("filename")
                .help("input file")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(MIN_ADMIN_LEVEL_ARG)
                .short("m")
                .long("min")
                .value_name("min_admin_level")
                .help("minimum administrative level (can take value from 1-11) [default: 8]")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(MAX_ADMIN_LEVEL_ARG)
                .short("x")
                .long("max")
                .value_name("max_admin_level")
                .help("max administrative level (can take value from 1-11) [default: 8]")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(OUTPUT_FOLDER)
                .short("p")
                .long("path")
                .value_name("path")
                .help("path to which the output will be saved to [default: '<input_filename>_polygons/']")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(OVERWRITE_ARG)
            .short("o")
            .long("overwrite")
            .takes_value(false)
            .help("set this flag to overwrite files without asking; if neither this nor --skip is set the user is being prompted should a file be overwritten.")
        )
        .arg(
            Arg::with_name(SKIP_ARG)
            .short("s")
            .long("skip")
            .takes_value(false)
            .help("set this flag to skip overwriting files; if neither this nor --overwrite is set the user is being prompted should a file be overwritten.")
        )
        .arg(
            Arg::with_name(GEOJSON_ARG)
            .short("g")
            .long("geojson")
            .takes_value(false)
            .help("set this flag to generate geojson output")
        )
        /*
        TODO: implement ability to skip features to filewriter
        .arg(
            Arg::with_name(SKIP_ADMIN_LEVEL_PROPERTY)
            .long("skip-geojson-admin-level")
            .takes_value(false)
            .help("Skip setting the admin_level in the GeoJSON output as a property")
        )
        */
        .get_matches();

    let min_admin_level = matches
        .value_of(MIN_ADMIN_LEVEL_ARG)
        .unwrap_or("8")
        .parse::<i8>()
        .unwrap();
    let max_admin_level = matches
        .value_of(MAX_ADMIN_LEVEL_ARG)
        .unwrap_or("8")
        .parse::<i8>()
        .unwrap();

    if min_admin_level > max_admin_level {
        println!(
            "error: --min={} has bigger value than --max={}",
            min_admin_level, max_admin_level
        );
        std::process::exit(-1);
    }

    let overwrite_all = matches.is_present(OVERWRITE_ARG);
    let skip_all = matches.is_present(SKIP_ARG);

    if overwrite_all && skip_all {
        println!("error: cannot set both -o (--overwrite) and -s (--skip)!");
        std::process::exit(-1);
    }

    let overwrite_configuration = if overwrite_all {
        OverwriteConfiguration::OverwriteAll
    } else if skip_all {
        OverwriteConfiguration::SkipAll
    } else {
        OverwriteConfiguration::Ask
    };

    let geojson_output = matches.is_present(GEOJSON_ARG);

    let output_handler_config = OutputHandlerConfiguration {
        overwrite_configuration,
        geojson_output,
    };

    let in_filename = matches.value_of(INPUT_ARG).unwrap();
    println!("Using input file: {}", in_filename);
    let default_path = format!("{}_polygons/", in_filename);
    let path = matches.value_of(OUTPUT_FOLDER).unwrap_or(&default_path);
    println!("Output path: {}", path);

    let relations = osm_reader::read_osm(in_filename, &min_admin_level, &max_admin_level);
    let polygons = converter::convert(relations);
    let result = output::output_handler::write(path, &polygons, output_handler_config);

    match result {
        Ok(size) => println!("success! wrote {} files!", size),
        Err(e) => println!("error! {:?}", e),
    }
}
