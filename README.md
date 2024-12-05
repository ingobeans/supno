# supno
![supno_ex](https://github.com/user-attachments/assets/f7b0f2e6-c6c7-4623-a309-bae9b36217b4)

supno, a super note taking app (imho) for the command line. its designed to be fast to navigate. you'll use a command line to navigate through your notes but it will be constantly autocompleting and you'll rarely use actual commands.

written in rust uses my [cool-rust-input](https://github.com/ingobeans/cool-rust-input) crate for input.

all data is stored in the ✨cloud✨ through [jsonbio.io](https://jsonbin.io/). make and account and create a bin with the key "supno keep" with any value. add the api config to config.yaml and you're golden!

## general quick start

when launchuing the program you'll be greeted with a command line, with this information displayed:

1. the first line states your current working directory.
2. the second line the items in the current directory (green means directory, blue means note)
3. the third line will contain output of previous command / action.

type a note's name to start editing it. type a directory's name to navigate to it. type any amount of "." to navigate back a directory.

when the program autocompletes in the command line, you can press enter and it will submit that autocompletion. no need to press tab or a key like that.

## commands
* n <note_name> - create a new note in current dir
* d <dir_name> - create a new dir in current dir
* rm <dir_or_note_name> - remove an item in the current dir
* exit - exit program and sync all changes
* abort - exit program without syncing any changes (dangerous!)

## keyboard shortcuts
in command line:
* esc - navigate back a dir (will close program and save if at root)

in note:
* ctrl+x - save current note and exit
* ctrl+s - save current note
* ctrl+q - exit current note without saving
