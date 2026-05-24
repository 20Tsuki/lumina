use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
pub struct Series {
    pub id: i64,
    pub title: String,
    pub year: Option<i64>,
    pub plot: Option<String>,
    pub poster_url: Option<String>,
    pub tmdb_id: Option<i64>,
}

#[allow(dead_code)]
#[derive(FromRow, Serialize)]
pub struct Season {
    pub id: i64,
    pub series_id: i64,
    pub season_number: i64,
}

#[allow(dead_code)]
#[derive(FromRow, Serialize)]
pub struct Episode {
    pub id: i64,
    pub season_id: i64,
    pub episode_number: i64,
    pub title: String,
    pub file_id: Option<i64>,
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct SeriesDetail {
    pub id: i64,
    pub title: String,
    pub year: Option<i64>,
    pub plot: Option<String>,
    pub poster_url: Option<String>,
    pub seasons: Vec<SeasonDetail>,
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct SeasonDetail {
    pub id: i64,
    pub season_number: i64,
    pub episodes: Vec<Episode>,
}
