use chrono::{DateTime, Local};
use dotenv::dotenv;
use reqwest::blocking::get;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::{cmp::min, fmt};

const STATIONBOARD_ENDPOINT: &str = "https://transport.opendata.ch/v1/stationboard";
const JOURNEYS_LIMIT: u32 = 5;
const MAX_DISPLAYED_LINES: usize = 5;
const RESULT_FILE_NAME: &str = "api_result.tsv";

fn main() {
    dotenv().ok();
    let station_id = std::env::var("STATION_ID").expect("STATION_ID must be set in .env file.");
    let station_board = StationBoardResponse::get(
        None,
        Some(&station_id),
        Some(JOURNEYS_LIMIT),
        None,
        None,
        None,
    )
    .expect("error with the API call");
    // println!("{:#?}", station_board);

    let nb_stations = min(station_board.stationboard.len(), MAX_DISPLAYED_LINES);
    let mut lines_info: Vec<LineInfo> = Vec::with_capacity(nb_stations);
    for i in 0..nb_stations {
        let s = &station_board.stationboard[i].stop;
        let j = &station_board.stationboard[i].journey;
        lines_info.push(LineInfo {
            line_number: j
                .number
                .to_owned()
                .expect("expect line number to be available"),
            direction: j.to.to_owned(),
            normal_departure: format!(
                "{}",
                s.departure
                    .expect("departure time should be present")
                    .format("%H:%M")
            ),
            delay: s.delay.unwrap_or_default(),
        })
    }

    let mut to_write = String::from("");
    for l in lines_info {
        println!("{}", l);
        to_write += &l.to_string();
        to_write.push('\n');
    }

    let path = Path::new(RESULT_FILE_NAME);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    // Write the `to_write` string to `file`, returns `io::Result<()>`
    match file.write_all(to_write.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Coordinate {
    r#type: String,
    x: Option<f32>,
    y: Option<f32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Location {
    id: Option<String>,
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
    name: Option<String>,
    category: String,
    category_code: Option<String>,
    number: Option<String>,
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
        let mut args: Vec<(&str, String)> = Vec::with_capacity(6);
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

#[derive(Debug)]
struct LineInfo {
    line_number: String,
    direction: String,
    normal_departure: String,
    delay: i32,
}

impl fmt::Display for LineInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}\t{}",
            self.line_number, self.direction, self.normal_departure, self.delay
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader, path::Path};

    use super::*;

    const TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

    fn coordinate_geneva() -> Coordinate {
        Coordinate {
            r#type: String::from("WGS84"),
            x: Some(46.209751),
            y: Some(6.14242),
        }
    }

    fn coordinate_zurich() -> Coordinate {
        Coordinate {
            r#type: String::from("WGS84"),
            x: Some(47.377847),
            y: Some(8.540502),
        }
    }

    fn location_geneva() -> Location {
        Location {
            id: Some(String::from("8587057")),
            name: Some(String::from("Gen\u{00e8}ve, gare Cornavin")),
            score: None,
            coordinate: coordinate_geneva(),
            distance: None,
        }
    }

    fn location_zurich() -> Location {
        Location {
            id: Some(String::from("8503000")),
            name: Some(String::from("Z\u{00fc}rich HB")),
            score: None,
            coordinate: coordinate_zurich(),
            distance: None,
        }
    }

    fn prognosis_geneva() -> Prognosis {
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

    fn prognosis_zurich() -> Prognosis {
        Prognosis {
            platform: None,
            arrival: None,
            departure: Some(
                DateTime::parse_from_str("2024-04-23T11:38:00+0200", TIME_FORMAT)
                    .unwrap()
                    .with_timezone(&Local),
            ),
            capacity1st: None,
            capacity2nd: None,
        }
    }

    fn stop_geneva() -> Stop {
        Stop {
            station: location_geneva(),
            arrival: None,
            departure: Some(
                DateTime::parse_from_str("2024-04-19T12:09:00+0200", TIME_FORMAT)
                    .unwrap()
                    .with_timezone(&Local),
            ),
            delay: Some(3),
            platform: Some(String::from("F")),
            prognosis: Some(prognosis_geneva()),
        }
    }

    fn stop_zurich() -> Stop {
        Stop {
            station: location_zurich(),
            arrival: None,
            departure: Some(
                DateTime::parse_from_str("2024-04-23T11:38:00+0200", TIME_FORMAT)
                    .unwrap()
                    .with_timezone(&Local),
            ),
            delay: Some(0),
            platform: Some(String::from("8")),
            prognosis: Some(prognosis_zurich()),
        }
    }

    #[test]
    fn parse_coordinates() {
        let expected = coordinate_geneva();
        let test_data = r#"
         {
            "type": "WGS84",
            "x": 46.209751,
            "y": 6.14242
        }"#;

        let c: Coordinate = serde_json::from_str(test_data).unwrap();
        assert_eq!(c, expected);

        let expected = coordinate_zurich();
        let test_data = r#"
        {
            "type": "WGS84",
            "x": 47.377847,
            "y": 8.540502
        }"#;

        let c: Coordinate = serde_json::from_str(test_data).unwrap();
        assert_eq!(c, expected)
    }

    #[test]
    fn parse_location() {
        let expected = location_geneva();
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
        assert_eq!(l, expected);

        let expected = location_zurich();
        let test_data = r#"{
            "id": "8503000",
            "name": "Z\u00fcrich HB",
            "score": null,
            "coordinate": {
                "type": "WGS84",
                "x": 47.377847,
                "y": 8.540502
            },
            "distance": null
        }"#;

        let l: Location = serde_json::from_str(test_data).unwrap();
        assert_eq!(l, expected)
    }

    #[test]
    fn parse_prognosis() {
        let expected = prognosis_geneva();
        let test_data = r#"{
            "platform": null,
            "arrival": "2024-04-19T12:15:32+0200",
            "departure": "2024-04-19T12:12:00+0200",
            "capacity1st": null,
            "capacity2nd": null
        }"#;
        let p: Prognosis = serde_json::from_str(test_data).unwrap();
        assert_eq!(p, expected);

        let expected = prognosis_zurich();
        let test_data = r#"{
            "platform": null,
            "arrival": null,
            "departure": "2024-04-23T11:38:00+0200",
            "capacity1st": null,
            "capacity2nd": null
        }"#;
        let p: Prognosis = serde_json::from_str(test_data).unwrap();
        assert_eq!(p, expected);
    }

    #[test]
    fn parse_stop() {
        let expected = stop_geneva();
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

        let expected = stop_zurich();
        let test_data = r#"{
            "station": {
                "id": "8503000",
                "name": "Z\u00fcrich HB",
                "score": null,
                "coordinate": {
                    "type": "WGS84",
                    "x": 47.377847,
                    "y": 8.540502
                },
                "distance": null
            },
            "arrival": null,
            "arrivalTimestamp": null,
            "departure": "2024-04-23T11:38:00+0200",
            "departureTimestamp": 1713865080,
            "delay": 0,
            "platform": "8",
            "prognosis": {
                "platform": null,
                "arrival": null,
                "departure": "2024-04-23T11:38:00+0200",
                "capacity1st": null,
                "capacity2nd": null
            },
            "realtimeAvailability": null,
            "location": {
                "id": "8509000",
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
            name: Some(String::from("315188")),
            category: String::from("B"),
            category_code: None,
            number: Some(String::from("3")),
            operator: String::from("TPG"),
            to: String::from("Grand-Saconnex, Giacometti"),
            capacity1st: None,
            capacity2nd: None,
            pass_list: vec![Stop {
                station: Location {
                    id: Some(String::from("8592899")),
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
                prognosis: Some(prognosis_geneva()),
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

        assert_eq!(j, expected);

        let expected = Journey {
            name: Some(String::from("000567")),
            category: String::from("IC"),
            category_code: None,
            number: Some(String::from("3")),
            operator: String::from("SBB"),
            to: String::from("Chur"),
            capacity1st: None,
            capacity2nd: None,
            pass_list: vec![Stop {
                station: Location {
                    id: Some(String::from("8509000")),
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
                    DateTime::parse_from_str("2024-04-23T11:38:00+0200", TIME_FORMAT)
                        .unwrap()
                        .with_timezone(&Local),
                ),
                delay: Some(0),
                platform: Some(String::from("8")),
                prognosis: Some(prognosis_zurich()),
            }],
        };
        let test_data = r#"{
            "stop": {
                "station": {
                    "id": "8503000",
                    "name": "Z\u00fcrich HB",
                    "score": null,
                    "coordinate": {
                        "type": "WGS84",
                        "x": 47.377847,
                        "y": 8.540502
                    },
                    "distance": null
                },
                "arrival": null,
                "arrivalTimestamp": null,
                "departure": "2024-04-23T11:38:00+0200",
                "departureTimestamp": 1713865080,
                "delay": 0,
                "platform": "8",
                "prognosis": {
                    "platform": null,
                    "arrival": null,
                    "departure": "2024-04-23T11:38:00+0200",
                    "capacity1st": null,
                    "capacity2nd": null
                },
                "realtimeAvailability": null,
                "location": {
                    "id": "8509000",
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
            "name": "000567",
            "category": "IC",
            "subcategory": null,
            "categoryCode": null,
            "number": "3",
            "operator": "SBB",
            "to": "Chur",
            "passList": [
                {
                    "station": {
                        "id": "8509000",
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
                    "departure": "2024-04-23T11:38:00+0200",
                    "departureTimestamp": 1713865080,
                    "delay": 0,
                    "platform": "8",
                    "prognosis": {
                        "platform": null,
                        "arrival": null,
                        "departure": "2024-04-23T11:38:00+0200",
                        "capacity1st": null,
                        "capacity2nd": null
                    },
                    "realtimeAvailability": null,
                    "location": {
                        "id": "8509000",
                        "name": null,
                        "score": null,
                        "coordinate": {
                            "type": "WGS84",
                            "x": null,
                            "y": null
                        },
                        "distance": null
                    }
                }],
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
        assert_eq!(s.station.coordinate, coordinate_geneva());
        assert_eq!(s.station, location_geneva());
        assert_eq!(
            s.stationboard[0].clone().stop.prognosis.unwrap(),
            prognosis_geneva()
        );
        assert_eq!(s.stationboard[0].clone().stop, stop_geneva());

        let file =
            File::open(Path::new(env!("CARGO_MANIFEST_DIR")).join("zurich_hb_test.json")).unwrap();
        let reader = BufReader::new(file);
        let s: StationBoardResponse = serde_json::from_reader(reader).unwrap();
        assert_eq!(s.station.coordinate, coordinate_zurich());
        assert_eq!(s.station, location_zurich());
        assert_eq!(
            s.stationboard[0].clone().stop.prognosis.unwrap(),
            prognosis_zurich()
        );
        assert_eq!(s.stationboard[0].clone().stop, stop_zurich());
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

    #[test]
    fn test_api_different_stations() {
        for s in [
            "Genève, gare Cornavin",
            "Zürich HB",
            "Lausanne, gare",
            "Bern, Bahnhof",
        ] {
            StationBoardResponse::get(Some(s), None, None, None, None, None)
                .expect(&format!("error with the API call for station {}", s));
        }
    }
}
