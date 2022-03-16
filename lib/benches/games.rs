use criterion::{criterion_group, criterion_main, Criterion};
use vhs_diff::*;

use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;

#[derive(Debug, PartialEq, Clone, Patch, Diff, Serialize, Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct GameUpdate {
    id: String,
    day: i16,
    phase: Option<i16>,
    season: i16,
    // this block can be turned into lookup tables if needed
    away_team_name: String,
    home_team_name: String,
    away_team_color: String,
    home_team_color: String,
    away_team_emoji: String,
    home_team_emoji: String,
    away_batter_name: String,
    home_batter_name: String,
    away_team_nickname: String,
    home_team_nickname: String,
    away_team_secondary_color: String,
    home_team_secondary_color: String,
    // -/
    away_odds: f64,
    home_odds: f64,
    finalized: bool,
    game_start: bool,
    base_runner_names: Vec<String>,
    tournament: Option<i8>,
    repeat_count: Option<i16>,
    terminology: Option<String>,
    home_team: Option<String>,
    away_team: Option<String>,
    home_batter: Option<String>,
    away_batter: Option<String>,
    home_batter_mod: Option<String>,
    away_batter_mod: Option<String>,
    home_pitcher: Option<String>,
    away_pitcher: Option<String>,
    home_pitcher_name: Option<String>,
    away_pitcher_name: Option<String>,
    home_pitcher_mod: Option<String>,
    away_pitcher_mod: Option<String>,
    home_outs: Option<i64>,
    home_strikes: Option<i64>,
    home_balls: Option<i64>,
    home_bases: Option<i64>,
    away_outs: Option<i64>,
    away_strikes: Option<i64>,
    away_balls: Option<i64>,
    away_bases: Option<i64>,
    stadium_id: Option<String>,
    weather: Option<i64>,
    series_length: Option<i64>,
    series_index: Option<i64>,
    #[serde(rename = "isPostseason")]
    is_post_season: Option<bool>,
    is_title_match: Option<bool>,
    inning: Option<i64>,
    top_of_inning: Option<bool>,
    half_inning_score: Option<f64>,
    home_score: Option<f64>,
    away_score: Option<f64>,
    at_bat_balls: Option<i64>,
    at_bat_strikes: Option<i64>,
    half_inning_outs: Option<i64>,
    baserunner_count: Option<i64>,
    bases_occupied: Vec<Option<i64>>, // i have no idea why this is an option but i'm keeping it that way out of fear
    base_runner_mods: Vec<String>,
    base_runners: Vec<String>,
    last_update: Option<String>,
    score_ledger: Option<String>,
    score_update: Option<String>,
    away_team_batter_count: Option<i64>,
    home_team_batter_count: Option<i64>,
    shame: Option<bool>,
    outcomes: Vec<String>,
    secret_baserunner: Option<String>,
    state: Option<JSONValue>,
    play_count: i64,
    game_complete: Option<bool>,
    statsheet: Option<String>,
    rules: Option<String>,
}

fn full_game_decode(game: &[u8], data: &[&[u8]]) {
    let mut base: GameUpdate = rmp_serde::from_slice(game).unwrap();

    for patch in data {
        PatchDeserializer::apply(&mut base, &mut rmp_serde::Deserializer::new(*patch)).unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let games: Vec<GameUpdate> = serde_json::from_str(include_str!("../game.json")).unwrap();
    let base = rmp_serde::to_vec(&games[0]).unwrap();

    let mut patches: Vec<Vec<u8>> = Vec::new();

    for vals in games.windows(2) {
        patches.push(rmp_serde::to_vec(&vals[0].diff(vals[1].clone())).unwrap());
    }

    let patches_borrow: Vec<&[u8]> = patches.iter().map(|v| v.as_ref()).collect();

    c.bench_function("patch a full game", |b| {
        b.iter(|| full_game_decode(&base, &patches_borrow[..]))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
