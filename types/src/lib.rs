use std::{collections::VecDeque, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TestMeasure {
    Success {
        time: f64,
        exit_status: Option<i32>,
    },
    Failure {
        stdout: String,
        stderr: String,
        exit_status: Option<i32>,
    },
    InternalProgramError {
        description: String,
    },
    Timeout,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestResult {
    name: String,
    outcome: TestMeasure,
}

const SIZE_LEN: u32 = u32::BITS / 8;

impl TestResult {
    pub fn new(name: String, outcome: TestMeasure) -> Self {
        Self { name, outcome }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn outcome(&self) -> &TestMeasure {
        &self.outcome
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BackendMessage {
    ExecutionStarted { test_name: String },
    TestCompleted { result: TestResult },
    TestingProcessCompleted,
}

// TRSTER CONFIG STANDARD
#[derive(Serialize, Deserialize, Debug)]
pub enum Language {
    Cpp,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum RunningMode {
    GitRepository {
        address: String,
    },
    Local {
        in_test_path: String,
        out_test_path: String,
        err_test_path: String,
        compiled_program_path: String,
    },
}

impl Display for RunningMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::GitRepository { .. } => "Git repository mode",
            Self::Local {..} => "Local mode"
        };

        write!(f, "{name}")
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Concurrency {
    Disabled,
    Enabled(u64),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    mode: RunningMode,
    test_list: Vec<String>,
    language: Language,
    concurrency: Concurrency,
}

impl Config {
    pub fn concurrency_settings(&self) -> &Concurrency {
        &self.concurrency
    }

    pub fn test_names(&self) -> impl Iterator<Item = &String> {
        self.test_list.iter()
    }

    pub fn running_mode(&self) -> &RunningMode {
        &self.mode
    }
}

#[test]
fn tak() {
    let vals: [u8; 64] = [
        0, 5, 116, 101, 115, 116, 49, 1, 5, 116, 101, 115, 116, 49, 1, 2, 53, 10, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let mut vd = VecDeque::new();
    for i in vals {
        vd.push_back(i);
    }

    println!("FIRST {:?}", BackendMessage::try_deserialize(&mut vd));

    let bm = BackendMessage::ExecutionStarted {
        test_name: "test1".into(),
    };
    let x = bm.serialize();
    println!("X = {x:?}");
    let mut vd = VecDeque::new();

    for i in x.iter() {
        vd.push_back(*i);
    }

    for i in x.iter() {
        vd.push_back(*i);
    }

    let bm = BackendMessage::ExecutionStarted {
        test_name: "tak".into(),
    };
    let x = bm.serialize();

    for i in x.iter() {
        vd.push_back(*i);
    }

    vd.push_back(0);

    println!("{:?}", x);
    println!("{:?}", BackendMessage::try_deserialize(&mut vd));
    println!("{:?}", x);
}

impl BackendMessage {
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = postcard::to_allocvec(&self).unwrap();
        let size = serialized.len() as u32;

        let mut result = Vec::new();
        size.to_be_bytes()
            .into_iter()
            .for_each(|num| result.push(num));
        result.append(&mut serialized);

        result
    }

    pub fn try_deserialize(bytes: &mut VecDeque<u8>) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }

        let size = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        if size + 4 > bytes.len() as u32 {
            return None;
        }

        for _ in 0..SIZE_LEN {
            bytes.pop_front();
        }

        let mut self_data = Vec::new();

        for _ in 0..size {
            self_data.push(bytes.pop_front().unwrap());
        }

        Some(postcard::from_bytes(&self_data).unwrap())
    }
}
