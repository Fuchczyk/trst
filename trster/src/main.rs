mod testing;
use std::time::Duration;

use clap::Parser;
use serde::{Deserialize, Serialize};
use testing::Paths;

use crate::testing::Executor;
pub use trst_types::*;

/// Program used by trst program in order to conduct tests
#[derive(Parser, Debug)]
struct Args {
    /// Configuration in RON format
    #[arg(long, short)]
    configuration: String,

    /// Timeout limit for test
    #[arg(long)]
    timeout: f64,
}

static TEST_TIMEOUT: once_cell::sync::OnceCell<Duration> = once_cell::sync::OnceCell::new();

fn main() {
    pretty_env_logger::init();
    let args = Args::parse();
    let configuration_arg = args.configuration;

    let config: Config = match ron::from_str(&configuration_arg) {
        Err(e) => {
            eprintln!(
                "Error while converting rom to config: {}. ARG = ##[{}]##",
                e, &configuration_arg
            );
            return;
        }
        Ok(config) => config,
    };

    TEST_TIMEOUT
        .set(Duration::from_secs_f64(args.timeout))
        .unwrap();

    let executor = testing::load_tests(&config);

    executor.execute_testing(&config);
}
