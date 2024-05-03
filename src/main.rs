#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

mod settings;

use clap::{crate_version, Arg, ArgAction, Command};
use git_url_parse::GitUrl;
use octocrab::Octocrab;
use chrono::Local;
use chrono_humanize::HumanTime;
use colored::*;

use tokio::sync::mpsc;

#[macro_use]
extern crate prettytable;
use prettytable::{format, Table};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let app = Command::new("rtrack")
        .about("get releases from github")
        .version(crate_version!())
        .arg(
            Arg::new("config")
                .help("Path to config file")
                .short('c')
                .long("config")
                .action(ArgAction::Set),
        );
    let matches: clap::ArgMatches = app.get_matches();
    let default_config_file = shellexpand::tilde("~/.config/rtrack.yaml");
    let config_file = matches
        .get_one::<String>("config")
        .map(|s| s.as_str())
        .unwrap_or(
            &*default_config_file
        );
    
    let config = settings::AppConfig::new(config_file).unwrap();

    let now = Local::now();    
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);    

    let (tx, mut rx) = mpsc::channel(32);

    config.repos.into_iter().for_each(|repo|{
        let url = GitUrl::parse(&repo).unwrap();
        let octo = Octocrab::builder().personal_token(config.token.clone()).build().unwrap();

        let tx = tx.clone();
        tokio::spawn(async move {
            let release = octo
                .repos(url.owner.unwrap(), url.name.to_string())
                .releases()
                .get_latest()
                .await
                .unwrap();

            let release_time = release
                    .published_at
                    .unwrap()
                    .signed_duration_since(now)
                    .to_owned();

            let output = match release_time.num_hours().abs() {
                0..=24 => {
                    HumanTime::from(release_time).to_string().green()
                },
                25..=48 => {
                    HumanTime::from(release_time).to_string().blue()
                },
                _ => {
                    HumanTime::from(release_time).to_string().normal()
                }
                
            };
            
            tx.send(row![repo.to_string(), release.tag_name, output]).await.unwrap();
        });
    });

    drop(tx);
    let mut rows = Vec::new();
    while let Some(row) = rx.recv().await {
        rows.push(row);
    }

    rows.sort_by(|a, b| {
        let a = a.get_cell(0).unwrap().to_string();
        let b = b.get_cell(0).unwrap().to_string();
        a.cmp(&b)
    });

    rows.into_iter().into_iter().for_each(|row| {
        table.add_row(row);
    });

    table.printstd();
}
