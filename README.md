# About

Process Interactive Kill is a command line tool that helps to find and kill process.
It works like pkill command but search is interactive.

## Bugs to fix

- [ ] Fix all TODO's
- [ ] Fix all FIXME's
- [ ] There is bug, if list of processes is empty TAB key will make index out of boundary exception
- [ ] Empty table is showing 1/0 instead of 0/0
- [ ] On linux Signal::Kill forces process to stop while Signal::Term terminate a process gracefully, add handling to check if os is supporting Term
- [ ] this + 2 is due to '> ' at the beginning, maybe some fix? `f.set_cursor(area.x + app.character_index as u16 + 2, area.y);`

## Optimization

- [ ] I think that search takes a lot of CPU try to optimize it, maybe rendering of all processes takes time too?
- [ ] We are using `let sys = System::new_all();` but maybe we don't need all the data it collects?
- [ ] Kill process is not performant, we are removing process from the middle of vec
- [ ] Think about creating table rows without extra allocations

## UI improvements

- [ ] Make UI more slick so that it won't take whole window, something like fzf search
- [ ] Handle empty results properly - maybe do not open UI at all?
- [ ] Maybe if there is no more processes we should exit immediately or exit after killing a process?
- [ ] Sometimes exe path is too long and it is truncated in UI, try to fix this

## Features to add

- [ ] Add option to search process by port requires [listeners](<https://github.com/GyulyVGC/listeners>)
- [ ] Add option to search by path
- [ ] Maybe info how much memory/cpu is used by process can be helpful
- [ ] Add option to search in cmd line args - doable but low priority
- [ ] Add option to search in environment variables - This is doable, maybe we can show it in process details?
- [ ] Add Process details - we can either add it at the bottom or add pop up with details
- [ ] Add option to ask if user wants to kill all processes (???)
