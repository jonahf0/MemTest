
use std::process::{Command, exit};
use std::env;
use std::collections::HashMap;
use std::fs;

fn print_result(map: HashMap<String, String>) {

    println!("Resident set size: {}", map["Rss"]);
    println!("Proportional set size: {}", map["Pss"]);

}

fn read_from_proc(pid: u32) -> Option<HashMap<String, String>> {

    let smaps_rollup_location = format!("/proc/{}/smaps_rollup", pid.to_string());

    let mut return_hash_map: HashMap<String, String> = HashMap::new();

    let file_contents = match fs::read_to_string(smaps_rollup_location){

        Ok(value) => value,

        Err(_) => return None

    };
        
    let temp = file_contents
        .split("\n")
        .filter(|&line| { line != "" })
        .map( |line| -> Vec<String> {
            
            line.split_whitespace().collect::<Vec<&str>>().iter().map( |line| { line.replace(":", "") } ).collect()  

        }).collect::<Vec<Vec<String>>>();
    
 
        temp.iter().map( |line| {
        
        return_hash_map.insert(
                    line.get(0).unwrap().to_string(), 
                    format!("{} {}", line.get(1).unwrap(), line.get(2).unwrap())
                );
        }).for_each(drop);


        return Some(return_hash_map)
}

fn main() {
    
    let args: Vec<String> =  env::args().collect();
    
    let executable = match args.get(1){
        
        Some(val) => val,

        None => {
            println!("No executable was provided!");    

            exit(0)
        }
    
    };
    
    let exec_args = match args.get(2..) {
        
        Some(vals) => vals,

        None => &[]

    };

    let mut proc_child = match Command::new(executable).args(exec_args).spawn(){
        
        Ok(child) => child,

        Err(err) => {
            
            println!("There was an error: {}", err);

            exit(0);

        }

    };
   
    let pid = proc_child.id();

    let mem_info = read_from_proc(pid);

    match proc_child.wait() {

        Ok(_) => {},

        Err(err) => println!("There was an error with the process: {}", err)

    };

    match mem_info {

        Some(map) => print_result(map),

        None => {}

    };

}
