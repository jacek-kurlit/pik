# About

Process Interactive Kill is a linux command line tool that helps to find and kill process.
It works like pkill command but search is interactive.

## Todo

- [ ] Add search by string functionality (input)
- [x] Add process kill functionality
- [x] Add cmd line search param
- [ ] pik exe of current process should be filtered so user will not kill it
- [ ] Nice feature would be to kill process that is using some port
- [ ] Make UI more slick so that it won't take whole window, something like fzf search
- [x] Consider to use https://crates.io/crates/sysinfo as it is cross platform, it also allows to kill process
- [ ] Handle empty results properly - maybe do not open UI at all?
- [ ] Maybe if there is no more processes we should exit immediately or exit after killing a process?
- [ ] Add option to search in cmd line args - is this even needed?
