use super::*;
use crate::{backend, Borg};
use std::{sync::mpsc, time::Duration};

#[derive(Args, Debug)]
pub struct Args {
    #[arg(short, long)]
    progress: bool,

    #[arg(short, long)]
    dry_run: bool,
}

pub fn run(mut borg: Borg, config: Config, args: Args) {
    if args.dry_run {
        borg.dry_run();
    }

    let borg = std::sync::Arc::new(borg);
    let (tx, rx) = mpsc::channel();
    let mp = indicatif::MultiProgress::new();
    let multi = config.backups.len() > 1;

    let mut handles = vec![];
    for (idx, backup) in config.backups.into_iter().enumerate() {
        let pb = mp.add(indicatif::ProgressBar::no_length());
        let prefix = if multi {
            format!("[{}] ", &backup.repo)
        } else {
            String::new()
        };
        let template = format!(
            "{}{}",
            &prefix, "{elapsed:.dim} {spinner:.green} {prefix:.yellow} {wide_msg}"
        );
        let sty = indicatif::ProgressStyle::default_spinner()
            .template(&template)
            .unwrap()
            // .tick_chars("◜◠◝◞◡◟");
            .tick_strings(&["▱▱▱▱", "▰▱▱▱", "▰▰▱▱", "▱▰▰▱", "▱▱▰▰", "▱▱▱▰", "▰▰▰▰"]);
        pb.set_style(sty);

        // pb.enable_steady_tick(Duration::from_secs(1));
        // indicatif::ProgressStyle::with_template(&template)
        //     //.tick_strings(&vec!["▱▱▱▱", "▰▱▱▱", "▰▰▱▱", "▱▰▰▱", "▱▱▰▰", "▱▱▱▰"])
        //     .template(&template),

        let backup = std::sync::Arc::new(backup);
        let borg = borg.clone();

        let tx = tx.clone();
        let handle = std::thread::spawn(move || {
            let res = borg.create_archive::<backend::borg::BorgWrapper>(&backup, |e| {
                tx.send((idx, e)).unwrap();
            });

            if let Err(e) = res {
                tx.send((idx, e.into())).unwrap();
            }
        });

        handles.push((handle, pb, prefix));
    }
    // Drop original tx so that the receiver stops when all threads finish
    drop(tx);

    for (idx, event) in rx {
        let (_, pb, prefix) = &mut handles[idx];

        if event.r#type() == "archive_progress" {
            event.update_progress(pb);
            pb.enable_steady_tick(Duration::from_secs(1));
            continue;
        }

        if event.r#type() == "question_prompt" {
            pb.disable_steady_tick();
        }

        if let Some(err) = event.error() {
            pb.println(format!("{prefix}Error: {err}"));
        }

        let line = format!("{prefix}{event}");

        pb.println(line);
    }

    mp.clear().unwrap();

    for (handle, _, _) in handles {
        handle.join().unwrap();
    }
}
