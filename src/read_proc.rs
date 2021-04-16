 use std::collections::HashMap;
 use std::fs;

/*reads the /proc files, the first arg is the PID of the process that gets spawned,
and the second argument is the specific file to read from; currently, it can just accept
None in order to default to "smaps_rollup"*/
pub fn read_smaps_rollup(pid: u32) -> Option<HashMap<String, String>> {

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