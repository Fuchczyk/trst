use once_cell::sync::OnceCell;

use trst_types::BackendMessage;

use crate::{Concurrency, Config};

use super::{
    test_unit::{TestingUnit, TestingUnitMessage},
    Executor,
};
use std::{
    io::Write,
    sync::{mpsc::Sender, Arc},
};

static EXECUTOR: OnceCell<()> = OnceCell::new();

#[derive(Debug)]
pub struct TestExecutor {
    tests: Vec<TestingUnit>,
}
#[derive(Debug)]
pub enum TestExecutorError {
    AlreadyCreated,
}

#[derive(Clone)]
pub struct Paths {
    program_path: Arc<String>,
    in_test_path: Arc<String>,
    out_test_path: Arc<String>,
    err_test_path: Arc<String>,
}

impl Paths {
    /*pub fn new(
        program_path: &str,
        in_test_path: &str,
        out_test_path: &str,
        err_test_path: &str,
    ) -> Self {
        Self {
            program_path: Arc::new(program_path.to_string()),
            in_test_path: Arc::new(in_test_path.to_string()),
            out_test_path: Arc::new(out_test_path.to_string()),
            err_test_path: Arc::new(err_test_path.to_string()),
        }
    }*/

    pub fn new(config: &Config) -> Self {
        match config.running_mode() {
            trst_types::RunningMode::Local {
                in_test_path,
                out_test_path,
                err_test_path,
                compiled_program_path,
            } => Self {
                program_path: Arc::new(compiled_program_path.clone()),
                in_test_path: Arc::new(in_test_path.clone()),
                out_test_path: Arc::new(out_test_path.clone()),
                err_test_path: Arc::new(err_test_path.clone()),
            },
            _ => todo!(),
        }
    }
}

impl TestExecutor {
    pub fn new() -> Result<Self, TestExecutorError> {
        if EXECUTOR.get().is_some() {
            return Err(TestExecutorError::AlreadyCreated);
        }

        Ok(Self { tests: Vec::new() })
    }

    pub fn push_test(&mut self, paths: Paths, test_name: String) {
        self.tests.push(TestingUnit::new(
            paths.program_path,
            paths.in_test_path,
            paths.out_test_path,
            paths.err_test_path,
            test_name,
        ))
    }

    pub fn execute_concurrent(self, max_concurrent_testing: u64) {
        let (tx, rx) = std::sync::mpsc::channel::<TestingUnitMessage>();

        let thread_job = move |unit: TestingUnit, tx: Sender<TestingUnitMessage>| {
            unit.run_test(tx);
        };

        let mut number_of_tests = self.tests.len();
        let mut tests = self.tests.into_iter();

        for _ in 0..max_concurrent_testing {
            if let Some(unit) = tests.next() {
                let thread_tx = tx.clone();

                std::thread::spawn(move || {
                    thread_job(unit, thread_tx);
                });
            } else {
                break;
            }
        }

        while number_of_tests > 0 {
            let msg = match rx.recv() {
                Err(e) => {
                    log::error!("Error while receiving value from thread. Error = {e:#?}");
                    break;
                }
                Ok(val) => val,
            };

            log::trace!("Message from thread = {msg:#?}");

            match msg {
                TestingUnitMessage::StartedExecution { .. } => {
                    let print_message: BackendMessage = msg.into();

                    let bytes = print_message.serialize();
                    std::io::stdout().write_all(&bytes).unwrap();
                }
                TestingUnitMessage::Done { .. } => {
                    number_of_tests -= 1;
                    let print_message: BackendMessage = msg.into();

                    let bytes = print_message.serialize();
                    std::io::stdout().write_all(&bytes).unwrap();

                    if let Some(unit) = tests.next() {
                        let thread_tx = tx.clone();

                        std::thread::spawn(move || {
                            thread_job(unit, thread_tx);
                        });
                    }
                }
            }
        }
    }
}

impl Executor for TestExecutor {
    fn execute_testing(self, config: &Config) {
        log::trace!("Executing testing for {self:#?}\n With config {config:#?}");

        match config.concurrency_settings() {
            Concurrency::Disabled => self.execute_concurrent(1),
            Concurrency::Enabled(threads) => self.execute_concurrent(*threads),
        }

        log::trace!("Testing process done, emitting end message");
        let end_message = BackendMessage::TestingProcessCompleted.serialize();
        std::io::stdout().write_all(&end_message).unwrap();
    }
}
