mod executor;
mod test_unit;
use std::time::Duration;

use crate::Config;

pub use self::executor::Paths;
use self::executor::TestExecutor;

const CHECK_STATUS_INTERVAL: Duration = Duration::from_millis(150);

pub trait Executor {
    fn execute_testing(self, config: &Config);
}

pub fn load_tests(config: &Config) -> impl Executor {
    let mut executor = TestExecutor::new().unwrap();
    let paths = Paths::new(config);

    for name in config.test_names() {
        executor.push_test(paths.clone(), name.into());
    }

    executor
}
