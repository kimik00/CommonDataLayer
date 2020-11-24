use crate::schema::DataPoint;
use serde::Serialize;

#[derive(Serialize)]
#[serde(remote = "DataPoint")]
pub struct DataPointDef {
    timestamp: String,
    value: f32,
}

#[derive(Serialize)]
pub struct DataPointSerializable(#[serde(with = "DataPointDef")] pub DataPoint);

#[cfg(test)]
mod tests {
    use crate::helper_types::DataPointSerializable;
    use crate::schema::DataPoint;
    #[test]
    fn it_works() {
        let given_input = vec![
            DataPointSerializable(DataPoint {
                timestamp: "2017-01-15T01:30:15.01Z".to_string(),
                value: 10.0,
            }),
            DataPointSerializable(DataPoint {
                timestamp: "2017-01-15T01:30:17.01Z".to_string(),
                value: 11.1,
            }),
        ];
        let output = serde_json::to_string(&given_input);
        // Fixme: Use more compact format of serializing
        let expected_output ="[{\"timestamp\":\"2017-01-15T01:30:15.01Z\",\"value\":10.0},{\"timestamp\":\"2017-01-15T01:30:17.01Z\",\"value\":11.1}]".to_string();
        match output {
            Ok(a) => assert_eq!(a, expected_output),
            Err(_) => assert!(false, "Serialization failed"),
        }
    }
}
