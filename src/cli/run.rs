use crate::{backend, Borg};

use super::*;

#[derive(Args, Debug)]
pub struct Args {
    #[clap(short, long)]
    progress: bool,

    #[clap(short, long)]
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
        let pb = mp.add(indicatif::ProgressBar::new(u64::MAX));
        let prefix = if multi {
            format!("[{}::{}] ", &backup.0, &backup.1)
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
            .tick_strings(&["▱▱▱▱", "▰▱▱▱", "▰▰▱▱", "▱▰▰▱", "▱▱▰▰", "▱▱▱▰"]);
        pb.set_style(sty);

        pb.enable_steady_tick(Duration::from_secs(1));
        // indicatif::ProgressStyle::with_template(&template)
        //     //.tick_strings(&vec!["▱▱▱▱", "▰▱▱▱", "▰▰▱▱", "▱▰▰▱", "▱▱▰▰", "▱▱▱▰"])
        //     .template(&template),

        let backup = std::sync::Arc::new(backup);
        let borg = borg.clone();

        let tx = tx.clone();
        let handle = std::thread::spawn(move || {
            let events = borg.create_archive::<backend::borg::BorgWrapper>(&backup.0, &backup.1);

            let events = match events {
                Ok(e) => e,
                Err(e) => {
                    log::error!("{}", e);
                    return;
                }
            };

            for event in events {
                tx.send((idx, event)).unwrap();
            }
            // .unwrap_or_else(|e| {
            //     tx.send((
            //         idx,
            //         borg::Event::Error {
            //             message: e.to_string(),
            //         },
            //     ))
            //     .unwrap()
            // });
        });

        handles.push((handle, pb, prefix));
    }
    // Drop original tx so that the receiver stops when all threads finish
    drop(tx);

    for (idx, event) in rx {
        let (_, pb, prefix) = &mut handles[idx];
        use crate::borrg::Event::*;
        match event {
            ArchiveProgress {
                nfiles,
                original_size,
                compressed_size,
                deduplicated_size,
                path,
                ..
            } => {
                let mut prefix = vec![];
                prefix.push(format!("O {}", indicatif::HumanBytes(original_size)));

                prefix.push(format!("C {}", indicatif::HumanBytes(compressed_size)));

                prefix.push(format!("D {}", indicatif::HumanBytes(deduplicated_size)));

                pb.set_position(nfiles);
                prefix.push(format!("N {}", nfiles));

                pb.set_prefix(prefix.join(" "));

                pb.set_message(format!("{}", path.display()));
            }
            ProgressMessage {
                message: Some(message),
                ..
            } => {
                pb.println(format!("{}{}", prefix, message));
            }
            LogMessage { message, .. } => {
                pb.println(format!("{}{}", prefix, message));
            }
            Error(e) => {
                pb.println(format!("{}Error: {}", prefix, e));
            }
            _ => {}
        }
    }

    mp.clear().unwrap();

    for (handle, _, _) in handles {
        handle.join().unwrap();
    }
}
