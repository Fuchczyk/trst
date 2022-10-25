use std::{
    io::Write,
    process::{Child, Command, Stdio},
    sync::{mpsc::Sender, Arc},
    time::SystemTime,
};
use trst_types::*;

use super::CHECK_STATUS_INTERVAL;

#[derive(Debug)]
pub enum TestingUnitMessage {
    StartedExecution { name: String },
    Done { result: TestResult },
}

impl From<TestingUnitMessage> for BackendMessage {
    fn from(msg: TestingUnitMessage) -> Self {
        match msg {
            TestingUnitMessage::Done { result } => BackendMessage::TestCompleted { result },
            TestingUnitMessage::StartedExecution { name } => {
                BackendMessage::ExecutionStarted { test_name: name }
            }
        }
    }
}

#[derive(Debug)]
pub struct TestingUnit {
    program_path: Arc<String>,
    in_test_path: Arc<String>,
    out_test_path: Arc<String>,
    err_test_path: Arc<String>,

    name: String,
}

impl TestingUnit {
    pub fn new(
        program_path: Arc<String>,
        in_test_path: Arc<String>,
        out_test_path: Arc<String>,
        err_test_path: Arc<String>,

        name: String,
    ) -> Self {
        Self {
            program_path,
            in_test_path,
            out_test_path,
            err_test_path,
            name,
        }
    }
    fn report_internal_error<E: ToString>(test_name: String, error: E) -> TestResult {
        let measure = TestMeasure::InternalProgramError {
            description: error.to_string(),
        };

        TestResult::new(test_name, measure)
    }

    fn report_timing_out(test_name: String) -> TestResult {
        let measure = TestMeasure::Timeout;

        TestResult::new(test_name, measure)
    }

    fn in_file_path(&self) -> String {
        if self.in_test_path.ends_with('/') {
            format!("{}{}.in", self.in_test_path, self.name)
        } else {
            format!("{}/{}.in", self.in_test_path, self.name)
        }
    }

    fn out_file_path(&self) -> String {
        if self.out_test_path.ends_with('/') {
            format!("{}{}.out", self.out_test_path, self.name)
        } else {
            format!("{}/{}.out", self.out_test_path, self.name)
        }
    }

    fn err_file_path(&self) -> String {
        if self.err_test_path.ends_with('/') {
            format!("{}{}.err", self.err_test_path, self.name)
        } else {
            format!("{}/{}.err", self.err_test_path, self.name)
        }
    }

    fn construct_child(
        &self,
        channel_status_report: &Sender<TestingUnitMessage>,
    ) -> Result<(Child, SystemTime), TestingUnitMessage> {
        let mut command = Command::new(self.program_path.as_ref());

        // Program's streams settings
        command
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped());

        let in_path = self.in_file_path();
        let in_file = std::fs::read(&in_path).map_err(|e| TestingUnitMessage::Done {
            result: Self::report_internal_error(self.name.clone(), e),
        })?;

        let mut spawned_command = command.spawn().map_err(|e| TestingUnitMessage::Done {
            result: Self::report_internal_error(self.name.clone(), e),
        })?;

        channel_status_report
            .send(TestingUnitMessage::StartedExecution {
                name: self.name.clone(),
            })
            .unwrap();

        spawned_command
            .stdin
            .as_mut()
            .unwrap()
            .write_all(in_file.as_ref())
            .map_err(|e| TestingUnitMessage::Done {
                result: Self::report_internal_error(self.name.clone(), e),
            })?;

        let started_time = SystemTime::now();

        Ok((spawned_command, started_time))
    }

    pub fn run_test(self, channel_status_report: Sender<TestingUnitMessage>) {
        let (mut spawned_command, started_time) = match self.construct_child(&channel_status_report)
        {
            Ok(result) => result,
            Err(e) => {
                channel_status_report.send(e).unwrap();
                return;
            }
        };

        let exited_time;
        loop {
            log::trace!("LOOP ENTRY. [{:?}]", spawned_command.try_wait());
            match spawned_command.try_wait() {
                Ok(None) => {
                    log::trace!("Checking time = [{:?}]", crate::TEST_TIMEOUT);
                    if SystemTime::now().duration_since(started_time).unwrap()
                        > *crate::TEST_TIMEOUT.get().unwrap()
                    {
                        let _ = spawned_command.kill();

                        channel_status_report
                            .send(TestingUnitMessage::Done {
                                result: Self::report_timing_out(self.name),
                            })
                            .unwrap();
                        return;
                    } else {
                        // TODO: Check if this is working correctly
                        // TODO: Make this interruptible by process ending (another thread can be blocked waiting on this and some channels)
                        std::thread::sleep(CHECK_STATUS_INTERVAL);
                    }
                }

                Err(_) => {
                    todo!();
                }

                Ok(_) => {
                    exited_time = SystemTime::now();
                    break;
                }
            }
        }

        let command_output = spawned_command.wait_with_output().unwrap();

        channel_status_report
            .send(
                self.check_outcome(
                    command_output,
                    exited_time
                        .duration_since(started_time)
                        .unwrap()
                        .as_secs_f64(),
                ),
            )
            .unwrap();
    }

    fn failed_test(
        self,
        stdout: String,
        stderr: String,
        exit_status: Option<i32>,
    ) -> TestingUnitMessage {
        TestingUnitMessage::Done {
            result: TestResult::new(
                self.name,
                TestMeasure::Failure {
                    stdout,
                    stderr,
                    exit_status,
                },
            ),
        }
    }

    fn check_outcome(self, output: std::process::Output, elapsed_time: f64) -> TestingUnitMessage {
        let out_file = match std::fs::read_to_string(self.out_file_path()) {
            Ok(content) => content,
            Err(e) => {
                return TestingUnitMessage::Done {
                    result: Self::report_internal_error(self.name, e),
                };
            }
        };

        let program_stdout = String::from_utf8(output.stdout).unwrap();
        let program_stderr = String::from_utf8(output.stderr).unwrap();

        if program_stdout != out_file {
            // TODO: Segmentation Fault detection
            return self.failed_test(program_stdout, program_stderr, output.status.code());
        }

        drop(out_file);

        let err_file = match std::fs::read_to_string(self.err_file_path()) {
            Ok(content) => content,
            Err(e) => {
                return TestingUnitMessage::Done {
                    result: Self::report_internal_error(self.name, e),
                };
            }
        };

        if program_stderr != err_file {
            // TODO: Segmentation Fault detection
            return self.failed_test(program_stdout, program_stderr, output.status.code());
        }

        TestingUnitMessage::Done {
            result: TestResult::new(
                self.name,
                TestMeasure::Success {
                    time: elapsed_time,
                    exit_status: output.status.code(),
                },
            ),
        }
    }
}
