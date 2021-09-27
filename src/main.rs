use clap::{App, Arg};
use git_url_parse::GitUrl;
use octocrab;
use tokio;

use chrono::Local;
use chrono_humanize;
#[macro_use]
extern crate prettytable;
use prettytable::{format, Table};

fn main() {
    let path = shellexpand::tilde("~/.config/rtrack.yaml");
    let matches = App::new("rtrack")
        .arg(Arg::with_name("config").short("c").default_value(&*path))
        .get_matches();

    let mut cfg = config::Config::default();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut table = Table::new();
    let now = Local::now();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    cfg.merge(config::File::with_name(matches.value_of("config").unwrap()))
        .unwrap()
        .get_array("repos")
        .into_iter()
        .for_each(|x| {
            x.iter().for_each(|t| {
                rt.block_on(async {
                    let url = GitUrl::parse(&t.to_string()).unwrap();
                    let release = octocrab::instance()
                        .repos(url.owner.unwrap(), url.name.to_string())
                        .releases()
                        .get_latest()
                        .await
                        .unwrap();

                    table.add_row(row![
                        t.to_string(),
                        release.tag_name,
                        chrono_humanize::HumanTime::from(
                            release
                                .published_at
                                .unwrap()
                                .signed_duration_since(now)
                                .to_owned()
                        )
                        .to_string()
                    ]);
                });
            });
        });

    table.printstd();
}
