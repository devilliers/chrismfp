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
struct RawWeightRecord {
    date: String,
    body_fat: String,
    weight: String,
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

fn deserialize_weight_csv<R>(
    mut rdr: csv::Reader<R>,
) -> Result<Vec<RawWeightRecord>, Box<dyn Error>>
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
    let records: Vec<RawWeightRecord> = records.into_iter().map(Result::unwrap).collect();

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

fn build_weight_csv_for_clipboard(records: Vec<RawWeightRecord>) {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    for record in records {
        wtr.serialize(record).ok();
    }
    wtr.into_inner().ok();
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
        _ => (),
    }
    Ok(())
}
