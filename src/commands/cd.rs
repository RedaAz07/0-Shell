use crate::commands::pwd_state::PwdState;
use std::{ env, path::PathBuf };

pub fn command_cd( mut args: Vec<String>, pwd_state: &mut PwdState) {
    if args.len() > 1 {
        eprintln!("cd: too many arguments");
        return;
    }
    if args.len() == 1 {
        args[0] = args[0].replace("\\n", "\n");
    }
    let target_dir = if args.is_empty() {
        match env::var("HOME") {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                eprintln!("cd: HOME environment variable not set");
                return;
            }
        }
    } else if args[0] == "-" {
        PathBuf::from(pwd_state.get_old_dir())
    } else if args[0] == "~" {
        match env::var("HOME") {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                eprintln!("cd: HOME environment variable not set");
                return;
            }
        }
    } else {
        PathBuf::from(&args[0])
    };

    let current_before_move = pwd_state.get_current_dir();

    match env::set_current_dir(&target_dir) {
        Ok(_) => {
            if let Ok(new_current) = env::current_dir() {
                pwd_state.set_states(new_current.display().to_string(), current_before_move);

            } else {
                pwd_state.set_states(PathBuf::from(".").display().to_string(), current_before_move);
            }
            return;
        }
        Err(e) => eprintln!("cd: {}: {}", args[0].replace("\n", "\\n"), e),
    }
}
