// use std::collections::HashMap;
use indexmap::IndexMap;
use std::error::Error;
use std::fmt::Debug;
use std::io::Read;

use csv::Reader;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct RawNutritionRecord {
    date: String,
    meal: String,
    calories: String,
    fat: String,
    saturated_fat: String,
    polyunsaturated_fat: String,
    monounsaturated_fat: String,
    trans_fat: String,
    cholesterol: String,
    sodium: String,
    potassium: String,
    carbohydrates: String,
    fiber: String,
    sugar: String,
    protein: String,
    vitamin_a: String,
    vitamin_c: String,
    calcium: String,
    iron: String,
    note: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct ProcessedNutritionRecord {
    protein: f64,
    carbohydrates: f64,
    fat: f64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct WeightRecord {
    date: String,
    body_fat: Option<String>,
    weight: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StepsRecord {
    date: String,
    exercise: String,
    _type: String,
    exercise_calories: String,
    exercise_minutes: String,
    sets: String,
    rps: String,
    kilograms: String,
    steps: String,
    note: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct RawWorkoutRecord {
    date: String,
    workout_name: String,
    duration: String,
    exercise_name: String,
    set_order: String,
    weight: String,
    reps: String,
    distance: String,
    seconds: String,
    notes: String,
    workout_notes: String,
    rpe: String,
}

#[derive(Debug, serde::Serialize)]
struct ProcessedWorkoutRecord {
    workout_name: String,
    weight: String,
    reps: String,
}

fn deserialize_nutrition_csv<R>(
    mut rdr: csv::Reader<R>,
) -> Result<Vec<RawNutritionRecord>, Box<dyn Error>>
where
    R: std::io::Read,
{
    let new_headers = rdr
        .headers()
        .iter()
        .next()
        .unwrap()
        .iter()
        .map(|h| match h {
            "Fat (g)" => "fat",
            "Sodium (mg)" => "sodium",
            "Carbohydrates (g)" => "carbohydrates",
            "Protein (g)" => "protein",
            "Saturated Fat" => "saturated_fat",
            "Polyunsaturated Fat" => "polyunsaturated_fat",
            "Monounsaturated Fat" => "monounsaturated_fat",
            "Trans Fat" => "trans_fat",
            "Vitamin A" => "vitamin_a",
            "Vitamin C" => "vitamin_c",
            _ => h,
        })
        .map(|h| h.to_lowercase())
        .collect::<csv::StringRecord>();

    rdr.set_headers(new_headers);

    // Partition suggestion taken from https://doc.rust-lang.org/rust-by-example/error/iter_result.html
    let (records, _): (Vec<_>, Vec<_>) = rdr.deserialize().partition(Result::is_ok);
    let records: Vec<RawNutritionRecord> = records.into_iter().map(Result::unwrap).collect();

    Ok(records)
}

fn deserialize_weight_csv<R>(mut rdr: csv::Reader<R>) -> Vec<String>
where
    R: std::io::Read,
{
    let new_headers = rdr
        .headers()
        .iter()
        .next()
        .unwrap()
        .iter()
        .map(|h| match h {
            "Body Fat %" => "body_fat",
            _ => h,
        })
        .map(|h| h.to_lowercase())
        .collect::<csv::StringRecord>();

    rdr.set_headers(new_headers);

    // Partition suggestion taken from https://doc.rust-lang.org/rust-by-example/error/iter_result.html
    let (records, _): (Vec<_>, Vec<_>) = rdr.deserialize().partition(Result::is_ok);
    let records: Vec<WeightRecord> = records.into_iter().map(Result::unwrap).collect();

    let mut only_weight = vec![];
    for record in records {
        only_weight.push(record.weight);
    }

    only_weight
}

fn build_nutrition_csv_for_clipboard<'a>(
    records: Vec<RawNutritionRecord>,
    buf: Box<dyn std::io::Write + 'a>,
) {
    let mut wtr = csv::Writer::from_writer(buf);
    let mut records_grouped_by_date: IndexMap<String, ProcessedNutritionRecord> = IndexMap::new();

    for record in records {
        records_grouped_by_date
            .entry(record.date)
            .and_modify(|pr| {
                pr.protein += record.protein.parse::<f64>().unwrap();
                pr.carbohydrates += record.carbohydrates.parse::<f64>().unwrap();
                pr.fat += record.fat.parse::<f64>().unwrap();
            })
            .or_insert(ProcessedNutritionRecord {
                protein: record.protein.parse::<f64>().unwrap(),
                carbohydrates: record.carbohydrates.parse::<f64>().unwrap(),
                fat: record.fat.parse::<f64>().unwrap(),
            });
    }
    let processed_records = records_grouped_by_date.values().collect::<Vec<_>>();

    for record in processed_records {
        wtr.serialize(record).ok();
    }
    wtr.into_inner().ok();
}

fn generic_build_csv_for_clipboard<'a, R>(records: Vec<R>, buf: Box<dyn std::io::Write + 'a>)
where
    R: serde::Serialize,
{
    let mut wtr = csv::Writer::from_writer(buf);

    for record in records {
        wtr.serialize(record).ok();
    }
    wtr.into_inner().ok();
}

fn deserialize_steps_csv<B>(mut rdr: Reader<B>) -> Vec<String>
where
    B: std::io::Read,
{
    let new_headers = rdr
        .headers()
        .iter()
        .next()
        .unwrap()
        .iter()
        .map(|h| match h {
            "Type" => "_type",
            "Exercise Calories" => "exercise_calories",
            "Exercise Minutes" => "exercise_minutes",
            "Reps Per Set" => "rps",
            _ => h,
        })
        .map(|h| h.to_lowercase())
        .collect::<csv::StringRecord>();

    rdr.set_headers(new_headers);

    // Partition suggestion taken from https://doc.rust-lang.org/rust-by-example/error/iter_result.html
    let (records, _): (Vec<_>, Vec<_>) = rdr.deserialize().partition(Result::is_ok);
    let records: Vec<StepsRecord> = records.into_iter().map(Result::unwrap).collect();

    let mut only_steps = vec![];
    for record in records {
        if record.steps != "" {
            only_steps.push(record.steps);
        }
    }

    only_steps
}

pub fn process<B>(file_bytes: B, file_type: &str) -> String
where
    B: std::io::Read,
{
    let csv_reader = csv::Reader::from_reader(file_bytes);
    let mut buf = vec![];

    match file_type {
        "Macros" => {
            let records = deserialize_nutrition_csv(csv_reader).unwrap();
            build_nutrition_csv_for_clipboard(records, Box::new(&mut buf));
        }
        "Weight" => {
            let records = deserialize_weight_csv(csv_reader);
            generic_build_csv_for_clipboard(records, Box::new(&mut buf));
        }
        "Steps" => {
            let records = deserialize_steps_csv(csv_reader);
            generic_build_csv_for_clipboard(records, Box::new(&mut buf));
        }
        _ => (),
    };

    //from https://stackoverflow.com/questions/63024483/writing-to-a-file-or-string-in-rust
    let mut bytes = &buf[..];
    let mut out = String::new();
    bytes.read_to_string(&mut out).unwrap();

    // FIXME this hack fixes a bug somewhere in the code...
    if out.contains("protein") {
        out = out.replace("protein,carbohydrates,fat\n", "")
    }

    out = out.replace("\n", "\r\n");
    out
}
