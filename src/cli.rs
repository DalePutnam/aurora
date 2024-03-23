use clap::App;
use clap::Arg;

pub struct Parameters
{
    // Input lua file to be rendered
    pub input_file: String,

    // Output image file name
    // This overrides the output file in the LUA file
    pub output_file: Option<String>,

    // Output image resolution
    // This overrides the resolution in the LUA file
    pub resolution: Option<(u32, u32)>,

    // Single pixel to trace (used for debugging)
    pub single_pixel: Option<(u32, u32)>,
}

pub fn parse_args() -> Parameters
{
    let app = App::new("Aurora")
        .arg(
            Arg::with_name("pixel")
                .help("Render a single pixel")
                .long_help("Specify a single pixel to render for debugging purposes")
                .long("pixel")
                .short("p")
                .hidden(true)
                .takes_value(true)
                .validator(validate_pixel_value),
        )
        .arg(
            Arg::with_name("output file")
                .help("Output PNG file")
                .long_help(
                    "PNG file to render the image to, this will override the file specified in \
                     the LUA file",
                )
                .long("output")
                .short("o")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("resolution")
                .help("Resolution of rendered image. eg: 1920x1080")
                .long_help(
                    "Set the resolution of the rendered image, this will override the file \
                     specified in the LUA file",
                )
                .long("resolution")
                .short("r")
                .takes_value(true)
                .validator(validate_resolution_value),
        )
        .arg(
            Arg::with_name("input file")
                .help("Input LUA file")
                .long_help("LUA script to render")
                .required(true)
                .takes_value(true),
        );

    let matches = app.get_matches();

    let input_file = matches.value_of("input file").unwrap();

    let pixel = matches.value_of("pixel").map(|s| parse_pixel_value(s));

    let output_file = matches.value_of("output file").map(|s| String::from(s));

    let resolution = matches
        .value_of("resolution")
        .map(|s| parse_resolution_value(s));

    Parameters {
        input_file: String::from(input_file),
        output_file: output_file,
        resolution: resolution,
        single_pixel: pixel,
    }
}

fn validate_pixel_value(pixel_string: String) -> Result<(), String>
{
    let coords: Vec<&str> = pixel_string.split(',').collect();

    if coords.len() != 2 {
        return Err(String::from("Expected format is \"X,Y\""));
    }

    coords[0]
        .parse::<u32>()
        .map_err(|_| String::from(format!("Invalid X coordinate \"{}\"", coords[0])))?;

    coords[1]
        .parse::<u32>()
        .map_err(|_| String::from(format!("Invalid Y coordinate \"{}\"", coords[1])))?;

    Ok(())
}

fn parse_pixel_value(pixel_string: &str) -> (u32, u32)
{
    let coords: Vec<&str> = pixel_string.split(',').collect();

    (
        coords[0].parse::<u32>().unwrap(),
        coords[1].parse::<u32>().unwrap(),
    )
}

fn validate_resolution_value(resolution_string: String) -> Result<(), String>
{
    let resolution: Vec<&str> = resolution_string.split(&['x', 'X'][..]).collect();

    if resolution.len() != 2 {
        return Err(String::from("Expected format is \"HxV\""));
    }

    resolution[0].parse::<u32>().map_err(|_| {
        String::from(format!(
            "Invalid horizontal resolution \"{}\"",
            resolution[0]
        ))
    })?;

    resolution[1].parse::<u32>().map_err(|_| {
        String::from(format!(
            "Invalid veritical resolution \"{}\"",
            resolution[1]
        ))
    })?;

    Ok(())
}

fn parse_resolution_value(resolution_string: &str) -> (u32, u32)
{
    let resolution: Vec<&str> = resolution_string.split(&['x', 'X'][..]).collect();

    (
        resolution[0].parse::<u32>().unwrap(),
        resolution[1].parse::<u32>().unwrap(),
    )
}
