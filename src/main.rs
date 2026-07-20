use axum::{
    extract::{DefaultBodyLimit, Multipart},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use epub::doc::EpubDoc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Seek};
use tower_http::cors::CorsLayer;

#[derive(Serialize, Deserialize, Debug, Clone)]
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

struct SupabaseConfig {
    url: String,
    key: String,
    bucket: String,
}

impl SupabaseConfig {
    fn from_env() -> Result<Self, String> {
        let url = std::env::var("SUPABASE_URL")
            .map_err(|_| "Variabel SUPABASE_URL tidak ditemukan".to_string())?;
        let key = std::env::var("SUPABASE_KEY")
            .map_err(|_| "Variabel SUPABASE_KEY tidak ditemukan".to_string())?;
        let bucket = std::env::var("SUPABASE_BUCKET")
            .unwrap_or_else(|_| "books".to_string());
        Ok(Self { url, key, bucket })
    }
}

fn get_mime_type(file_name: &str) -> &'static str {
    let ext = file_name.split('.').last().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "html" | "xhtml" => "text/html",
        "css" => "text/css",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "xml" | "opf" | "ncx" => "application/xml",
        "js" => "application/javascript",
        "json" => "application/json",
        "epub" => "application/epub+zip",
        _ => "application/octet-stream",
    }
}

async fn upload_to_supabase(
    client: &Client,
    config: &SupabaseConfig,
    file_path: &str,
    data: Vec<u8>,
    mime_type: &str,
) -> Result<(), String> {
    let url = format!(
        "{}/storage/v1/object/{}/{}",
        config.url.trim_end_matches('/'),
        config.bucket,
        file_path
    );

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.key))
        .header("apikey", &config.key)
        .header("Content-Type", mime_type)
        .header("x-upsert", "true")
        .body(data)
        .send()
        .await
        .map_err(|e| format!("Gagal mengirim request upload: {}", e))?;

    if !response.status().is_success() {
        let err_body = response.text().await.unwrap_or_default();
        return Err(format!(
            "Upload ke Supabase Storage gagal untuk {}: {}",
            file_path, err_body
        ));
    }

    Ok(())
}

async fn insert_book_to_supabase(
    client: &Client,
    config: &SupabaseConfig,
    book: &BookInfo,
) -> Result<(), String> {
    let url = format!("{}/rest/v1/books", config.url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.key))
        .header("apikey", &config.key)
        .header("Content-Type", "application/json")
        .header("Prefer", "resolution=merge-duplicates")
        .json(book)
        .send()
        .await
        .map_err(|e| format!("Gagal mengirim request database: {}", e))?;

    if !response.status().is_success() {
        let err_body = response.text().await.unwrap_or_default();
        return Err(format!("Gagal menyimpan metadata ke database: {}", err_body));
    }

    Ok(())
}

async fn get_books_api() -> Result<Json<Vec<BookInfo>>, StatusCode> {
    let config = match SupabaseConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Kesalahan konfigurasi Supabase: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let client = Client::new();
    let url = format!("{}/rest/v1/books?select=*", config.url.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.key))
        .header("apikey", &config.key)
        .send()
        .await
        .map_err(|e| {
            eprintln!("Gagal mengambil buku dari database: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !response.status().is_success() {
        eprintln!("Status response database error: {}", response.status());
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let books: Vec<BookInfo> = response.json().await.map_err(|e| {
        eprintln!("Gagal mengurai JSON daftar buku: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(books))
}

async fn upload_book_api(
    mut multipart: Multipart,
) -> Result<Json<BookInfo>, (StatusCode, String)> {
    let config = SupabaseConfig::from_env()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Konfigurasi error: {}", e)))?;

    let client = Client::new();
    let mut file_name = None;
    let mut file_data = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart stream error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            file_name = field.file_name().map(|s| s.to_string());
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal membaca data file: {}", e)))?,
            );
        }
    }

    let file_name = match file_name {
        Some(name) => name,
        None => return Err((StatusCode::BAD_REQUEST, "File input wajib diisi".to_string())),
    };

    if !file_name.ends_with(".epub") {
        return Err((StatusCode::BAD_REQUEST, "File harus berformat .epub".to_string()));
    }

    let file_bytes = match file_data {
        Some(data) => data.to_vec(),
        None => return Err((StatusCode::BAD_REQUEST, "Data file kosong".to_string())),
    };

    // 1. Ekstrak metadata dari EPUB di memori
    let cursor = Cursor::new(file_bytes.clone());
    let doc = EpubDoc::from_reader(cursor)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Gagal membaca EPUB: {}", e)))?;

    let folder_name = file_name.replace(".epub", "");
    let extracted_base_url = format!(
        "{}/storage/v1/object/public/{}/extracted/{}/",
        config.url.trim_end_matches('/'),
        config.bucket,
        folder_name
    );

    let book_info = BookInfo::from_epub(&doc, file_name.clone(), extracted_base_url);

    // 2. Upload file EPUB asli ke Supabase Storage
    let epub_path = format!("epubs/{}", file_name);
    upload_to_supabase(
        &client,
        &config,
        &epub_path,
        file_bytes.clone(),
        "application/epub+zip",
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal upload EPUB: {}", e)))?;

    // 3. Ekstrak ZIP di memori dan upload kontennya ke Supabase Storage
    let zip_cursor = Cursor::new(file_bytes);
    let mut archive = zip::ZipArchive::new(zip_cursor)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Format zip EPUB tidak valid: {}", e)))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Gagal membaca file di dalam ZIP index {}: {}", i, e),
            )
        })?;
        let file_path_in_zip = file.name().to_string();

        if file_path_in_zip.ends_with('/') {
            continue;
        }

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Gagal membaca konten file {}: {}", file_path_in_zip, e),
            )
        })?;

        let mime = get_mime_type(&file_path_in_zip);
        let storage_path = format!("extracted/{}/{}", folder_name, file_path_in_zip);

        upload_to_supabase(&client, &config, &storage_path, buffer, mime)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Gagal mengunggah file terekstrak {}: {}", file_path_in_zip, e),
                )
            })?;
    }

    // 4. Simpan metadata buku ke Supabase Database
    insert_book_to_supabase(&client, &config, &book_info)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal simpan database: {}", e)))?;

    Ok(Json(book_info))
}

#[tokio::main]
async fn main() {
    // Memuat variabel lingkungan dari file .env jika ada (untuk development lokal)
    let _ = dotenvy::dotenv();

    let app = Router::new()
        .route("/api/books", get(get_books_api))
        .route("/api/upload", post(upload_book_api))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // Batasan upload maks 100MB
        .layer(CorsLayer::permissive());

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT harus berupa angka");

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    println!("Server berjalan di http://0.0.0.0:{}", port);

    axum::serve(listener, app).await.unwrap();
}
