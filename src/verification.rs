pub mod lattice_verification {
    use nalgebra::DMatrix;
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::Path;
    use std::env;

    fn initialize_file_reader<P: AsRef<Path>>(path: P) -> io::Result<io::BufReader<File>> {
        let file = File::open(path)?;
        Ok(io::BufReader::new(file))
    }

    fn read_matrix<P: AsRef<Path>>(path: P) -> Result<DMatrix<f64>, Box<dyn std::error::Error>> {
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
    // Function to read a number from a file.
    fn read_number<P: AsRef<Path>>(filename: P) -> io::Result<f64> {
        let file = File::open(filename)?;
        let line = io::BufReader::new(file).lines().next().ok_or(io::Error::new(io::ErrorKind::Other, "No line found"))??;
        line.parse().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to parse number"))
    }

    pub fn lattice_public_secret_verification() {
        let args: Vec<String> = env::args().collect();

        if args.len() < 3 {
            eprintln!("Usage: {} <matrix_file_path> <number_file_path>", args[0]);
            std::process::exit(1);
        }

        let matrix_path = &args[1];
        let number_path = &args[2];
        let matrix = read_matrix(matrix_path)
            .expect("Failed to read the matrix from file");

        let number = read_number(number_path)
            .expect("Failed to read the number from file");

        let trace = matrix.norm();

        if trace == number {
            println!("The norm of the matrix equals the number: {}", number);
        } else {
            println!("The norm of the matrix does not equal the number: {} != {}", trace, number);
        }
    }


}