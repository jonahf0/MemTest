pub(crate) 
use clap::{App, Arg};
use std::collections::{BTreeMap, HashMap};
use std::process::{exit, Command};
use std::sync::mpsc::channel;
use std::{thread, time};

mod print_results;
mod read_proc;

use print_results::print_smaps_result;
use read_proc::read_smaps_rollup;

fn main() {
    //use the clap crate to parse the arguments
    let matches = App::new("mem_test").about("\nDesigned to act as a simple command-line wrapper to test\nmemory usage of a program during its lifetime (think like the time utility)")
    .arg(Arg::from_usage("--interval [NUMBER] 'optional interval (in milliseconds [ms] or microseconds [us])\nto check memory usage; default is 1 millisecond'"))
    .arg(Arg::from_usage("--output [FILE] 'file location to write output to'"))
    .arg(Arg::with_name("executable")
        .help("The executable to use")
        .required(true))
    .arg(Arg::with_name("executable args")
        .help("The arguments to pass to the executable")
        .multiple(true)
        .required(false))
    .get_matches();

   let interval = match matches.value_of("interval") {

    Some(val) => str::parse(val).unwrap(),

    _ => 10

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

    let mem_info = run_process(executable, exec_args, interval);

    match matches.value_of("output") {


        Some(location) => println!("writing to file not currently implemented!"),//write_to_file(mem_info, location),

        None => print_smaps_result(mem_info)

    }
}


//runs the process for taking measurements
fn run_process(executable: &str, exec_args: Vec<&str>, interval: u64) -> BTreeMap<u64, HashMap<String, String>> {

    //the BTreeMap to pass to the print functions
    let mut mem_info_list: BTreeMap<u64, HashMap<String, String>> = BTreeMap::new();

    //variable used to track time in the loop
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
        /*start_time and elapsed_time are used to see how long it takes to read;
        it usually doesn't take long, but the program will get very unprecise after
        long runs */
        let start_time = time::Instant::now();

        let mem_info = read_smaps_rollup(pid);

        let elapsed_time = start_time.elapsed().as_micros();

        if let Some(map) = mem_info {
            time_variable += elapsed_time as u64;
            mem_info_list.insert(time_variable, map);
        } 

        time_variable += interval;
        
        if let Ok(_) = loop_rx.recv_timeout(time::Duration::from_millis(interval)) {
            break;
        }
    }

    match waiting_thread.join() {
        Ok(_) => {}

        Err(err) => println!("{:?}", err),
    }

    return mem_info_list
}