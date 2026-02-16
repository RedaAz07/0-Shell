use chrono::{ DateTime, Duration, Local };
use std::cmp::max;
use std::os::unix::fs::{ FileTypeExt, MetadataExt, PermissionsExt };
use std::path::{ Path };
use std::time::SystemTime;
use std::{ fs };
use users::{ get_group_by_gid, get_user_by_uid };

#[derive(Debug, Clone, Copy, Default)]
pub struct Flag {
    pub a: bool,
    pub l: bool,
    pub f: bool, // (-F) indicators
}

#[derive(Clone)]
struct LongEntry {
    perms: String,
    links: String,
    user: String,
    group: String,
    size: String,
    date: String,
    name: String,
    blocks: u64,
}

pub fn ls(args: Vec<String>) {
    let (flag, files, dirs, errors) = parse_args(args);

    // default path "."
    let dirs = if files.is_empty() && dirs.is_empty() && errors.is_empty() {
        vec![".".to_string()]
    } else {
        dirs
    };

    // errors first
    for e in &errors {
        eprintln!("ls: cannot access '{}': No such file or directory", e);
    }
    if !errors.is_empty() && files.is_empty() && dirs.is_empty() {
        return;
    }

    // print files (before dirs)
    if !files.is_empty() {
        if flag.l {
            let mut ents = Vec::new();
            for p in &files {
                if let Ok(m) = fs::symlink_metadata(p) {
                    ents.push(prepare_long_entry(p.clone(), &m, flag, Path::new(p)));
                }
            }
            if !ents.is_empty() {
                print!("{}", align_and_format(ents, false));
            }
        } else {
            for p in &files {
                if let Ok(m) = fs::symlink_metadata(p) {
                    let out = if flag.f { decorate_name(p.clone(), &m) } else { p.clone() };
                    println!("{out}");
                }
            }
        }
    }
    //print directories
    let show_headers = !files.is_empty() || dirs.len() > 1 || !errors.is_empty();

    for (i, d) in dirs.iter().enumerate() {
        if i > 0 || !files.is_empty() {
            println!();
        }
        if show_headers {
            println!("{}:", d);
        }

        if flag.l {
            // long listing for directory
            match build_long_entries_for_dir(d, flag) {
                Ok(ents) => print!("{}", align_and_format(ents, true)),
                Err(_) => {
                    return;
                }
            }
        } else {
            // normal listing
            match read_dir_names(d, flag.a) {
                Ok(names) => {
                    if names.is_empty() {
                        continue;
                    }
                    if flag.f {
                        // add indicators (same behavior as -F)
                        let decorated = decorate_names_in_dir(d, names);
                        println!("{}", decorated.join(" "));
                    } else {
                        println!("{}", names.join(" "));
                    }
                }
                Err(e) => {
                    eprintln!("ls: cannot access '{}': {}", d, e);
                    return;
                }
            }
        }
    }
}

/* ---------------- parsing ---------------- */

fn parse_args(args: Vec<String>) -> (Flag, Vec<String>, Vec<String>, Vec<String>) {
    let mut flag = Flag::default();
    let mut files = Vec::new();
    let mut dirs = Vec::new();
    let mut errors = Vec::new();

    let mut after_double_dash = false;

    for arg in args {
        if arg == "--" {
            after_double_dash = true;
            continue;
        }

        if arg.starts_with('-') && !after_double_dash {
            if !parse_flag(&arg, &mut flag) {
                println!("ls: unrecognized option '{arg}'");
                return (flag, vec![], vec![], vec!["__STOP__".into()]);
            }
            continue;
        }

        let p = Path::new(&arg);
        if p.exists() || fs::symlink_metadata(p).is_ok() {
            if p.is_dir() && !p.is_symlink() {
                dirs.push(arg);
            } else {
                files.push(arg);
            }
        } else {
            errors.push(arg);
        }
    }

    // if we used __STOP__ trick, stop outside
    if errors.len() == 1 && errors[0] == "__STOP__" {
        return (flag, vec![], vec![], vec!["__STOP__".into()]);
    }

    (flag, files, dirs, errors)
}

fn parse_flag(arg: &str, flag: &mut Flag) -> bool {
    if arg.len() <= 1 {
        return false;
    }
    if !arg[1..].chars().all(|c| matches!(c, 'a' | 'l' | 'F')) {
        return false;
    }
    for c in arg[1..].chars() {
        match c {
            'a' => {
                flag.a = true;
            }
            'l' => {
                flag.l = true;
            }
            'F' => {
                flag.f = true;
            }
            _ => {}
        }
    }
    true
}

/* ---------------- directory reading ---------------- */

fn read_dir_names(dir: &str, show_hidden: bool) -> std::io::Result<Vec<String>> {
    let mut names = Vec::new();

    // -a adds "." and ".." (like your logic)
    if show_hidden {
        if fs::metadata(dir).is_ok() {
            names.push(".".to_string());
        }
        if fs::metadata(Path::new(dir).join("..")).is_ok() {
            names.push("..".to_string());
        }
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let s = name.to_string_lossy().to_string();
        if !show_hidden && s.starts_with('.') {
            continue;
        }
        names.push(s);
    }

    // keep your "special first" + case-insensitive-ish sort
    names.sort_by(|a, b| {
        let a_spec = a == "." || a == "..";
        let b_spec = b == "." || b == "..";
        if a_spec != b_spec {
            return if a_spec { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
        }
        if a_spec && b_spec {
            return a.cmp(b);
        }
        let ca = a.trim_start_matches('.').to_lowercase();
        let cb = b.trim_start_matches('.').to_lowercase();
        ca.cmp(&cb)
    });

    Ok(names)
}

fn decorate_names_in_dir(dir: &str, names: Vec<String>) -> Vec<String> {
    names
        .into_iter()
        .map(|name| {
            let full = Path::new(dir).join(&name);
            if let Ok(m) = fs::symlink_metadata(&full) {
                decorate_name(name, &m)
            } else {
                name
            }
        })
        .collect()
}

fn build_long_entries_for_dir(dir: &str, flag: Flag) -> Result<Vec<LongEntry>, ()> {
    let mut ents = Vec::new();
    // if -a: add "." and ".."
    if flag.a {
        if let Ok(m) = fs::metadata(dir) {
            ents.push(prepare_long_entry(".".to_string(), &m, flag, Path::new(dir)));
        }
        let parent = Path::new(dir).join("..");
        if let Ok(m) = fs::metadata(&parent) {
            ents.push(prepare_long_entry("..".to_string(), &m, flag, &parent));
        }
    }

    let rd = fs::read_dir(dir).map_err(|_| ())?;
    let mut items: Vec<_> = rd.filter_map(Result::ok).collect();
  
    // keep your "clean alphanumeric" sort
    items.sort_by(|a, b| {
        let na = a.file_name().to_string_lossy().to_string();
        let nb = b.file_name().to_string_lossy().to_string();
        let clean = |s: &str|
            s
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
        let ka = clean(&na);
        let kb = clean(&nb);
        let ord = ka.cmp(&kb);
        if ord == std::cmp::Ordering::Equal {
            na.cmp(&nb)
        } else {
            ord
        }
    });

    for e in items {
        let name = e.file_name().to_string_lossy().to_string();
        if !flag.a && name.starts_with('.') {
            continue;
        }
        if let Ok(m) = e.metadata() {
            ents.push(prepare_long_entry(name, &m, flag, &e.path()));
        }
    }

    Ok(ents)
}

/* ---------------- formatting ---------------- */

fn decorate_name(mut name: String, m: &fs::Metadata) -> String {
    let ft = m.file_type();
    if m.is_dir() {
        name.push('/');
    } else if ft.is_symlink() {
        name.push('@');
    } else if ft.is_fifo() {
        name.push('|');
    } else if ft.is_socket() {
        name.push('=');
    } else if (m.permissions().mode() & 0o111) != 0 {
        name.push('*');
    }
    name
}

fn format_permissions(metadata: &fs::Metadata, file_path: &Path) -> String {
    let mode = metadata.permissions().mode();
    let mut s = String::with_capacity(11);

    if metadata.is_dir() {
        s.push('d');
    } else if metadata.is_symlink() {
        s.push('l');
    } else if metadata.file_type().is_char_device() {
        s.push('c');
    } else if metadata.file_type().is_block_device() {
        s.push('b');
    } else if metadata.file_type().is_fifo() {
        s.push('p');
    } else if metadata.file_type().is_socket() {
        s.push('s');
    } else {
        s.push('-');
    }

    // user
    s.push(if (mode & 0o400) != 0 { 'r' } else { '-' });
    s.push(if (mode & 0o200) != 0 { 'w' } else { '-' });
    s.push(if (mode & 0o100) != 0 { 'x' } else { '-' });

    // group
    s.push(if (mode & 0o040) != 0 { 'r' } else { '-' });
    s.push(if (mode & 0o020) != 0 { 'w' } else { '-' });
    s.push(if (mode & 0o010) != 0 { 'x' } else { '-' });

    // other
    s.push(if (mode & 0o004) != 0 { 'r' } else { '-' });
    s.push(if (mode & 0o002) != 0 { 'w' } else { '-' });
    s.push(if (mode & 0o001) != 0 { 'x' } else { '-' });

    let has_xattr = xattr
        ::list(file_path)
        .map(|mut i| i.next().is_some())
        .unwrap_or(false);

    s.push(if has_xattr { '+' } else { ' ' });
    s
}

fn format_date(modified: SystemTime) -> String {
    let now = SystemTime::now();
    let datetime: DateTime<Local> = modified.into();
    let datetime = datetime + Duration::hours(1);

    let six_months = std::time::Duration::from_secs(180 * 24 * 60 * 60);
    let is_old_or_future = match now.duration_since(modified) {
        Ok(d) => d > six_months,
        Err(_) => true,
    };

    if is_old_or_future {
        datetime.format("%b %d  %Y").to_string()
    } else {
        datetime.format("%b %d %H:%M").to_string()
    }
}

fn align_and_format(entries: Vec<LongEntry>, show_total: bool) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut w_links = 0;
    let mut w_user = 0;
    let mut w_group = 0;
    let mut w_size = 0;
    let mut w_date = 0;
    let mut total_blocks = 0;

    for e in &entries {
        w_links = max(w_links, e.links.len());
        w_user = max(w_user, e.user.len());
        w_group = max(w_group, e.group.len());
        w_size = max(w_size, e.size.len());
        w_date = max(w_date, e.date.len());
        total_blocks += e.blocks;
    }

    let mut out = String::new();
    if show_total {
        out.push_str(&format!("total {}\n", total_blocks / 2));
    }

    for e in entries {
        out.push_str(
            &format!(
                "{} {:>lw$} {:<uw$} {:<gw$} {:>sw$} {:>dw$} {}\n",
                e.perms,
                e.links,
                e.user,
                e.group,
                e.size,
                e.date,
                e.name,
                lw = w_links,
                uw = w_user,
                gw = w_group,
                sw = w_size,
                dw = w_date
            )
        );
    }

    out
}

/* ---------------- long entry builder ---------------- */

fn prepare_long_entry(
    mut name: String,
    m: &fs::Metadata,
    flag: Flag,
    full_path: &Path
) -> LongEntry {
    // -F on the file itself (but not for symlink here, same as your logic)
    if flag.f && !m.is_symlink() {
        name = decorate_name(name, m);
    }

    // symlink arrow target
    if m.file_type().is_symlink() {
        if let Ok(target) = fs::read_link(full_path) {
            let mut target_str: String = target.to_string_lossy().to_string();

            // if -F: decorate target based on resolved metadata
            if flag.f {
                let resolved = if target.is_absolute() {
                    target.clone()
                } else {
                    full_path.parent().unwrap_or(Path::new(".")).join(&target)
                };
                if let Ok(tm) = fs::metadata(&resolved) {
                    target_str = decorate_name(target_str, &tm);
                }
            }

            name.push_str(" -> ");
            name.push_str(&target_str);
        }
    }

    let perms = format_permissions(m, full_path);
    let links = m.nlink().to_string();

    let uid = m.uid();
    let user = get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().to_string())
        .unwrap_or_else(|| uid.to_string());

    let gid = m.gid();
    let group = get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().to_string())
        .unwrap_or_else(|| gid.to_string());

    let size = if m.file_type().is_block_device() || m.file_type().is_char_device() {
        let dev = m.rdev() as libc::dev_t;
        let (maj, min) = (libc::major(dev), libc::minor(dev));
        format!("{:>3}, {:>3}", maj, min)
    } else {
        m.len().to_string()
    };

    let date = format_date(m.modified().unwrap_or(SystemTime::now()));

    LongEntry {
        perms,
        links,
        user,
        group,
        size,
        date,
        name,
        blocks: m.blocks(),
    }
}
