use crate::response::ValidationResponse;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::BufReader;

fn read_request_file(path: &str) -> anyhow::Result<serde_json::Value> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let v = serde_json::from_reader(reader)?;

    Ok(v)
}

fn make_validate_payload<T>(request_file: &str, settings: &T) -> String
where
    T: DeserializeOwned + Serialize + crate::settings::Trusties,
{
    let req = read_request_file(request_file).unwrap();
    let payload = json!({
        "settings": settings,
        "request": req
    });

    payload.to_string()
}

#[allow(dead_code)]
type ValidateFn = fn(&[u8]) -> wapc_guest::CallResult;

pub struct Testcase<T>
where
    T: DeserializeOwned + crate::settings::Trusties,
{
    pub name: String,
    pub fixture_file: String,
    pub expected_validation_result: bool,
    pub settings: T,
}

#[allow(dead_code)]
impl<T> Testcase<T>
where
    T: DeserializeOwned + Serialize + crate::settings::Trusties,
{
    pub fn eval(&self, validate: ValidateFn) -> anyhow::Result<()> {
        let payload = make_validate_payload(self.fixture_file.as_str(), &self.settings);
        let raw_result = validate(payload.as_bytes()).unwrap();
        let result: ValidationResponse = serde_json::from_slice(&raw_result)?;
        assert_eq!(
            result.accepted, self.expected_validation_result,
            "Failure for test case: '{}': got {:?} instead of {:?}",
            self.name, result.accepted, self.expected_validation_result,
        );

        Ok(())
    }
}
