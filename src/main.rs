use std::collections::{BTreeMap, HashMap};
use std::process::{exit, Command};
use std::sync::mpsc::channel;
use std::{thread, time};
use clap::{App, Arg};

mod print_results;
mod read_proc;

use print_results::*;
use read_proc::*;

//runs the process for taking measurements
fn run_process(executable: &str, exec_args: Vec<&str>, interval: (u64, &str)) {

    let interval_val = if interval.1 == "us" { interval.0 / 1000 }
    
    else {
        interval.0 
    } ;

    let interval_unit = interval.1;

    let mut mem_info_list: BTreeMap<u64, HashMap<String,String>> = BTreeMap::new();

    let mut time_variable: u64 = 0;

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
    loop {

        let mem_info = read_smaps_rollup(pid);

        match mem_info {

            None => {},

            Some(map) => 
                {
                    mem_info_list.insert(time_variable ,map);
                }
        }

        time_variable += interval_val;

        if let Ok(_) = loop_rx.recv_timeout(time::Duration::from_millis(interval_val)) {
            break
        }
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
        .required(false))
    .get_matches();

    /*just a variable for keeping track of how many ms to wait before reading smaps;
    May be an optional cmd arg in the future -- 
    is a tuple representing a u64 of the actual value and the a str of the unit*/
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

    //represents the thing to execute and observe memory usage of
    let executable = match matches.value_of("executable") {

        Some(val) => val,

        None => {

            println!("No executable provided!");

            exit(0);

        }

    };

    //args to pass to the wrapped executable
    let exec_args = match matches.values_of("executable args") {

        Some(vals) => vals.collect::<Vec<&str>>().clone(),

        None => {
            let empty_vec: Vec<&str> = Vec::new();

            empty_vec
        }
    };

    run_process(executable, exec_args, interval);

}
