mod read_heap {

    use std::io::{Read, Seek, SeekFrom};

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

}