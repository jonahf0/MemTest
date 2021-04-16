use std::collections::HashMap;
use std::process::{exit, Command};
use std::sync::mpsc::channel;
use std::{fs, thread, time};

use clap::{App, Arg};

mod print_results;

use print_results::*;


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

        //I used for_each() just for fun
        temp.iter().for_each( |line| {

            /*The key is the initial index of the line, which should actually be a key.
            In order to account for lines that might have an unexpected format (like of
            I try reading from "status" instead of "smaps_rollup"), the default "empty"
            is used.*/
            return_hash_map.insert(
                line.get(0).unwrap_or(&"empty".to_string()).to_string(),
                format!("{} {}", line.get(1).unwrap_or(&"empty".to_string()), line.get(2).unwrap_or(&"empty".to_string())),
                );

        });
    
    return Some(return_hash_map);
}

fn run_process(executable: &str, exec_args: Vec<&str>, interval: (u64, &str)) {

    let interval_val = if interval.1 == "ms" { interval.0 }
    
    else {
        interval.0 / 1000
    } ;

    let interval_unit = interval.1;

    let mut mem_info_list: Vec<HashMap<String,String>> = Vec::new();

     //run the process; if there is an error, then print the error for user's benefit
     let mut proc_child = match Command::new(executable).args(exec_args).spawn() {
        Ok(child) => child,

        Err(err) => {
            println!("There was an error: {}", err);

            exit(0);
        }
    };

    let pid = proc_child.id();

    //channel for letting the main thread know if the child thread is finished waiting
    let (thread_tx, loop_rx) = channel();

    let waiting_thread = thread::spawn(move || match proc_child.wait() {
        Ok(_) => thread_tx.send(true).unwrap(),

        Err(err) => {
            println!("There was an error with the process: {}", err);
            thread_tx.send(true).unwrap();
        }
    });

    //waits for the child thread to finish; recv_timeout() will return an Err() if it timesout
    while if let Ok(_) = loop_rx.recv_timeout(time::Duration::from_millis(interval_val)) {
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

    print_smaps_result(mem_info_list, interval_unit);

    match waiting_thread.join() {

        Ok(_) => {},

        Err(err) => println!("{:?}", err)

    }

}

fn main() {
    
    //use the clap crate to parse the arguments
    let matches = App::new("mem_test").about("\nDesigned to act as a simple command-line wrapper to test\nmemory usage of a program during its lifetime (think like the time utility)")
    .arg(Arg::from_usage("--interval [NUMBER] [UNIT] 'optional interval (in milliseconds [ms] or microseconds [us])\nto check memory usage; default is 1 millisecond'"))
    .arg(Arg::with_name("executable")
        .help("The executable to use")
        .required(true))
    .arg(Arg::with_name("executable args")
        .help("The arguments to pass to the executable")
        .multiple(true)
        .required(true))
    .get_matches();

    /*just a variable for keeping track of how many ms to wait before reading smaps;
    May be an optional cmd arg in the future*/
    let interval: (u64, &str) = match matches.values_of("interval") {

        Some(vals) => {

            let temp_vec = vals.collect::<Vec<&str>>();

            let interval_val = match temp_vec[0].parse::<u64>() {

                Ok(val) => val,

                Err(err) => { 
                    println!("There was a problem understanding the interval:\n{}", err);
                    exit(0)    
                }   
            };
            
            
            let interval_unit = match temp_vec[1] {

                "ms" => "ms",

                "us" => "us",

                _ => {

                    println!("Did not understand the unit provided for interval");
                    exit(0)

                }

            };

            (interval_val, interval_unit)

        }

        None => (1,"ms")

    };

    let executable = match matches.value_of("executable") {

        Some(val) => val,

        None => {

            println!("No executable provided!");

            exit(0);

        }

    };

    let exec_args = match matches.values_of("executable args") {

        Some(vals) => vals.collect::<Vec<&str>>().clone(),

        None => {
            let empty_vec: Vec<&str> = Vec::new();

            empty_vec
        }
    };

    run_process(executable, exec_args, interval);

}
