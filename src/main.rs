// use std::collections::HashMap;
use indexmap::IndexMap;
use std::error::Error;
use std::fs::File;
use std::io;
use std::path::Path;

use clap::Parser;
use csv::Reader;

/// Processes raw MFP app export CSV to chris' spreadsheet format
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path of the CSV file to process
    #[arg(short, long)]
    path: String,

    /// type of data to process
    #[arg(short, long)]
    data_type: String,
}

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
    body_fat: String,
    weight: String,
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

#[derive(Debug, Clone)]
struct ExerciseRow {
    sets: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
struct ExercisesMap {
    map: IndexMap<String, ExerciseRow>,
}

impl ExercisesMap {
    fn new(exercise_name: String, exercises: ExerciseRow) -> Self {
        let mut map = IndexMap::new();
        map.insert(exercise_name, exercises);
        Self { map }
    }
}

#[derive(Debug, Clone)]
struct ExerciseRowsForADate {
    exercises: ExercisesMap,
}

impl ExerciseRowsForADate {
    fn new(exercises: ExercisesMap) -> Self {
        Self { exercises }
    }
}

#[derive(Debug)]
struct LastFourDaysWorkouts {
    data: IndexMap<String, ExerciseRowsForADate>,
}

impl From<Vec<RawWorkoutRecord>> for LastFourDaysWorkouts {
    fn from(raw_workout_records: Vec<RawWorkoutRecord>) -> Self {
        // {
        //  date: {
        //      exercise_1: [(weight, reps),(weight, reps)],
        //      exercise_2: [(weight, reps),(weight, reps)],
        //  }
        // }
        let mut records_grouped_by_date: IndexMap<String, ExerciseRowsForADate> = IndexMap::new();

        for mut raw_workout_record in raw_workout_records {
            raw_workout_record.date = raw_workout_record
                .date
                .split_whitespace()
                .next()
                .unwrap()
                .to_string();

            let set_tuple_copy = (
                raw_workout_record.weight.clone(),
                raw_workout_record.reps.clone(),
            );
            let set_tuple_copy_2 = (
                raw_workout_record.weight.clone(),
                raw_workout_record.reps.clone(),
            );
            let date_copy = raw_workout_record.date.clone();
            let name_copy: String = raw_workout_record.exercise_name.clone();

            records_grouped_by_date
                .entry(raw_workout_record.date)
                // date exists; find exercise
                .and_modify(|erd| {
                    erd.exercises
                        .map
                        .entry(raw_workout_record.exercise_name)
                        // exercise exists: append to row
                        .and_modify(|er| {
                            er.sets
                                .push((raw_workout_record.weight, raw_workout_record.reps));
                        })
                        // new exercise: create row with one tuple
                        .or_insert(ExerciseRow {
                            sets: vec![set_tuple_copy],
                        });
                })
                // date doesn't exist; create it with a new exercise map
                .or_insert(ExerciseRowsForADate::new(ExercisesMap::new(
                    name_copy,
                    ExerciseRow {
                        sets: vec![set_tuple_copy_2],
                    },
                )));
        }

        // Only want to get the most recent four days to paste into the spreadsheet
        records_grouped_by_date.reverse();
        records_grouped_by_date.truncate(4);

        LastFourDaysWorkouts {
            data: records_grouped_by_date,
        }
    }
}

fn read_csv<P: AsRef<Path>>(filename: P) -> Result<Reader<File>, Box<dyn Error>> {
    let file = File::open(filename)?;
    Ok(csv::Reader::from_reader(file))
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

fn deserialize_weight_csv<R>(mut rdr: csv::Reader<R>) -> Result<Vec<WeightRecord>, Box<dyn Error>>
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

    Ok(records)
}

fn build_nutrition_csv_for_clipboard(records: Vec<RawNutritionRecord>) {
    let mut wtr = csv::Writer::from_writer(io::stdout());
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

fn build_weight_csv_for_clipboard(records: Vec<WeightRecord>) {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    for record in records {
        wtr.serialize(record).ok();
    }
    wtr.into_inner().ok();
}

fn deserialize_workout_csv(mut rdr: Reader<File>) -> Result<Vec<RawWorkoutRecord>, Box<dyn Error>> {
    let new_headers = rdr
        .headers()
        .iter()
        .next()
        .unwrap()
        .iter()
        .map(|h| match h {
            "Workout Name" => "workout_name",
            "Exercise Name" => "exercise_name",
            "Set Order" => "set_order",
            "Workout Notes" => "workout_notes",
            _ => h,
        })
        .map(|h| h.to_lowercase())
        .collect::<csv::StringRecord>();

    rdr.set_headers(new_headers);

    // Partition suggestion taken from https://doc.rust-lang.org/rust-by-example/error/iter_result.html
    let (records, _): (Vec<_>, Vec<_>) = rdr.deserialize().partition(Result::is_ok);
    let records: Vec<RawWorkoutRecord> = records.into_iter().map(Result::unwrap).collect();

    Ok(records)
}

fn flatten(sets: &[(String, String)]) -> Vec<String> {
    sets.iter().fold(vec![], |mut array, tup| {
        array.push(tup.0.clone());
        array.push(tup.1.clone());
        array
    })
}

fn build_workout_csv_for_clipboard(records: Vec<RawWorkoutRecord>) {
    let last_four_days_records = LastFourDaysWorkouts::from(records);

    for data in last_four_days_records.data {
        for exercise_row in data.1.exercises.map {
            let row = flatten(&exercise_row.1.sets);

            // Just print it out because csv writer throws a wobbly when the rows are different
            // lengths
            println!("{}", row.join(","));
        }
        println!("")
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let filename = args.path;
    match args.data_type.as_ref() {
        "nutrition" => {
            let csv_reader = read_csv(filename)?;
            let records = deserialize_nutrition_csv(csv_reader).unwrap();
            build_nutrition_csv_for_clipboard(records);
        }
        "weight" => {
            let csv_reader = read_csv(filename)?;
            let records = deserialize_weight_csv(csv_reader).unwrap();
            build_weight_csv_for_clipboard(records);
        }
        "workouts" => {
            let csv_reader = read_csv(filename)?;
            let records = deserialize_workout_csv(csv_reader).unwrap();
            build_workout_csv_for_clipboard(records);
        }
        _ => (),
    }
    Ok(())
}
