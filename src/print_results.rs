
use std::collections::HashMap;

//just a utility for printing the results
pub fn print_smaps_result(list: Vec<HashMap<String, String>>, unit: &str) {

    let mut previous_data = "".to_string();

    for (index, map) in list.iter().enumerate() {
            
            let current_data = format!("Resident set size: {}\nProportional set size: {}\n",
                                                map["Rss"],
                                                map["Pss"],
                                            );

            match previous_data == current_data {

                true => {},
                
                false => {
                    println!("[{} {}]:", index, unit);
                    println!("{}", current_data);

                    previous_data = current_data;
                }

            }

    }
}