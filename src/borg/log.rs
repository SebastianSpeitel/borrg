use std::time::{Duration, SystemTime};

use serde_derive::Deserialize;
use serde_json::{Map, Value};
use smol_str::{SmolStr, SmolStrBuilder};

use crate::util::ByteSize;

type Json = Map<String, Value>;

#[derive(Debug, Deserialize)]
pub struct Event {
    #[serde(default = "default_type")]
    r#type: SmolStr,
    #[serde(default = "default_msgid")]
    msgid: SmolStr,
    #[serde(default, deserialize_with = "deserialize_time")]
    time: Option<SystemTime>,
    #[serde(default)]
    message: Option<SmolStr>,
    #[serde(flatten)]
    json: Json,
    #[serde(skip)]
    error: Option<Box<dyn core::error::Error + Send + Sync>>,
}

impl Event {
    #[inline]
    pub fn from_error(err: impl core::error::Error + Send + Sync + 'static) -> Self {
        Self {
            r#type: SmolStr::new_static("error"),
            msgid: SmolStr::new_static("borrg.error"),
            time: None,
            message: None,
            json: Json::new(),
            error: Some(Box::new(err)),
        }
    }

    #[inline]
    #[must_use]
    pub const fn r#type(&self) -> &SmolStr {
        &self.r#type
    }

    #[inline]
    #[must_use]
    pub const fn msgid(&self) -> &SmolStr {
        &self.msgid
    }

    #[inline]
    #[must_use]
    pub const fn time(&self) -> Option<SystemTime> {
        self.time
    }

    #[inline]
    pub const fn message(&self) -> Option<&SmolStr> {
        self.message.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn error(&self) -> Option<&(dyn core::error::Error + Send + Sync)> {
        self.error.as_deref()
    }

    #[inline]
    fn path(&self) -> Option<&str> {
        self.json.get("path").and_then(Value::as_str)
    }

    #[inline]
    fn status(&self) -> Option<&str> {
        self.json.get("status").and_then(Value::as_str)
    }

    #[inline]
    pub fn file_stats(&self) -> Option<SmolStr> {
        use std::fmt::Write;

        let nfiles = self.json.get("nfiles")?.as_u64()?;

        let mut stats = SmolStrBuilder::new();

        write!(stats, "N {nfiles}").ok()?;

        if let Some(o) = self.json.get("original_size").and_then(Value::as_u64) {
            write!(stats, " O {:.2}B", ByteSize(o)).ok()?;
        }

        #[cfg(feature = "borg1-compat")]
        if let Some(c) = self.json.get("compressed_size").and_then(Value::as_u64) {
            write!(stats, " C {:.2}B", ByteSize(c)).ok()?;
        }

        #[cfg(feature = "borg1-compat")]
        if let Some(d) = self.json.get("deduplicated_size").and_then(Value::as_u64) {
            write!(stats, " D {:.2}B", ByteSize(d)).ok()?;
        }

        #[cfg(feature = "borg2")]
        if let Some(stat_map) = self.json.get("files_stats").and_then(Value::as_object) {
            for (k, v) in stat_map {
                if let Some(v) = v.as_f64() {
                    write!(stats, " {k} {v}").ok()?;
                }
            }
        }

        Some(stats.finish())
    }

    #[inline]
    pub fn update_progress(&self, progress: &mut indicatif::ProgressBar) {
        if let Some(path) = self.json.get("path").and_then(Value::as_str) {
            progress.set_message(path.to_string());
        }

        if let Some(prefix) = self.file_stats() {
            progress.set_prefix(prefix.to_string());
        }

        if let Some(nfiles) = self.json.get("nfiles").and_then(Value::as_u64) {
            progress.set_position(nfiles);
        }

        if self.json.get("finished") == Some(&Value::Bool(true)) {
            progress.finish();
        }
    }
}

impl From<crate::Error> for Event {
    #[inline]
    fn from(err: crate::Error) -> Self {
        Self {
            r#type: SmolStr::new_static("error"),
            msgid: SmolStr::new_static("borrg.error"),
            time: None,
            message: None,
            json: Json::new(),
            error: Some(err),
        }
    }
}

impl core::fmt::Display for Event {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if f.alternate() {
            let mut entry = self.json.clone();
            entry.insert("type".into(), self.r#type.to_string().into());
            entry.insert("msgid".into(), self.msgid.to_string().into());
            if let Some(time) = self.time {
                let dur = time
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|_| core::fmt::Error)?;

                entry.insert("time".into(), dur.as_secs_f64().into());
            }
            let json = serde_json::to_string(&entry).map_err(|_| core::fmt::Error)?;

            return json.fmt(f);
        }

        match self.r#type.as_str() {
            "archive_progress" => {
                let stats = self.file_stats().unwrap_or_default();
                let path = self.path().unwrap_or_default();

                stats.fmt(f)?;
                f.write_str(" ")?;
                path.fmt(f)?;
            }
            "file_status" => {
                let status = self.status().unwrap_or_default();
                let path = self.path().unwrap_or_default();

                status.fmt(f)?;
                f.write_str(" ")?;
                path.fmt(f)?;
            }
            "question_prompt" => {
                let prompt = self
                    .json
                    .get("prompt")
                    .and_then(Value::as_str)
                    .unwrap_or_default();

                prompt.fmt(f)?;
            }
            "question_env_answer" => {
                let answer = self
                    .json
                    .get("answer")
                    .and_then(Value::as_str)
                    .unwrap_or_default();

                answer.fmt(f)?;
            }
            _ => {
                if let Some(msg) = self.message() {
                    f.write_str(msg)?;
                } else {
                    core::fmt::Debug::fmt(self, f)?;
                }
            }
        }
        Ok(())
    }
}

pub struct Reader<R> {
    lines: std::io::Lines<R>,
}

impl<R: std::io::BufRead> Iterator for Reader<R> {
    type Item = Event;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let line = match self.lines.next()? {
            Ok(line) => line,
            Err(e) => return Some(Event::from_error(e)),
        };

        match serde_json::from_str::<Event>(&line) {
            Ok(entry) => Some(entry),
            Err(e) => {
                let mut entry = Event::from_error(e);
                entry.message.replace(line.into());
                Some(entry)
            }
        }
    }
}

#[inline]
pub fn read(read: impl std::io::Read) -> impl Iterator<Item = Event> {
    use std::io::{BufRead, BufReader};
    let reader = BufReader::new(read);
    Reader {
        lines: reader.lines(),
    }
}

const fn default_type() -> SmolStr {
    SmolStr::new_static("unknown")
}

const fn default_msgid() -> SmolStr {
    SmolStr::new_static("borrg.unknown")
}

fn deserialize_time<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<SystemTime>, D::Error> {
    use serde::{de::Error, Deserialize};

    if let Some(secs) = Option::<f64>::deserialize(deserializer)? {
        let dur = Duration::try_from_secs_f64(secs).map_err(D::Error::custom)?;
        let time = SystemTime::UNIX_EPOCH.checked_add(dur);
        return Ok(time);
    }

    Ok(None)
}

// {
//     "chunking_time": Number(0.0),
//     "files_stats": Object {
//         "U": Number(15229),
//         "d": Number(1276),
//         "s": Number(1648),
//     },
//     "finished": Bool(false),
//     "hashing_time": Number(0.12131854161270894),
//     "nfiles": Number(15229),
//     "original_size": Number(1709550361),
//     "path": String("home/satan/.local/share/Steam/ubuntu12_64/steam-runtime-heavy/usr/share/X11/locale/koi8-c"),
//     "time": Number(1744643951.9812548),
//     "type": String("archive_progress"),
// }
