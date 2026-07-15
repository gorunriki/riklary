use axum::{Json, Router, routing::get};
use epub::doc::EpubDoc;
use serde::Serialize;
use std::fs::{self, File, read_dir};
use std::io::{Read, Seek};
use std::path::Path;
use std::vec::Vec;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[derive(Serialize, Debug)]
struct BookInfo {
    title: String,
    author: String,
    date: String,
    subjects: Vec<String>,
    file_name: String,
    extracted_path: String,
}

impl BookInfo {
    fn from_epub<R: Read + Seek>(
        doc: &EpubDoc<R>,
        file_name: String,
        extracted_path: String,
    ) -> Self {
        // Get singel string metadata properties
        let get_meta = |prop_name: &str| {
            doc.metadata
                .iter()
                .find(|item| item.property == prop_name)
                .map(|item| item.value.clone())
                .unwrap_or_default()
        };

        // Get multiple string metadata properties (book subjects)
        let subjects = doc
            .metadata
            .iter()
            .filter(|item| item.property == "subject")
            .map(|item| item.value.clone())
            .collect();

        BookInfo {
            title: get_meta("title"),
            author: get_creator_meta(doc),
            date: get_meta("date"),
            subjects,
            file_name,
            extracted_path,
        }
    }
}

fn get_creator_meta<R: Read + Seek>(doc: &EpubDoc<R>) -> String {
    doc.metadata
        .iter()
        .find(|item| item.property == "creator")
        .map(|item| item.value.clone())
        .unwrap_or_else(|| "Unknown Author".to_string())
}

fn extract_epub(file_path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(file_path)?; // Membuka file dengan tanda tanya (?) mengembalikan File asli
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => output_dir.join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

async fn get_books_api() -> Json<Vec<BookInfo>> {
    let books_dir = "./books/";
    let extracted_base = "./books/extracted/";

    let _ = fs::create_dir_all(extracted_base);

    let book_paths = read_dir(books_dir).expect("Gagal membaca folder books/!");
    let mut books: Vec<BookInfo> = vec![];

    for book_path in book_paths {
        let entry = match book_path {
            Ok(e) => e,
            Err(_) => continue,
        };

        let file_path = entry.path();

        if file_path.is_dir() {
            continue;
        }

        if file_path.extension().and_then(|ext| ext.to_str()) == Some("epub") {
            if let Ok(doc) = EpubDoc::new(file_path.clone()) {
                let file_name = file_path
                    .file_name()
                    .and_then(|os_str| os_str.to_str())
                    .unwrap_or_default()
                    .to_string();

                let folder_name = file_name.replace(".epub", "");
                let output_dir = Path::new(extracted_base).join(&folder_name);

                if !output_dir.exists() {
                    println!("Mengekstrak e-book baru: {}", file_name);
                    if let Err(e) = extract_epub(&file_path, &output_dir) {
                        println!("Gagal mengekstrak {}: {}", file_name, e);
                        continue;
                    }
                }

                let extracted_path = format!("extracted/{}/", folder_name);

                // Memperbaiki pemanggilan dengan mengoper parameter ketiga: extracted_path
                books.push(BookInfo::from_epub(&doc, file_name, extracted_path));
            }
        }
    }

    Json(books)
}

#[tokio::main]
async fn main() {
    let serve_dir = ServeDir::new("./books");

    let app = Router::new()
        .route("/api/books", get(get_books_api))
        .nest_service("/static", serve_dir)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    println!("Server berjalan di http://localhost:8080");

    axum::serve(listener, app).await.unwrap();
}

// fn main() {
//     // Open dan baca isi folder
//     let book_paths = read_dir("./books/").expect("Gagal membaca folder");

//     let mut books: Vec<BookInfo> = vec![];

//     for book_path in book_paths {
//         // handle error jika ada problem saat get path
//         let entry = match book_path {
//             Ok(e) => e,
//             Err(_) => continue,
//         };

//         let file_path = entry.path();

//         if file_path.extension().and_then(|ext| ext.to_str()) == Some("epub") {
//             println!("Memproses: {:?}", entry.file_name());

//             if let Ok(doc) = EpubDoc::new(file_path) {
//                 books.push(BookInfo::from_epub(&doc));
//             } else {
//                 println!("Gagal membuka file {:?}", entry.file_name());
//             }
//         }
//     }

//     println!("{:#?}", books);
// }
