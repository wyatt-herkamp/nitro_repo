use std::fs::read_dir;
use std::path::Path;

use chrono::NaiveDateTime;

use crate::error::internal_error::InternalError;

#[allow(dead_code)]
fn get_artifacts(path: &Path) -> Vec<String> {
    let dir = read_dir(path).unwrap();
    let mut values = Vec::new();
    for x in dir {
        let x1 = x.unwrap();
        if x1.file_type().unwrap().is_file() {
            let file_name = x1.file_name().to_str().unwrap().to_string();
            if file_name.ends_with(".sha1") || file_name.ends_with(".md5") {
                continue;
            }
            values.push(file_name);
        }
    }
    values
}

#[allow(dead_code)]
pub fn parse_maven_date_time(path: &str) -> Result<NaiveDateTime, InternalError> {
    let result = NaiveDateTime::parse_from_str(path, "%Y%m%d%H%M%S")?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::repository::maven::utils::parse_maven_date_time;

    #[test]
    fn parse_maven_date_time_test() {
        println!(
            "{}",
            parse_maven_date_time("20211201213303")
                .unwrap()
                .format("%Y-%m-%dT%H:%M:%S.%3fZ")
        );
    }
}
