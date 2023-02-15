# arc - Aggregate Repository Control

## _The bridge across repositories_  

```text
USAGE:
    arc [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -v, --version    Displays version of arc

OPTIONS:
    -c <COMMAND>...        Invoke any command across all repos
    -g <COMMAND>...        Invoke any git command across all repos

SUBCOMMANDS:
    checkout    Checks out all dependent repos on same branch
    help        Prints this message or the help of the given subcommand(s)
    info        Print contact info, and purpose of arc
    init        Initialize all repos defined in arc.json
    pull        Fetch and merge on current branch
    reset       Reset worktree to origin/master
    show        Display current configuration
    status      Show status of all repos in arc.json
    sync        Checkout master and pull
```

### 1) Credential Manager

- Windows  
  - install the latest [Git Credential Manager](https://github.com/microsoft/Git-Credential-Manager-for-Windows/releases)
- Linux/Mac

        git config --global credential.helper cache
        git config --global credential.helper "store --file=~/.my-credentials"

- The credential helper command will store credentials for 15 mins. Passing  
  the `--file` argument will store your credentials permanently, however they  
  are in ***plain text***. For more info look at the  
  [git-docs](https://git-scm.com/book/id/v2/Git-Tools-Credential-Storage)

  - An example credential file will look like this (only one line):  

      `http://username:3YU567IoTdcDGMor@domain`

### 2) Place [this](config/arc.json) configuration file in top level folder

**_parsed on the fly_**, meaning that **removal/addition** of entries in `arc.json` is captured every run of `arc`

> NOTE: _arc will anchor and run off the location of `arc.json`, once placed do not move the file_

### 3) Download latest [**release**](http://gitlab.com/krakjn/arc/-/releases) for your system

### 4) Place `arc` on PATH
Some common places
- Windows: create directory `C:\bin`, append to system PATH
- Mac: `/usr/local/bin`
- Linux: `/usr/bin/` for all users or `~/bin` just for you

### 5) Initialize

- run `arc init`
- arc will clone all repos defined in `arc.json`.  
- _Feature Enhancement:_ allow for multiple remotes, currently only one is supported `origin`.

#### `arc upgrade`

_Under Maintenance:_  In the future, `arc` will be able to update the binary from the command line.

### Workflow

Understand that along with easing development by automating the synchronization of the entire project, `arc` could prove useful as a __workflow__ tool. Adding the capability of git hooks could prove beneficial to aide issue management in GitLab. Maintainers of their respective projects will not be affected by this flow as they will be the ones enforcing it. Developers working on a project will push to their _specific_ branches and send a _merge request_ to the maintainer when the issue is complete.
