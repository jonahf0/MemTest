
mod run_process {


    //runs the process for taking measurements
    pub fn run_process(executable: &str, exec_args: Vec<&str>, interval: u64) -> BTreeMap<u64, HashMap<String, String>> {

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

}

pub use run_process::run_process;