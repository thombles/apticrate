use std::collections::HashMap;
use std::env;
use std::process::Command;

fn main() {
    let search_term = env::args().skip(1).next();

    let mut outputs = query_available(search_term.as_deref());
    mark_installed(&mut outputs);

    let mut max_title_width = 0;
    let rows: Vec<_> = outputs
        .into_iter()
        .map(|o| {
            let installed_str = if o.installed { "installed" } else { "--" };
            let title = o.title();
            if title.len() > max_title_width {
                max_title_width = title.len();
            }
            (title, installed_str, o.apt_pkg)
        })
        .collect();

    for (title, installed, pkg) in rows {
        println!(
            "{:<width$}  {:<9}  {}",
            title,
            installed,
            pkg,
            width = max_title_width
        )
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Semver {
    major: u32,
    minor: u32,
    patch: u32,
}

impl std::fmt::Display for Semver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(PartialEq, Eq)]
struct OutputItem {
    crate_name: String,
    metapkg_for_feature: Option<String>,
    version: Semver,
    apt_pkg: String,
    installed: bool,
}

impl OutputItem {
    fn title(&self) -> String {
        if let Some(feat) = &self.metapkg_for_feature {
            format!("  deps for feat \"{}\"", feat)
        } else {
            format!("{} {}", self.crate_name, self.version)
        }
    }
}

impl PartialOrd for OutputItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OutputItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // ascending crate name
        if self.crate_name != other.crate_name {
            return self
                .crate_name
                .to_lowercase()
                .cmp(&other.crate_name.to_lowercase());
        }
        // ascending versions
        if self.version != other.version {
            return self.version.cmp(&other.version);
        }
        // within a version, "root" crate first
        match (
            self.metapkg_for_feature.is_some(),
            other.metapkg_for_feature.is_some(),
        ) {
            (true, false) => return std::cmp::Ordering::Greater,
            (false, true) => return std::cmp::Ordering::Less,
            _ => (),
        }
        // within feature crates, ascending feature name
        if let (Some(f1), Some(f2)) = (&self.metapkg_for_feature, &other.metapkg_for_feature) {
            return f1.cmp(&f2);
        }
        std::cmp::Ordering::Equal
    }
}

fn query_available(search_term: Option<&str>) -> Vec<OutputItem> {
    let mut out = vec![];
    let cmd_output = Command::new("apt-cache")
        .args(["show", "librust-*"])
        .output()
        .expect("failed to run 'apt-cache'")
        .stdout;
    let cmd_output = String::from_utf8_lossy(&cmd_output);
    let mappings = static_mappings();
    for summary in cmd_output.split("Package: ") {
        let summary = summary.trim();
        if summary.is_empty() {
            continue;
        }
        if let Ok(item) = parse_package(&summary, &mappings) {
            if let Some(term) = search_term {
                if !item.crate_name.contains(term) {
                    continue;
                }
            }
            out.push(item);
        }
    }
    out.sort();
    out
}

fn parse_package(
    summary: &str,
    mappings: &HashMap<&'static str, &'static str>,
) -> Result<OutputItem, ()> {
    let apt_pkg = summary
        .split(|c: char| c.is_whitespace())
        .next()
        .ok_or(())?;
    let version_str = summary
        .split("Version: ")
        .skip(1)
        .next()
        .ok_or(())?
        .split(|c: char| !c.is_digit(10) && c != '.')
        .next()
        .ok_or(())?;
    let mut components = version_str.split(".");
    let version = Semver {
        major: components.next().ok_or(())?.parse().map_err(drop)?,
        minor: components.next().ok_or(())?.parse().map_err(drop)?,
        patch: components.next().ok_or(())?.parse().map_err(drop)?,
    };
    let joined = summary.replace("-\n ", "-");
    let joined = joined.replace("\n", "");
    let (crate_name, maybe_feature) = if joined.contains("contains the source for the Rust") {
        (
            joined
                .split("contains the source for the Rust ")
                .skip(1)
                .next()
                .ok_or(())?
                .split(" ")
                .next()
                .ok_or(())?,
            None,
        )
    } else if joined.contains("code for Debianized Rust crate") {
        (
            joined
                .split("code for Debianized Rust crate \"")
                .skip(1)
                .next()
                .ok_or(())?
                .split("\"")
                .next()
                .ok_or(())?,
            None,
        )
    } else if joined.contains("for the Rust") {
        (
            joined
                .split("for the Rust ")
                .skip(1)
                .next()
                .ok_or(())?
                .trim()
                .split(" ")
                .next()
                .ok_or(())?,
            Some(
                joined
                    .split("- feature \"")
                    .skip(1)
                    .next()
                    .ok_or(())?
                    .split("\"")
                    .next()
                    .ok_or(())?
                    .to_owned(),
            ),
        )
    } else if joined.contains("Rust crate") {
        (
            joined
                .split("Rust crate ")
                .skip(1)
                .next()
                .ok_or(())?
                .split(" ")
                .next()
                .ok_or(())?,
            None,
        )
    } else {
        if let Some(crate_name) = mappings.get(apt_pkg) {
            (crate_name.to_owned(), None)
        } else {
            return Err(());
        }
    };
    Ok(OutputItem {
        crate_name: crate_name.to_owned(),
        metapkg_for_feature: maybe_feature,
        version,
        apt_pkg: apt_pkg.to_owned(),
        installed: false,
    })
}

fn mark_installed(outputs: &mut Vec<OutputItem>) {
    let cmd_output = Command::new("dpkg")
        .args(["-l"])
        .output()
        .expect("failed to run 'dpkg'")
        .stdout;
    let cmd_output = String::from_utf8_lossy(&cmd_output);
    for l in cmd_output.lines() {
        if !l.contains("librust-") {
            continue;
        }
        for o in outputs.iter_mut() {
            if l.contains(&o.apt_pkg) {
                if l.starts_with("ii") {
                    o.installed = true;
                }
            }
        }
    }
}

/// Returns a short list of static mappings from package names to crate names.
///
/// Despite best efforts to match on regular patterns in the package descriptions,
/// some crates don't follow any pattern or fail to mention the real crate name at
/// all. Hopefully these will become more regular over time.
fn static_mappings() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("librust-aho-corasick-dev", "aho-corasick");
    m.insert("librust-capstone-dev", "capstone");
    m.insert("librust-darling-core-0.14-dev", "darling_core");
    m.insert("librust-darling-core-dev", "darling_core");
    m.insert("librust-darling-macro-dev", "darling_macro");
    m.insert("librust-darling-macro-0.14-dev", "darling_macro");
    m.insert("librust-notify-debouncer-mini-dev", "notify-debouncer-mini");
    m.insert("librust-zstd-sys-dev", "zstd-sys");
    m
}
