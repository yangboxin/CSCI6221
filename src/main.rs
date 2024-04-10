pub mod encrypted;
pub mod decrypted;
pub mod verification;

use actix_multipart::form::tempfile::{TempFile, TempFileConfig};
use actix_multipart::form::MultipartForm;
use actix_web::{post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result,Error};
use actix_files as afs;
use std::{fs, io::prelude::*};
use std::fs::File;
use std::fs::remove_file;
use std::process::Command;
use log::{info};
use encrypted::lattice_encrypt::lattice_encrypt_csv;
use decrypted::lattice_decrypt::lattice_decrypt_csv;
use verification::lattice_verification::lattice_public_secret_verification;


fn read_html_file(file_path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}


async fn index(req: HttpRequest) -> Result<HttpResponse> {
    let html_content = read_html_file("/Users/gurudeepmachupalli/Documents/Repositories/LATTICE/hello_world/src/frontend.html")?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html_content))
}

// 
async fn encrypt_file(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    info!("File path to be encrypted: {}", file_path);
    
    let output = lattice_encrypt_csv(file_path);

    info!("Output of Lattice Encryption: {:?}", output);

    match output {
        Ok(result) => {
            info!("Lattice Encryption has succeeded: {:?}", result);
            Ok(result)
        },
        Err(error) => {
            info!("Lattice Encryption has failed: {:?}", error);
            Err(error.into())
        },
    }

}

#[post("/encrypt")]
async fn encrypt_handler(MultipartForm(upload_encrypt_form): MultipartForm<UploadEncryptForm>) -> Result<HttpResponse> {

    // Pulls the name of the file from the Form that's passed in from the UI
    let filename = match upload_encrypt_form.encrypt_file.file_name {
        Some(name) => name.to_string(),
        None => return Ok(HttpResponse::BadRequest().body("Bad Request no CSV file is provided.")),
    };
    let path = format!("./temp_encrypted_input/{}", filename);
    
    // Will reate the CSV file to be encrypted in the temp_encrypted_input folder
    if let Err(error) = upload_encrypt_form.encrypt_file.file.persist(&path) {
        eprintln!("Failed to save CSV file: {}", error);
        return Ok(HttpResponse::InternalServerError().body("Failed to save CSV file"));
    }
    info!("Saving CSV file to Temp Path: {}", path);

    // Trigger the encrypt_file function
    let encryption_result = encrypt_file(&path).await;

    // Will safely remove the file after the encrypt_file function is run
    let _ = remove_file(&path);

    match encryption_result {
        Ok(_) => Ok(HttpResponse::Ok().body("File encrypted successfully")),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Encryption failed")),
    }
}


#[derive(MultipartForm)]
struct UploadEncryptForm {
    // Pulls the file that is uploaded to input field with the name fileInputEncrypt
    #[multipart(rename = "fileInputEncrypt")]
    encrypt_file: TempFile,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    log::info!("Temporary input and output folders being created.");
    std::fs::create_dir_all("./temp_encrypted_input")?;
    std::fs::create_dir_all("./temp_encrypted_output")?;
    std::fs::create_dir_all("./temp_decrypted_input")?;
    std::fs::create_dir_all("./temp_decrypted_output")?;

    log::info!("HTTP server starting: http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            // This creates temporary file storage
            .app_data(TempFileConfig::default().directory("./temp_encrypted_input"))
            .app_data(TempFileConfig::default().directory("./temp_encrypted_output"))
            .app_data(TempFileConfig::default().directory("./temp_decrypted_input"))
            .app_data(TempFileConfig::default().directory("./temp_decrypted_output"))
            // This will path the files created files in the temp_encrypted_output folder to the /encrypted endpoint
            .service(actix_files::Files::new("/encrypted", "./temp_encrypted_output").show_files_listing())
            .service(actix_files::Files::new("/decrypted", "./temp_decrypted_output").show_files_listing())
            .route("/", web::get().to(index))
            .service(encrypt_handler)
            .service(decrypt_handler)
    })
    .bind("localhost:8080")?
    .run()
    .await
}

#[derive(MultipartForm)]
struct UploadDecryptForm {
    // Pulls the file that is uploaded to input field with the name fileInputEncrypt
    #[multipart(rename = "fileInputDecryptMatrix")]
    encrypted_matrix_file: TempFile,

    #[multipart(rename = "fileInputDecryptSecretKey")]
    secret_key_file: TempFile,

    #[multipart(rename = "fileInputDecryptPublicKey")]
    public_key_file: TempFile,
}


async fn decrypt_file(encrypted_matrix_file_path: &str, secret_key_file_path: &str, public_key_file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    info!("File paths to be decrypted: {encrypted_matrix_file_path}, {secret_key_file_path} and {public_key_file_path}");
    
    let output = lattice_decrypt_csv(encrypted_matrix_file_path, secret_key_file_path, public_key_file_path);

    info!("Output of Lattice Decryption: {:?}", output);

    match output {
        Ok(result) => {
            info!("Lattice Encryption has succeeded: {:?}", result);
            Ok("Lattice Decryption has succeeded ".to_string())
        },
        Err(error) => {
            info!("Lattice Decryption has failed: {:?}", error);
            Err(error.into())
        },
    }
}

#[post("/decrypt")]
async fn decrypt_handler(MultipartForm(upload_decrypt_form): MultipartForm<UploadDecryptForm>) -> Result<HttpResponse> {

    // Retrives the file names of the uploaded decrypt files from the Form
    let encrypted_matrix_file_name = match upload_decrypt_form.encrypted_matrix_file.file_name {
        Some(name) => name.to_string(),
        None => return Ok(HttpResponse::BadRequest().body("Bad Request no Encrypted Matrix CSV file is provided."))
    };

    let secret_key_file_name = match upload_decrypt_form.secret_key_file.file_name {
        Some(name) => name.to_string(),
        None => return Ok(HttpResponse::BadRequest().body("Bad Request no Secret Key file is provided."))
    };

    let public_key_file_name = match upload_decrypt_form.public_key_file.file_name {
        Some(name) => name.to_string(),
        None => return Ok(HttpResponse::BadRequest().body("Bad Request no Public Key file is provided."))
    };

    let encrypted_matrix_file_path = format!("./temp_decrypted_input/{}", encrypted_matrix_file_name);
    let secret_key_file_path = format!("./temp_decrypted_input/{}", secret_key_file_name);
    let public_key_file_path = format!("./temp_decrypted_input/{}", public_key_file_name);

    
    // Will create the encrypted_matrix_file, secret_key and public_key file to be decrypted in the temp_decrypted_input folder
    if let Err(error) = upload_decrypt_form.encrypted_matrix_file.file.persist(&encrypted_matrix_file_path) {
        eprintln!("Failed to save Encrypted Matrix CSV file: {}", error);
        return Ok(HttpResponse::InternalServerError().body("Failed to Encrypted Matrix CSV file"));
    }
    info!("Saved Encrypted Matrix CSV file to Temp Path: {}", encrypted_matrix_file_path);

    if let Err(error) = upload_decrypt_form.secret_key_file.file.persist(&secret_key_file_path) {
        eprintln!("Failed to save Secret Key file: {}", error);
        return Ok(HttpResponse::InternalServerError().body("Failed to Secret Key file"));
    }
    info!("Saved Secret Key file to Temp Path: {}", encrypted_matrix_file_path);

    if let Err(error) = upload_decrypt_form.public_key_file.file.persist(&public_key_file_path) {
        eprintln!("Failed to save Public Key file: {}", error);
        return Ok(HttpResponse::InternalServerError().body("Failed to Public Key file"));
    }
    info!("Saved Public Key file to Temp Path: {}", public_key_file_path);


    // 
    let decryption_result = decrypt_file(&encrypted_matrix_file_path, &secret_key_file_path, &public_key_file_path).await;

    // Will safely remove the temp_decrypted_input files after the decrypt_file function is run
    let _ = remove_file(&encrypted_matrix_file_path);
    let _ = remove_file(&secret_key_file_path);
    let _ = remove_file(&public_key_file_path);


    match decryption_result {
        Ok(_) => Ok(HttpResponse::Ok().body("File decrypted successfully")),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Encryption failed")),
    }
}