#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Subcommand,

    #[arg(short, long, default_value_t = String::from("output"))]
    pub output_dir: String,

    #[arg(short, long, default_value_t = String::from("versions"))]
    pub versions_dir: String,
}

#[derive(clap::Subcommand, Debug)]
pub enum Subcommand {
    Download {
        #[arg(short, long, default_value_t = String::from("latest"))]
        version: String,
        #[arg(value_enum, short, long, default_value_t = ReleaseType::Release)]
        r#type: ReleaseType,
    },
    Generate {
        #[arg(short, long, default_value_t = String::from("latest"))]
        version: String,
        #[arg(short, long, default_value_t = String::from("--all"))]
        generator_args: String,
    },
    Transform {
        #[command(subcommand)]
        sub: TransformSubcommand,
    }
}

#[derive(clap::ValueEnum, Debug, Clone, Eq, PartialEq)]
pub enum ReleaseType {
    Release,
    Snapshot,
}

#[derive(Debug, clap::Subcommand)]
pub enum TransformSubcommand {
    Registry {
        #[arg(short, long, default_value_t = String::from("registries.json"))]
        output_file: String,
        #[arg(short, long, value_delimiter = ',')]
        registries: Vec<String>,
    },
    Blocks {
        #[arg(short, long, default_value_t = String::from("blocks.json"))]
        output_file: String,
    },
}
