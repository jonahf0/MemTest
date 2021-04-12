use std::collections::HashMap;
use std::process::{exit, Command};
use std::sync::mpsc::channel;
use std::{env, fs, thread, time};

//just a utility for printing the results
fn print_result(list: Vec<HashMap<String, String>>) {

    let mut previous_data = "".to_string();

    for (index, map) in list.iter().enumerate() {
            
            let current_data = format!("Resident set size: {}\nProportional set size: {}\n",
                                                map["Rss"],
                                                map["Pss"]);

            match previous_data == current_data {

                true => {},
                
                false => {
                    println!("[{} ms]:", index*100);
                    println!("{}", current_data);

                    previous_data = current_data;
                }
                
            }
    }
}

/*reads the /proc files, the first arg is the PID of the process that gets spawned,
and the second argument is the specific file to read from; currently, it can just accept
None in order to default to "smaps_rollup"*/
fn read_from_proc(pid: u32) -> Option<HashMap<String, String>> {

    let proc_file = "smaps_rollup";

    let file_location = format!("/proc/{}/{}", pid.to_string(), proc_file);

    /*used for constructing a map of every line in the file;
    they all have predictable formats like "Rss: <number> kb"*/
    let mut return_hash_map: HashMap<String, String> = HashMap::new();

    //main reason for returning an Option -- may be an issue reading from the file
    let file_contents = match fs::read_to_string(file_location) {
        Ok(value) => value,

        Err(_) => return None,
    };

    /*Pretty massive bunch of functions [how I like it >:)] --
    split the file contents by line, then split by white space into a vector, then
    remove all ':' chars*/ 
    let temp = file_contents
        .split("\n")
        .filter(|&line| line != "")
        .map(|line| -> Vec<String> {
            line.split_whitespace()
                .collect::<Vec<&str>>()
                .iter()
                .map(|line| line.replace(":", ""))
                .collect()
        })
        .collect::<Vec<Vec<String>>>(); 

        /*
        let key = match line.get(0) {

            Some(string) => string.to_string(),

            None => "nothing relevant".to_string()

        }

        let data = match line.get(1) {

            Some(string) => string.to_string(),

            None => 

        } */

        temp.iter().for_each( |line| {

            return_hash_map.insert(
                line.get(0).unwrap_or(&"empty".to_string()).to_string(),
                format!("{} {}", line.get(1).unwrap_or(&"empty".to_string()), line.get(2).unwrap_or(&"empty".to_string())),
                );

        });
        
    return Some(return_hash_map);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let interval = 100;
    
    let mut mem_info_list: Vec<HashMap<String,String>> = Vec::new();

    let executable = match args.get(1) {
        Some(val) => val,

        None => {
            println!("No executable was provided!");

            exit(0)
        }
    };

    let exec_args = match args.get(2..) {
        Some(vals) => vals,

        None => &[],
    };

    let mut proc_child = match Command::new(executable).args(exec_args).spawn() {
        Ok(child) => child,

        Err(err) => {
            println!("There was an error: {}", err);

            exit(0);
        }
    };

    let pid = proc_child.id();

    let (thread_tx, loop_rx) = channel();

    let waiting_thread = thread::spawn(move || match proc_child.wait() {
        Ok(_) => thread_tx.send(true).unwrap(),

        Err(err) => {
            println!("There was an error with the process: {}", err);
            thread_tx.send(true).unwrap();
        }
    });

    while if let Ok(_) = loop_rx.recv_timeout(time::Duration::from_millis(interval)) {
        false
    } else {
        true
    } {

        let mem_info = read_from_proc(pid);

        match mem_info {
            Some(map) => mem_info_list.push(map),

            None => {}
        };
    }

    print_result(mem_info_list);

    waiting_thread.join();
}
