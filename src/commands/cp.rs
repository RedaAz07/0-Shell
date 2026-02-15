use std::{fs, path::Path};

pub fn cp(args: Vec<String>)  {
    if args.is_empty() {
        eprintln!("cp: missing file operand");
        return ;
    } else if args.len() == 1 {
        eprintln!("cp: missing destination file operand after '{}'", args[0]);
        return ;
    }

    let source_path = Path::new(&args[0]);
    let destination_path = Path::new(&args[1]);

    if !source_path.is_file() {
        eprintln!(
            "cp: cannot stat '{}': No such file or directory",
            source_path.display()
        );
        return ;
    } else if source_path.is_file() && destination_path.is_dir() {
        let file_name = match source_path.file_name() {
            Some(name) => name,
            None => {
                eprintln!("cp: error getting file name from source path");
                return ;
            }
        };
        let mut dest_file_path = destination_path.to_path_buf();
        dest_file_path.push(file_name);

        let res = fs::copy(source_path, dest_file_path);

        match res {
            Ok(_) => {
                return ;
            }
            Err(e) => {
                eprintln!("cp: {}", e);
                return ;
            }
        }
    } else {
        let res = fs::copy(source_path, destination_path);
        match res {
            Ok(_) => return ,
            Err(e) => {
                eprintln!("cp: {}", e);
                return ;
            }
        }
    }
}
