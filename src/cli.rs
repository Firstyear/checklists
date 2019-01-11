extern crate getopts;

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use getopts::Options;
use std::env;

use std::fs::File;
use std::io::prelude::*;
use std::io;

use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
enum CheckState {
    UNCHECKED,
    CHECKED,
    SKIP,
}

impl fmt::Display for CheckState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            CheckState::UNCHECKED => {
                write!(f, "[ ] (Unchecked)")
            }
            CheckState::CHECKED => {
                write!(f, "[x] (Checked)")
            }
            CheckState::SKIP => {
                write!(f, "[-] (skipped)")
            }
        };
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ChecklistItem {
    name: String,
    desc: String,
    status: CheckState,
    comment: Option<String>,
}

impl fmt::Display for ChecklistItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name: {}\n", self.name);
        write!(f, "description: {}\n", self.desc);
        write!(f, "status: {}\n", self.status);
        match &self.comment {
            Some(c) => {
                write!(f, "comment: {}\n", c)
            }
            None => {
                write!(f, "comment: -\n")
            }
        };
        Ok(())
    }
}

impl ChecklistItem {
    pub fn set(&mut self, state: CheckState) {
        self.status = state;
    }

    pub fn set_comment(&mut self, comment: String) {
        self.comment = Some(comment);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Checklist {
    name: String,
    list: Vec<ChecklistItem>,
}

impl fmt::Display for Checklist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "checklist name: {}\n\n", self.name);
        for i in &self.list {
            write!(f, "\tname: {} -> {}\n", i.name, i.status);
        };
        Ok(())
    }
}

fn get_input() -> String {
    let mut data = String::new();
    match io::stdin().read_line(&mut data) {
        Ok(_) => {}
        Err(e) => { panic!(e) }
    };
    // Now strip the trailing new lines.
    let s = data.trim();
    String::from(s)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // Parse cli options
    let mut opts = Options::new();
    opts.optopt("c", "checklists", "Directory location of checklists", "CHECKLISTS");
    opts.optflag("v", "verbose", "Add extra verbose messages");
    opts.optflag("e", "example", "Create an example checklist into checklists directory");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("Error: {}", f.to_string());
            return;
        }
    };

    let verbose = if matches.opt_present("v") {
        true
    } else {
        false
    };

    let example = if matches.opt_present("e") {
        true
    } else {
        false
    };

    if example {
        let checklist_path = match matches.opt_str("c") {
            Some(p) => { p },
            None => {
                println!("-c is required with -e");
                return;
            }
        };

        // Create the example in checklist_path/example.list.json
        println!("Creating example checklist in {}/example.list.json", checklist_path);

        // Create same items
        let item1 = ChecklistItem {
            name: String::from("item1"),
            desc: String::from("item 1 description"),
            status: CheckState::UNCHECKED,
            comment: None,
        };
        let item2 = ChecklistItem {
            name: String::from("item2"),
            desc: String::from("item 2 description"),
            status: CheckState::UNCHECKED,
            comment: None,
        };
        // Put them in a list
        let list = Checklist {
            name: String::from("list 1"),
            list: vec![item1, item2],
        };
        // Serialise it out.
        let d = serde_json::to_string_pretty(&list).unwrap();
        println!("{}", d);

        let mut f = File::create(format!("{}/example.list.json", checklist_path)).unwrap();
        f.write_all(d.as_bytes());
    } else {
        // Check if we have a name
        let name = if !matches.free.is_empty() {
            matches.free[0].clone()
        } else {
            println!("Require a checklist path to work on");
            return;
        };

        // Read in a checklist of "name". If it already exists in data, read that.
        let mut contents = String::new();
        {
            let mut file = File::open(name.clone()).unwrap();
            file.read_to_string(&mut contents).unwrap();
        }
        let mut checklist: Checklist = serde_json::from_str(contents.as_str()).unwrap();

        let mut step: usize = 0;

        {
            println!("checklist: {}", checklist.name);
            let e = checklist.list.get(step).unwrap();
            println!("{}", e);
        }

        // Now we have to enter a main loop
        loop {
            println!("# ");
            let s = get_input();

            match s.as_str() {
                "exit" => { return }
                "save" => {
                    // Save check list state
                    println!("Saving to {} ...", name);
                    let d = serde_json::to_string_pretty(&checklist).unwrap();
                    let mut f = File::create(name.clone()).unwrap();
                    f.write_all(d.as_bytes());
                }
                "p" => {
                    let e = checklist.list.get(step).unwrap();
                    println!("{}", e);
                }
                "next" => {
                    if (step + 1) >= checklist.list.len() {
                        println!("End of list");
                    } else {
                        step = step + 1;
                        let e = checklist.list.get(step).unwrap();
                        println!("{}", e);
                    }
                }
                "back" => {
                    if step == 0 {
                        println!("Start of list");
                    } else {
                        step = step - 1;
                        let e = checklist.list.get(step).unwrap();
                        println!("{}", e);
                    }
                }
                "mark" => {
                    if let Some(e) = checklist.list.get_mut(step) {
                        e.set(CheckState::CHECKED);
                    }
                }
                "skip" => {
                    // Require a comment
                    if let Some(e) = checklist.list.get_mut(step) {
                        e.set(CheckState::SKIP);
                        println!("Comment: ");
                        e.set_comment(get_input());
                        println!("End comment -- ");
                    }
                }
                "unmark" => {
                    if let Some(e) = checklist.list.get_mut(step) {
                        e.set(CheckState::UNCHECKED);
                    }
                }
                "comment" => {
                    if let Some(e) = checklist.list.get_mut(step) {
                        println!("Comment: ");
                        e.set_comment(get_input());
                        println!("End comment -- ");
                    }
                }
                "l" => {
                    // Display all checklist elements and indexes
                    println!("{}", checklist);
                }
                _ => {
                    println!("Unknown command")
                }
            }

        }
    };
}

