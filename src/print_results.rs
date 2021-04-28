use std::collections::{BTreeMap, HashMap};

//just a utility for printing the results
pub fn print_smaps_result(mem_info: BTreeMap<u64, HashMap<String, String>>, unit: &str) {
    //previous_data acts as a way to compare when the memory usage changes
    let mut previous_data = "".to_string();

    for (key, map) in mem_info {
        let current_data = format!(
            "Resident set size: {}\nProportional set size: {}\n",
            map["Rss"], map["Pss"],
        );

        //prints the data and puts current into previous if the previous data does not
        //equal the current (i.e. a change occured in smaps_rollup);
        //I could have used an if-else statement, but meh
        match previous_data == current_data {
            true => {}

            false => {

                print_formatted_time(key, unit);

                println!("{}", current_data);

                previous_data = current_data;
            }
        }
    }
}

/*
pub fn write_to_file(mem_info: BTreeMap<u64, HashMap<String, String>>, location: &str) {
    //previous_data acts as a way to compare when the memory usage changes
    let mut previous_data = "".to_string();

    for (key, map) in mem_info {
        let current_data = format!(
            "Resident set size: {}\nProportional set size: {}\n",
            map["Rss"], map["Pss"],
        );

        //prints the data and puts current into previous if the previous data does not
        //equal the current (i.e. a change occured in smaps_rollup);
        //I could have used an if-else statement, but meh
        match previous_data == current_data {
            true => {}

            false => {

                print_formatted_time(key, );

                println!("{}", current_data);

                previous_data = current_data;
            }
        }
    }

}
*/

fn print_formatted_time(key: u64, unit: &str) {

    let time_passed = match unit {
        
        "us" => key, 

        "ms" => key / 1000,

        _ => key,
    };

    println!("[{} {}]:", time_passed, unit);

}
