use std::{ collections::HashSet, ffi::OsString, fs, path::Path };

pub fn cp(args: Vec<String>) {
    if args.is_empty() {
        eprintln!("cp: missing file operand");
        return;
    }
    if args.len() < 2 {
        eprintln!("cp: missing destination file operand after '{}'", args[0]);
        return;
    }

    let sources = &args[0..args.len() - 1];
    let last_arg = args.last().unwrap();
    let destination_path: &Path = Path::new(last_arg);

    if args.len() > 2 {
        if !destination_path.is_dir() {
            eprintln!("cp: target '{}' is not a directory", destination_path.display());
            return;
        }

        // 2. Track Destination filenames (to avoid "cp a ../a dir")
        let mut dest_seen: HashSet<OsString> = HashSet::new();

        for source in sources {
            let source_path = Path::new(source);

            // Check if this filename has already been copied to the destination in this run
            if let Some(file_name) = source_path.file_name() {
                if !dest_seen.insert(file_name.to_os_string()) {
                    eprintln!(
                        "cp: warning: cannot copy '{}' to '{}': destination file already used by another argument",
                        source,
                        destination_path.join(file_name).display()
                    );
                    continue;
                }
            }

            copy_file_logic(source_path, destination_path, true);
        }
    } else {
        let source_path = Path::new(&args[0]);
        copy_file_logic(source_path, destination_path, destination_path.is_dir());
    }
}

fn copy_file_logic(source: &Path, destination: &Path, dest_is_dir: bool) {
    if !source.exists() {
        eprintln!("cp: cannot stat '{}': No such file or directory", source.display());
        return;
    }
    if source.is_dir() {
        eprintln!("cp: -r not specified; omitting directory '{}'", source.display());
        return;
    }

    let final_dest = if dest_is_dir {
        let file_name = source.file_name().unwrap();
        destination.join(file_name)
    } else {
        destination.to_path_buf()
    };

    if final_dest.exists() {
        if let (Ok(src_can), Ok(dst_can)) = (source.canonicalize(), final_dest.canonicalize()) {
            if src_can == dst_can {
                eprintln!(
                    "cp: '{}' and '{}' are the same file",
                    source.display(),
                    final_dest.display()
                );
                return;
            }
        }
    }

    if let Err(e) = fs::copy(source, &final_dest) {
        eprintln!("cp: error copying to '{}': {}", final_dest.display(), e);
    }
}
