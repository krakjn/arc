# Advanced Repository Control
 _The bridge across repositories_

## Installation
  Windows users, go ahead and run `install/install.exe`  
  then within your workspace folder open up cmd-prompt or git-bash and type: `arc init`  
  __and you are done!__
### 1) Place [this](arc.json) configuration file in top level folder
**_parsed on the fly_**, meaning that removal/addition of entries in `arc.json` is captured every run of `arc`
  - **alongside** `avilon-lru` & `avilon-lru-core` & `sandel-development`
  - in example ```C:\Users\username\workspace\git```
  - **NOTE:** _arc will anchor and run off the location of arc.json, once placed do not move the file_

### 2) Download a binary for your system [**here**](ftp://10.0.0.45/arc-releases/)
  - grab the latest release and _**rename**_ it to `arc`
### 3) Load binary into `PATH` variable
  - Windows -> `C:\bin`
  - Mac -> `/usr/bin` or `/usr/local/bin`
  - Linux -> `/usr/bin/` for all users or `~/bin` just for you

## Initialize
  `arc init` will run parsing the `arc.json` file and load all the repos

## Useful Commands
|Command (optional) | Description|
|:-------------------------------|:---------------------------------------------------|            
| **-h, --help**                 | display this help message on the command line|
|**-v, --version**               | print out version number of arc |
|**init**                        | initialize arc |
|**info**                        | print contact info, and purpose of arc|
|**status** (**-v**)             | display status of all projects (-v for verbose)|
|**checkout** (**-b**)           | checkout <branch_name> (-b) to create|
|**pull**						 | pulls down all repos, on current branch|
|**sync**                        | checkout master, and pull down all repos|
|**reset** (**-s, -sanitize**)   | CAUTION: reset working tree to remote|
|**-c, cmd**<command>            | invoke any shell command across arc |
|**upgrade**                     | update binary to latest release for your system |

### NOTES
Understand that along with easing development by automating the synchronization of the entire project, we are trying to establish a __workflow__ via `arc`. Maintainers of their respective projects will not be affected by this flow as they will be the ones enforcing it. Developers working on a project will push to their _specific_ branches and send a _merge request_ to the maintainer when the issue is complete.  
