use std::env;
use std::io::{ self, stdout, Write };

pub mod commands;
pub mod helpers;

use commands::pwd_state::*;
use crossterm::cursor::MoveToColumn;
use crossterm::event::{ self, Event, KeyCode, KeyEventKind, KeyModifiers };
use crossterm::execute;
use crossterm::terminal::{ disable_raw_mode, enable_raw_mode, Clear, ClearType };
use helpers::parser::{ parse_input, ParseResult };
use helpers::print_banner::print_banner;

use crate::helpers::executor::execute;

const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

fn main() -> io::Result<()> {
    print_banner();
    enable_raw_mode()?;

    let mut input_buffer = String::new();
    let mut is_continuation = false;
    let mut input_perln = String::new();

    let start_dir = env::current_dir().expect("Failed to get current working directory");
    let mut pwd_state = PwdState::new(
        start_dir.display().to_string(),
        start_dir.display().to_string()
    );

    loop {
        let current_display_dir = pwd_state.get_current_dir().replace("\n", "\\n");

        let prompt_len = if is_continuation { 2 } else { current_display_dir.len() + 2 };

        execute!(stdout(), MoveToColumn(0), Clear(ClearType::CurrentLine))?;

        let prompt_text = if !is_continuation {
            format!("{GREEN}{}$ {RESET}", current_display_dir)
        } else {
            "> ".to_string()
        };

        print!("{}", prompt_text);
        io::stdout().flush()?;

        loop {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char(c) => {
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) && c == 'd' {
                                print!("\r\n");
                                disable_raw_mode()?;
                                return Ok(());
                            } else if
                                // Handle Ctrl+C (Cancel)
                                key_event.modifiers.contains(KeyModifiers::CONTROL) &&
                                c == 'c'
                            {
                                print!("\r\n");
                                input_buffer.clear();
                                input_perln.clear();
                                is_continuation = false;
                                break;
                            }

                            // CLEAN LOGIC: Just push to end and print char
                            input_perln.push(c);
                            input_buffer.push(c);
                            print!("{}", c);
                            io::stdout().flush()?;
                        }

                        KeyCode::Backspace => {
                            if !input_perln.is_empty() {
                                // CLEAN LOGIC: Just pop the last char
                                input_buffer.pop();
                                input_perln.pop();
                                // Redraw the line to visually remove the character
                                execute!(
                                    stdout(),
                                    MoveToColumn(prompt_len as u16),
                                    Clear(ClearType::UntilNewLine)
                                )?;
                                print!("{}", input_perln);

                                io::stdout().flush()?;
                            }
                        }

                        KeyCode::Enter => {
                            print!("\r\n");
                            io::stdout().flush()?;
                            input_perln.clear();
                            match parse_input(&input_buffer) {
                                ParseResult::Ok(cmds) => {
                                    if cmds.is_empty() {
                                        input_buffer.clear();
                                        is_continuation = false;
                                        break;
                                    }
                                    disable_raw_mode()?;
                                    execute(cmds[0].clone(), &mut pwd_state);
                                    enable_raw_mode()?;

                                    input_buffer.clear();
                                    is_continuation = false;
                                    break;
                                }
                                ParseResult::Incomplete => {
                                    input_buffer.push('\n');
                                    is_continuation = true;
                                    break;
                                }
                                ParseResult::Err(e) => {
                                    println!("Error: {}\r", e);
                                    input_buffer.clear();
                                    is_continuation = false;
                                    break;
                                }
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }
}
