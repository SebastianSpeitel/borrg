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
            .tick_strings(&["▱▱▱▱", "▰▱▱▱", "▰▰▱▱", "▱▰▰▱", "▱▱▰▰", "▱▱▱▰", "▰▰▰▰"]);
        pb.set_style(sty);

        pb.enable_steady_tick(Duration::from_secs(1));
        // indicatif::ProgressStyle::with_template(&template)
        //     //.tick_strings(&vec!["▱▱▱▱", "▰▱▱▱", "▰▰▱▱", "▱▰▰▱", "▱▱▰▰", "▱▱▱▰"])
        //     .template(&template),

        let backup = std::sync::Arc::new(backup);
        let borg = borg.clone();

        let tx = tx.clone();
        let handle = std::thread::spawn(move || {
            let res =
                borg.create_archive::<backend::borg::BorgWrapper>(&backup.0, &backup.1, |e| {
                    tx.send((idx, e)).unwrap();
                });

            if let Err(e) = res {
                tx.send((idx, crate::Event::Error(e))).unwrap();
            }
        });

        handles.push((handle, pb, prefix));
    }
    // Drop original tx so that the receiver stops when all threads finish
    drop(tx);

    for (idx, event) in rx {
        let (_, pb, prefix) = &mut handles[idx];
        use crate::borrg::Event as E;
        match event {
            E::ArchiveProgress {
                nfiles,
                original_size,
                compressed_size,
                deduplicated_size,
                path,
                ..
            } => {
                let mut prefix = Vec::with_capacity(4);
                prefix.push(format!("O {}", indicatif::HumanBytes(original_size)));

                prefix.push(format!("C {}", indicatif::HumanBytes(compressed_size)));

                prefix.push(format!("D {}", indicatif::HumanBytes(deduplicated_size)));

                pb.set_position(nfiles);
                prefix.push(format!("N {nfiles}"));

                pb.set_prefix(prefix.join(" "));

                pb.set_message(format!("{}", path.display()));
            }
            E::Error(e) => {
                pb.println(format!("{prefix}Error: {e}"));
            }
            ev => {
                pb.println(format!("{prefix}{ev}"));
            }
        }
    }

    mp.clear().unwrap();

    for (handle, _, _) in handles {
        handle.join().unwrap();
    }
}
