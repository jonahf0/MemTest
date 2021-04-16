A simple "wrapper" for testing memory usage of applications on Linux systems. 

Due to the inaccuracy / variability of memory usage metrics given by the /proc files, multiple metrics are given so developers can make judgements based on what they are looking for. However, this tool is meant for general "eye-balling" of memory usage. The closest comparison is the Linux time cmd utility, which just acts as a wrapper to give basic timing info for the running program.

From my understanding, the metrics displayed represent the following:

1. Resident Set Size: "actual" memory usage, private + shared memory space

2. Proportional Set Size: a better approximation of "actual" memory usage,
                        reduced shared memory space (dividing shared space with
                        all the processes using it) + private
                        
One of the primary reasons for writing this script was to learn Rust. Another reason for writing this script was to add to the body of memory testing programs that currently exist; hopefully this script can successfully help to determine relevant metrics of memory usage (maybe even including heap memory usage).
