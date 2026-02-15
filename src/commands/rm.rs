use std::path::Path;

pub fn rm(args: Vec<String>) {
    let mut recursive = false;

    for arg in &args {
        if arg == "--recursive" {
            recursive = true;
            continue;
        }
        if arg.starts_with("-") {
            for c in arg[1..].chars() {
                if c != '-' && c != 'r' && c != 'R' {
                    eprintln!("rm: invalid option -- '{}'", c);
                    return;
                }
            }

            recursive = true;
        }
    }

    let targets: Vec<&String> = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .collect();

    if targets.is_empty() {
        eprintln!("rm: missing operand");
        return;
    }

    for arg in targets {
        let path = Path::new(arg);
        if
            matches!(
                path.file_name().and_then(|n| n.to_str()),
                Some(".") | Some("..")
            )
        {
            eprintln!("rm: refusing to remove '.' or '..' directory: skipping '..'");
            continue;
        }

        match std::fs::symlink_metadata(path) {
            Ok(meta) => {
                if meta.is_symlink() {
                    if let Err(e) = std::fs::remove_file(path) {
                        eprintln!(
                            "rm: cannot remove symlink '{}': {}",
                            arg.replace("\n", "\\n"),
                            e
                        );
                    }
                } else if meta.is_dir() {
                    if !recursive {
                        eprintln!(
                            "rm: cannot remove '{}': Is a directory",
                            arg.replace("\n", "\\n")
                        );
                    } else {
                        if let Err(e) = std::fs::remove_dir_all(path) {
                            eprintln!("rm: cannot remove '{}': {}", arg.replace("\n", "\\n"), e);
                        }
                    }
                } else {
                    if let Err(e) = std::fs::remove_file(path) {
                        eprintln!("rm: cannot remove '{}': {}", arg.replace("\n", "\\n"), e);
                    }
                }
            }
            Err(e) => eprintln!("rm: cannot remove '{}': {}", arg.replace("\n", "\\n"), e),
        }
    }
}
