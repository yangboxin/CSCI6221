pub mod lattice_decrypt {
    use std::fs::File;
    use std::env;
    use std::error::Error;  
    use std::io::{self, BufRead};
    use std::path::Path;
    use nalgebra::DMatrix;
    use csv::Writer;


    fn initialize_file_reader<P: AsRef<Path>>(path: P) -> io::Result<io::BufReader<File>> {
        let file = File::open(path)?;
        Ok(io::BufReader::new(file))
    }

    fn process_file<P: AsRef<Path>>(path: P) -> Result<DMatrix<f64>, Box<dyn std::error::Error>> {
        let reader = initialize_file_reader(&path)?;
        let mut matrix = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let mut numbers = Vec::new();
            let mut current_number = String::new();
            
            for fart in line.chars() {
                if fart.is_digit(10) || fart == '.' || fart == '-' {
                    current_number.push(fart);
                } else if !current_number.is_empty() {
                    if let Ok(num) = current_number.parse::<f64>() {
                        numbers.push(num);
                        current_number.clear();
                    } else {
                        // Handle parsing error here
                    }
                }
            }
            if !numbers.is_empty() {
                matrix.push(numbers);
            }

            // for row in &matrix {
            //     println!("{:?}", row);
            // }
        }
        // Convert pushed numbers array to matrix<f64>
        // Convert Vec<Vec<u64>> to nalgebra DMatrix
        let rows = matrix.len();

        let cols = matrix[0].len(); // Assuming all inner vectors have the same length
        let output_matrix = DMatrix::from_fn(rows, cols, |i, j| matrix[i][j]);

        Ok(output_matrix)
    }

    fn decrypt(encrypted_matrix: Result<DMatrix<f64>, Box<dyn std::error::Error>>, secret_key: Result<DMatrix<f64>, Box<dyn std::error::Error>>) -> Vec<Vec<u64>> {
        // Unwrap the results or handle the errors
        let encrypted_matrix = match encrypted_matrix {
            Ok(matrix) => matrix,
            Err(_err) => return vec![vec![]], // Return an empty vector if there's an error
        };
        let secret_key = match secret_key {
            Ok(matrix) => matrix,
            Err(_err) => return vec![vec![]], // Return an empty vector if there's an error
        };

        // Double check encryption isn't invalid
        if encrypted_matrix.ncols() != secret_key.nrows() {
            println!("Incompatible dimensions for matrix multiplication");
            return vec![vec![]]; // Return an empty vector if dimensions are incompatible
        }

        // Perform matrix multiplication
        let testing_f64 = encrypted_matrix * secret_key;

        // Convert testing matrix back to u64 and maintain shape
        let mut decrypted_matrix = Vec::new();
        for row in testing_f64.row_iter() {
            let mut row_values = Vec::new();
            for &elem in row.iter() {
                row_values.push(elem.round() as u64);
            }
            decrypted_matrix.push(row_values);
        }

        decrypted_matrix
    }

    fn utf8_to_string(strings: Vec<Vec<u64>>) -> Result<Vec<String>, Box<dyn Error>> {
        let mut result_strings = Vec::new();
        for line in strings {
            let mut bytes = Vec::new();
            for &val in &line {
                bytes.push(val as u8);
            }
            let string = String::from_utf8(bytes)?;
            result_strings.push(string);
        }
        Ok(result_strings)
    }

    // New function to process strings and write to a file
    fn write_processed_strings_to_file(strings: Vec<String>, file_path: &str) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_writer(File::create(file_path)?);

        for (index, line) in strings.iter().enumerate() {
            // Remove trailing '|' characters and then split by commas
            let processed_line = line.trim_end_matches('|');
            let record: Vec<&str> = processed_line.split(',').collect();

            if index == 0 {
                // The first line contains headers
                wtr.write_record(&record)?;
            } else {
                // Subsequent lines are data rows
                wtr.write_record(&record)?;
            }
        }

        // Flush data to the CSV file
        wtr.flush()?;
        Ok(())
    }

    pub fn lattice_decrypt_csv(encrypted_matrix_path: &str, private_key_path: &str, public_key_path: &str) -> io::Result<()> {        
        // Specify the path to the encrypted_matrix
        let encrypted_matrix_path = encrypted_matrix_path;
        let encrypted_matrix = process_file(encrypted_matrix_path);
        // Specify
        let secret_key_path = private_key_path;
        let secret_key = process_file(secret_key_path);
        
        let output_file_path = "temp_decrypted_output/output.csv";

        let result = decrypt(encrypted_matrix, secret_key);
        // Print the shape of the decrypted matrix
        let _num_rows = result.len();
        let _num_cols = if let Some(row) = result.get(0) {
            row.len()
        } else {
            0
        };
        // println!("Number of rows in decrypted matrix: {}", num_rows);
        // println!("Number of columns in decrypted matrix: {}", num_cols);
        // utf8_to_string(result);
        match utf8_to_string(result) {
            Ok(strings) => {
                // Update the path as needed
                if let Err(e) = write_processed_strings_to_file(strings, output_file_path) {
                    eprintln!("Failed to write processed strings to file: {}", e);
                }
            },
            Err(e) => eprintln!("Failed to convert matrix to strings: {}", e),
        }
        Ok(())
    }
}