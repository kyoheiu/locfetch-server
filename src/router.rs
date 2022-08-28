use axum::{
    body,
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokei::Language;

#[derive(Error, Debug)]
pub enum MyError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("reqwest error")]
    RequestGet(String),
    #[error("git clone error")]
    GitClone(String),
    #[error("PathBuf to str error")]
    TempPath(String),
}

impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        let body = match self {
            MyError::Io(msg) => body::boxed(body::Full::from(msg.to_string())),
            MyError::RequestGet(msg) => body::boxed(body::Full::from(msg)),
            MyError::GitClone(msg) => body::boxed(body::Full::from(msg)),
            MyError::TempPath(msg) => body::boxed(body::Full::from(msg)),
        };
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body)
            .unwrap()
    }
}

#[derive(Serialize, Debug)]
pub struct ResponseBody {
    origin: String,
    stats: Vec<(String, Stat)>,
    total: Stat,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Input {
    pub url: String,
}

#[derive(Serialize, Debug, Clone)]
struct Stat {
    files: usize,
    lines: usize,
    codes: usize,
    comments: usize,
    blanks: usize,
}

fn make_stat(lang: Language) -> Stat {
    let lines = lang.code + lang.comments + lang.blanks;
    let files = lang.reports.len();
    Stat {
        files,
        lines,
        codes: lang.code,
        comments: lang.comments,
        blanks: lang.blanks,
    }
}

pub async fn health() -> Result<String, MyError> {
    Ok("Hello, developer.".to_string())
}

pub async fn stats(Json(de): Json<Input>) -> Result<Json<ResponseBody>, MyError> {
    info!("fn stats started.");
    let url = de.url;
    info!("Target is: {}", url);
    if let Ok(res) = reqwest::get(&url).await {
        if !res.status().is_success() {
            let message = "GET request failed.";
            warn!("{}", message);
            return Err(MyError::RequestGet(message.to_string()));
        }
    } else {
        let message = "URL seems invalid.";
        warn!("{}", message);
        return Err(MyError::RequestGet(message.to_string()));
    }

    let temp_path = tempfile::TempDir::new()?.into_path();
    info!("{:?}", temp_path);
    info!("temp_path ready.");
    let temp_path_as_str = temp_path.to_str();
    if temp_path_as_str.is_none() {
        let message = "Failed to make a temporary directory.";
        warn!("{}", message);
        return Err(MyError::TempPath(message.to_string()));
    }

    if std::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            &url,
            temp_path.to_str().unwrap(),
            "-q",
        ])
        .status()
        .is_err()
    {
        let message = "Failed to git clone.";
        warn!("{}", message);
        return Err(MyError::GitClone(message.to_string()));
    }
    info!("git clone finished.");

    let git_path = temp_path.join(".git");
    if !git_path.exists() {
        let message = ".git directory not found.";
        warn!("{}", message);
        return Err(MyError::GitClone(message.to_string()));
    }

    let config = tokei::Config::default();
    let mut languages = tokei::Languages::new();
    languages.get_statistics(&[&temp_path], &[], &config);

    let mut stats = Vec::new();
    for (lang_type, contents) in languages {
        let stat = make_stat(contents);
        stats.push((lang_type.to_string(), stat));
    }
    stats.sort_by(|a, b| b.1.lines.cmp(&a.1.lines));

    let mut total = Stat {
        files: 0,
        lines: 0,
        codes: 0,
        comments: 0,
        blanks: 0,
    };
    for stat in stats.iter().map(|x| x.1.clone()) {
        total.files += stat.files;
        total.lines += stat.lines;
        total.codes += stat.codes;
        total.comments += stat.comments;
        total.blanks += stat.blanks;
    }

    let j = ResponseBody {
        origin: url,
        stats,
        total,
    };
    std::fs::remove_dir_all(temp_path.clone())?;
    assert!(!temp_path.exists());
    info!("Deleted temporary directory.");
    Ok(Json(j))
}
