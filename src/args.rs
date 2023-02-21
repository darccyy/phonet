use clap::Parser;

use phonet::DisplayLevel::{self, *};

#[derive(Parser)]
#[clap(author, version)]
/// A program to validate phonotactic patterns
///
/// More information: https://github.com/darccyy/phonet
pub struct Args {
    /// Custom tests (optional)
    ///
    /// This overrides all tests in the file
    pub tests: Vec<String>,

    /// Name and path of file to run and test
    ///
    /// If name ends with a period, the 'phonet' extension is implied
    ///
    /// Eg. `phonet -f ./myfile.phonet` or `phonet -f ./myfile.` (same result)
    #[arg(short, long, default_value_t = String::from("phonet"))]
    pub file: String,

    /// What types of outputs to display
    ///
    /// Options can be single letter
    ///
    /// Eg. `phonet -d only-fails` or `phonet -do`
    #[arg(short, long, default_value_t = ShowAll, value_enum)]
    pub display_level: DisplayLevel,

    /// Minify file and save
    #[arg(short, long)]
    pub minify: bool,

    /// Include tests in minified file
    #[arg(short, long)]
    pub with_tests: bool,

    /// Generate random words
    ///
    /// Default count 1, specify with number
    #[arg(short, long)]
    pub generate: Option<Option<usize>>,

    /// Set minimum length (inclusive) for generated words
    ///
    /// Use with the `--generate` or `-g` flag
    ///
    /// Note: This increases generation time exponentially
    #[arg(long = "gmin")]
    pub generate_min_len: Option<usize>,

    /// Set maximum length (inclusive) for generated words
    ///
    /// Use with the `--generate` or `-g` flag
    #[arg(long = "gmax")]
    pub generate_max_len: Option<usize>,

    /// Display output in default color
    ///
    /// Use for piping standard output to a file
    #[arg(short, long)]
    pub no_color: bool,
}
