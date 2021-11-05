use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};

//test

/*reads the /proc files, the first arg is the PID of the process that gets spawned,
and the second argument is the specific file to read from; currently, it can just accept
None in order to default to "smaps_rollup"*/
pub fn read_smaps_rollup(pid: u32) -> Option<HashMap<String, String>> {
    
    let file_contents = match read_procs_file(pid, "smaps_rollup"){

        Some(val) => val,

        None => return None

    };

    /*used for constructing a map of every line in the file;
    they all have predictable formats like "Rss: <number> kb"*/
    let mut return_hash_map: HashMap<String, String> = HashMap::new();

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
    temp.iter().for_each(|line| {
        /*The key is the initial index of the line, which should actually be a key.
        In order to account for lines that might have an unexpected format (like of
        I try reading from "status" instead of "smaps_rollup"), the default "empty"
        is used.*/
        return_hash_map.insert(
            line.get(0).unwrap_or(&"empty".to_string()).to_string(),
            format!(
                "{} {}",
                line.get(1).unwrap_or(&"empty".to_string()),
                line.get(2).unwrap_or(&"empty".to_string())
            ),
        );
    });

    return Some(return_hash_map);
}


//reads from the heap memory
pub fn read_heap(pid: u32) -> Option<Vec<u8>> {
    
    //read from maps
    let file_contents = match read_procs_file(pid, "maps"){

        Some(val) => val,

        None => return None

    };

    //indicate the start and end of the designated heap memory
    let mut start = 0;
    let mut end = 0;

    //look for the [heap] line in maps
    for line in file_contents.lines() {

        if line.contains("[heap]") {
            let split_line = line.split(" ").collect::<Vec<&str>>()[0].split("-").collect::<Vec<&str>>();

            start = u64::from_str_radix(split_line[0], 16).unwrap();

            end = u64::from_str_radix(split_line[1], 16).unwrap();

        }

    }
    
    //try opening the heap
    let mut mem_contents =  match fs::File::open(format!("/proc/{}/mem", pid)) {

        Ok(file) => file,

        Err(err) => {

            println!("There was an error opening the heap: {}", err);

            return None

        }

    };

    //seek to the location in the heap that can be read
    mem_contents.seek(SeekFrom::Start(start)).unwrap();

    let mut heap = vec![0; (end-start) as usize];

    //read the memory into the "heap" vector
    match mem_contents.read_exact(&mut heap[..]) {

        Ok(_) => {},

        Err(err) => {

            println!("There was an error reading the heap: {}", err);

            return None
        }

    };

    return Some(heap);

}

fn read_procs_file(pid: u32, file: &str) -> Option<String> {

    let proc_file = file;

    let file_location = format!("/proc/{}/{}", pid.to_string(), proc_file);

    //main reason for returning an Option -- may be an issue reading from the file
    let file_contents = match fs::read_to_string(file_location) {
        
        Ok(value) => Some(value),

        Err(_) => return None,
    };

    return file_contents

}
