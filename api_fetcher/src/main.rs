use chrono::{DateTime, Local};
use reqwest::blocking::get;
use reqwest::Url;
use serde::{Deserialize, Serialize};

const STATIONBOARD_ENDPOINT: &str = "https://transport.opendata.ch/v1/stationboard";

fn main() {
    println!("Hello, world!");

    println!(
        "{:?}",
        StationBoardResponse::get(
            Some("Genève, Cornavin"),
            Some("8587057"),
            Some(3),
            Some(vec!["metro", "tram"]),
            Some(chrono::Local::now()),
            Some("arrival")
        )
        .expect("error with the API call")
    );
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Coordinate {
    r#type: String,
    x: Option<f32>,
    y: Option<f32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Location {
    id: String,
    name: Option<String>,
    score: Option<f32>,
    coordinate: Coordinate,
    distance: Option<f32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Prognosis {
    platform: Option<String>,
    departure: Option<DateTime<Local>>,
    arrival: Option<DateTime<Local>>,
    capacity1st: Option<u32>,
    capacity2nd: Option<u32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Stop {
    station: Location,
    arrival: Option<DateTime<Local>>,
    departure: Option<DateTime<Local>>,
    delay: Option<i32>,
    platform: Option<String>,
    prognosis: Option<Prognosis>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Journey {
    name: String,
    category: String,
    category_code: Option<String>,
    number: String,
    operator: String,
    to: String,
    capacity1st: Option<u32>,
    capacity2nd: Option<u32>,
    pass_list: Vec<Stop>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct StationBoardElement {
    stop: Stop,
    #[serde(flatten)]
    journey: Journey,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct StationBoardResponse {
    station: Location,
    stationboard: Vec<StationBoardElement>,
}

impl StationBoardResponse {
    fn get(
        station: Option<&str>,
        id: Option<&str>,
        limit: Option<u32>,
        transportations: Option<Vec<&str>>,
        datetime: Option<DateTime<Local>>,
        r#type: Option<&str>,
    ) -> Result<Self, String> {
        if station == None && id == None {
            return Err(String::from("must provide either a station or an id"));
        }
        let mut args: Vec<(&str, String)> = vec![];
        match station {
            Some(s) => args.push(("station", s.to_owned())),
            _ => (),
        }
        match id {
            Some(s) => args.push(("id", s.to_owned())),
            _ => (),
        }
        match r#type {
            Some(s) => args.push(("type", s.to_owned())),
            _ => (),
        }
        match limit {
            Some(l) => args.push(("limit", l.to_string())),
            _ => (),
        }
        match transportations {
            Some(t) => {
                for e in t {
                    args.push(("transportations", e.to_owned()))
                }
            }
            _ => (),
        }
        match datetime {
            Some(d) => args.push(("datetime", format!("{}", d.format("%Y-%m-%d %H:%M")))),
            _ => (),
        }

        let url = Url::parse_with_params(STATIONBOARD_ENDPOINT, args)
            .or(Err(String::from("url parameters should be parsable")))?;

        get(url)
            .or_else(|e| Err(format!("could not perform get request: {}", e)))?
            .json::<StationBoardResponse>()
            .or_else(|e| Err(format!("could not parse json received: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader, path::Path};

    use super::*;

    const TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

    fn coordinate() -> Coordinate {
        Coordinate {
            r#type: String::from("WGS84"),
            x: Some(46.209751),
            y: Some(6.14242),
        }
    }

    fn location() -> Location {
        Location {
            id: String::from("8587057"),
            name: Some(String::from("Gen\u{00e8}ve, gare Cornavin")),
            score: None,
            coordinate: coordinate(),
            distance: None,
        }
    }

    fn prognosis() -> Prognosis {
        Prognosis {
            platform: None,
            arrival: Some(
                DateTime::parse_from_str("2024-04-19T12:15:32+0200", TIME_FORMAT)
                    .unwrap()
                    .with_timezone(&Local),
            ),
            departure: Some(
                DateTime::parse_from_str("2024-04-19T12:12:00+0200", TIME_FORMAT)
                    .unwrap()
                    .with_timezone(&Local),
            ),
            capacity1st: None,
            capacity2nd: None,
        }
    }

    fn stop() -> Stop {
        Stop {
            station: location(),
            arrival: None,
            departure: Some(
                DateTime::parse_from_str("2024-04-19T12:09:00+0200", TIME_FORMAT)
                    .unwrap()
                    .with_timezone(&Local),
            ),
            delay: Some(3),
            platform: Some(String::from("F")),
            prognosis: Some(prognosis()),
        }
    }

    #[test]
    fn parse_coordinates() {
        let expected = coordinate();
        let test_data = r#"
         {
            "type": "WGS84",
            "x": 46.209751,
            "y": 6.14242
        }"#;

        let c: Coordinate = serde_json::from_str(test_data).unwrap();
        assert_eq!(c, expected)
    }

    #[test]
    fn parse_location() {
        let expected = location();
        let test_data = r#"{
            "id": "8587057",
            "name": "Gen\u00e8ve, gare Cornavin",
            "score": null,
            "coordinate": {
                "type": "WGS84",
                "x": 46.209751,
                "y": 6.14242
            },
            "distance": null
        }"#;

        let l: Location = serde_json::from_str(test_data).unwrap();
        assert_eq!(l, expected)
    }

    #[test]
    fn parse_prognosis() {
        let expected = prognosis();
        let test_data = r#"{
            "platform": null,
            "arrival": "2024-04-19T12:15:32+0200",
            "departure": "2024-04-19T12:12:00+0200",
            "capacity1st": null,
            "capacity2nd": null
        }"#;
        let p: Prognosis = serde_json::from_str(test_data).unwrap();
        assert_eq!(p, expected);
    }

    #[test]
    fn parse_stop() {
        let expected = stop();
        let test_data = r#"{
            "station": {
                "id": "8587057",
                "name": "Gen\u00e8ve, gare Cornavin",
                "score": null,
                "coordinate": {
                    "type": "WGS84",
                    "x": 46.209751,
                    "y": 6.14242
                },
                "distance": null
            },
            "arrival": null,
            "arrivalTimestamp": null,
            "departure": "2024-04-19T12:09:00+0200",
            "departureTimestamp": 1713521340,
            "delay": 3,
            "platform": "F",
            "prognosis": {
                "platform": null,
                "arrival": "2024-04-19T12:15:32+0200",
                "departure": "2024-04-19T12:12:00+0200",
                "capacity1st": null,
                "capacity2nd": null
            },
            "realtimeAvailability": null,
            "location": {
                "id": "8592899",
                "name": null,
                "score": null,
                "coordinate": {
                    "type": "WGS84",
                    "x": null,
                    "y": null
                },
                "distance": null
            }
        }"#;

        let s: Stop = serde_json::from_str(test_data).unwrap();
        assert_eq!(s, expected);
    }

    #[test]
    fn parse_journey() {
        let expected = Journey {
            name: String::from("315188"),
            category: String::from("B"),
            category_code: None,
            number: String::from("3"),
            operator: String::from("TPG"),
            to: String::from("Grand-Saconnex, Giacometti"),
            capacity1st: None,
            capacity2nd: None,
            pass_list: vec![Stop {
                station: Location {
                    id: String::from("8592899"),
                    name: None,
                    score: None,
                    coordinate: Coordinate {
                        r#type: String::from("WGS84"),
                        x: None,
                        y: None,
                    },
                    distance: None,
                },
                arrival: None,
                departure: Some(
                    DateTime::parse_from_str("2024-04-19T12:09:00+0200", TIME_FORMAT)
                        .unwrap()
                        .with_timezone(&Local),
                ),
                delay: Some(3),
                platform: Some(String::from("F")),
                prognosis: Some(prognosis()),
            }],
        };
        let test_data = r#"{
            "stop": {
                "station": {
                    "id": "8587057",
                    "name": "Gen\u00e8ve, gare Cornavin",
                    "score": null,
                    "coordinate": {
                        "type": "WGS84",
                        "x": 46.209751,
                        "y": 6.14242
                    },
                    "distance": null
                },
                "arrival": null,
                "arrivalTimestamp": null,
                "departure": "2024-04-19T12:09:00+0200",
                "departureTimestamp": 1713521340,
                "delay": 3,
                "platform": "F",
                "prognosis": {
                    "platform": null,
                    "arrival": "2024-04-19T12:15:32+0200",
                    "departure": "2024-04-19T12:12:00+0200",
                    "capacity1st": null,
                    "capacity2nd": null
                },
                "realtimeAvailability": null,
                "location": {
                    "id": "8592899",
                    "name": null,
                    "score": null,
                    "coordinate": {
                        "type": "WGS84",
                        "x": null,
                        "y": null
                    },
                    "distance": null
                }
            },
            "name": "315188",
            "category": "B",
            "subcategory": null,
            "categoryCode": null,
            "number": "3",
            "operator": "TPG",
            "to": "Grand-Saconnex, Giacometti",
            "passList": [
                {
                    "station": {
                        "id": "8592899",
                        "name": null,
                        "score": null,
                        "coordinate": {
                            "type": "WGS84",
                            "x": null,
                            "y": null
                        },
                        "distance": null
                    },
                    "arrival": null,
                    "arrivalTimestamp": null,
                    "departure": "2024-04-19T12:09:00+0200",
                    "departureTimestamp": 1713521340,
                    "delay": 3,
                    "platform": "F",
                    "prognosis": {
                        "platform": null,
                        "arrival": "2024-04-19T12:15:32+0200",
                        "departure": "2024-04-19T12:12:00+0200",
                        "capacity1st": null,
                        "capacity2nd": null
                    },
                    "realtimeAvailability": null,
                    "location": {
                        "id": "8592899",
                        "name": null,
                        "score": null,
                        "coordinate": {
                            "type": "WGS84",
                            "x": null,
                            "y": null
                        },
                        "distance": null
                    }
                }
            ],
            "capacity1st": null,
            "capacity2nd": null
        }"#;
        let j: Journey = serde_json::from_str(test_data).unwrap();

        assert_eq!(j, expected)
    }

    #[test]
    fn parse_stationboard() {
        let file =
            File::open(Path::new(env!("CARGO_MANIFEST_DIR")).join("geneve_cornavin_test.json"))
                .unwrap();
        let reader = BufReader::new(file);
        let s: StationBoardResponse = serde_json::from_reader(reader).unwrap();
        assert_eq!(s.station.coordinate, coordinate());
        assert_eq!(s.station, location());
        assert_eq!(
            s.stationboard[0].clone().stop.prognosis.unwrap(),
            prognosis()
        );
        assert_eq!(s.stationboard[0].clone().stop, stop());
    }

    #[test]
    fn test_api_call_all_params() {
        StationBoardResponse::get(
            Some("Genève, Cornavin"),
            Some("8587057"),
            Some(3),
            Some(vec!["metro", "tram"]),
            Some(chrono::Local::now()),
            Some("arrival"),
        )
        .expect("error with the API call");
    }
}
